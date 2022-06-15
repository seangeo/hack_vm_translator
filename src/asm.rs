use crate::vm::{Command, Segment, SourceCommand};
use indoc::formatdoc;

pub fn generate_code(commands: Vec<SourceCommand>) -> Result<Vec<String>, String> {
    let mut scope: Vec<String> = Vec::new();
    let mut should_bootstrap = false;

    let mut instructions =
        commands
        .iter()
        .map(|source_command|{
            if let Command::Function {name: function, nvars: _} = source_command.command() {
                should_bootstrap = should_bootstrap || *function == "Sys.init";
                scope.push(format!("{function}"));
            }

            generate_code_for_command(&source_command, scope.last())
        }).collect::<Result<Vec<String>, String>>()?;

    if should_bootstrap {
        instructions.insert(0, bootstrap());
    }

    Ok(instructions)
}

fn bootstrap() -> String {
    let mut asm: Vec<String> = Vec::new();
    asm.push(formatdoc!(
        "@256
        D=A
        @SP
        M=D
        @LCL
        M=-1
        @2
        D=-A
        @ARG
        M=D
        @3
        D=-A
        @THIS
        M=D
        @4
        D=-A
        @THAT
        M=D"
    ));

    let command = Command::Call { name: "Bootstrap", nargs: 0 };
    let sc = SourceCommand::bootstrap(command);
    asm.push(generate_call(&sc, "Sys.init", 0, Some(&"Bootstrap".to_string())).unwrap());

    asm.join("\n")
}

fn generate_code_for_command(source_command: &SourceCommand, scope: Option<&String>) -> Result<String, String> {
    let code = match source_command.command() {
        Command::Add => generate_add(),
        Command::And => generate_and(),
        Command::Eq => generate_eq(source_command),
        Command::Gt => generate_gt(source_command),
        Command::Lt => generate_lt(source_command),
        Command::Neg => generate_neg(),
        Command::Not => generate_not(),
        Command::Or => generate_or(),
        Command::Pop { segment, index } => generate_pop(source_command, segment, *index),
        Command::Push { segment, index } => generate_push(source_command, segment, *index),
        Command::Sub => generate_sub(),
        Command::Goto(label) => generate_goto(source_command, label, scope),
        Command::IfGoto(label) => generate_if_goto(source_command, label, scope),
        Command::Label(label) => generate_label(source_command, label, scope),
        Command::Call {name, nargs } => generate_call(source_command, name, *nargs, scope),
        Command::Function { name, nvars } => generate_function(name, *nvars),
        Command::Return => generate_return(),
    };

    if let Ok(code) = code {
        let mut result = String::new();
        result.push_str(&comment(source_command));
        result.push_str(&code);
        Ok(result)
    } else {
        code
    }
}

fn generate_if_goto(source_command: &SourceCommand, label: &str, scope: Option<&String>) -> Result<String, String> {
    let file = source_command.file_base().to_string();
    let label_scope = scope.unwrap_or(&file);
    let mut asm: Vec<String> = Vec::new();
    asm.push(pop_d());
    asm.push(formatdoc!(
            "@{label_scope}${label}
            D;JNE"));
    Ok(asm.join("\n"))
}

fn generate_goto(source_command: &SourceCommand, label: &str, scope: Option<&String>) -> Result<String, String> {
    let file = source_command.file_base().to_string();
    let label_scope = scope.unwrap_or(&file);

    Ok(formatdoc!(
            "@{label_scope}${label}
             0;JMP"))
}

fn generate_label(source_command: &SourceCommand, label: &str, scope: Option<&String>) -> Result<String, String> {
    let file = source_command.file_base().to_string();
    let label_scope = scope.unwrap_or(&file);

    Ok(format!("({label_scope}${label})"))
}

fn generate_call(source_command: &SourceCommand, name: &str, nargs: u16, scope: Option<&String>) -> Result<String, String> {
    let arg_offset = nargs + 5;
    let file = source_command.file_base().to_string();
    let label_scope = scope.unwrap_or(&file);
    let return_label = format!("{label_scope}$ret.{}", source_command.line());
    let mut asm: Vec<String> = Vec::new();
    asm.push(formatdoc!(
        "@{return_label}
        D=A"
    ));
    asm.push(push_d());
    asm.push(push_symbol("LCL"));
    asm.push(push_symbol("ARG"));
    asm.push(push_symbol("THIS"));
    asm.push(push_symbol("THAT"));
    // ARG= SP - 5 - args
    asm.push(formatdoc!(
        "@SP
        D=M
        @{arg_offset}
        D=D-A
        @ARG
        M=D"
    ));
    asm.push(formatdoc!(
        "@SP
        D=M
        @LCL
        M=D"
    ));
    asm.push(formatdoc!(
        "@{name}
        0;JMP"
    ));
    asm.push(format!("({return_label})"));

    Ok(asm.join("\n"))
}

fn generate_function(name: &str, nvars: u16) -> Result<String, String> {
    let mut asm: Vec<String> = Vec::new();
    asm.push(format!("({name})"));

    for _ in 0..nvars {
        asm.push(push_constant(0)?);
    }

    Ok(asm.join("\n"))
}

fn generate_return() -> Result<String, String> {
    let mut asm: Vec<String> = Vec::new();
    // frame = LCL
    asm.push(formatdoc!(
        "@LCL
         D=M
         @frame
         M=D"));
    // retAdd = *(frame - 5)
    asm.push(formatdoc!(
        "@frame
        D=M
        @5
        A=D-A
        D=M
        @retaddr
        M=D"
    ));
    // *arg = pop
    asm.push(pop_d());
    asm.push(formatdoc!(
        "@ARG
         A=M
         M=D"
    ));
    // sp = arg + 1
    asm.push(formatdoc!(
        "@ARG
         D=M+1
         @SP
         M=D"
    ));
    // that = *(frame - 1)
    // this = *(frame - 2)
    // arg = *(frame - 3)
    // lcl = *(frame - 4)
    asm.push(formatdoc!(
        "@frame
        AM=M-1
        D=M
        @THAT
        M=D
        @frame
        AM=M-1
        D=M
        @THIS
        M=D
        @frame
        AM=M-1
        D=M
        @ARG
        M=D
        @frame
        AM=M-1
        D=M
        @LCL
        M=D"
    ));
    // goto retaddr
    asm.push(formatdoc!(
        "@retaddr
        A=M
        0;JMP"
    ));


    Ok(asm.join("\n"))
}

fn generate_and() -> Result<String, String> {
    generate_binary_operation("D&M")
}

fn generate_or() -> Result<String, String> {
    generate_binary_operation("D|M")
}

fn generate_add() -> Result<String, String> {
    generate_binary_operation("D+M")
}

fn generate_sub() -> Result<String, String> {
    generate_binary_operation("M-D")
}

fn generate_neg() -> Result<String, String> {
    generate_unary("-D")
}

fn generate_not() -> Result<String, String> {
    generate_unary("!D")
}

fn generate_pop(sc: &SourceCommand, segment: &Segment, index: u16) -> Result<String, String> {
    match segment {
        Segment::Argument => pop_to_segment("ARG", index),
        Segment::Local => pop_to_segment("LCL", index),
        Segment::Pointer => pop_to_address(index + 3),
        Segment::Static => pop_to_variable(&format!("{}.{index}", sc.file_base())),
        Segment::Temp => pop_to_address(index + 5),
        Segment::That => pop_to_segment("THAT", index),
        Segment::This => pop_to_segment("THIS", index),
        _ => Err(format!("Unable to address segment for pop: {segment:?}")),
    }
}

fn pop_to_address(address: u16) -> Result<String, String> {
    pop_to_variable(&address.to_string())
}

fn pop_to_segment(segment_name: &str, index: u16) -> Result<String, String> {
    let mut asm: Vec<String> = Vec::new();
    asm.push(formatdoc!(
        "@{segment_name}
        D=M
        @{index}
        D=D+A
        @SP
        AM=M-1
        D=D+M
        A=D-M
        M=D-A"
    ));

    Ok(asm.join("\n"))
}

fn pop_to_variable(variable: &str) -> Result<String, String> {
    let mut asm: Vec<String> = Vec::new();
    asm.push(pop_d());
    asm.push(formatdoc!(
        "@{variable}
        M=D"
    ));

    Ok(asm.join("\n"))
}

fn generate_push(sc: &SourceCommand, segment: &Segment, index: u16) -> Result<String, String> {
    let file = sc.file_base();

    match segment {
        Segment::Argument => push_from_segment("ARG", index),
        Segment::Constant => push_constant(index),
        Segment::Local => push_from_segment("LCL", index),
        Segment::Pointer => push_from_address(index + 3),
        Segment::Static => push_from_variable(&format!("{file}.{index}")),
        Segment::Temp => push_from_address(index + 5),
        Segment::That => push_from_segment("THAT", index),
        Segment::This => push_from_segment("THIS", index),
    }
}

fn push_from_variable(variable: &str) -> Result<String, String> {
    let mut asm: Vec<String> = Vec::new();
    asm.push(formatdoc!(
        "@{variable}
        D=M
        "
    ));
    asm.push(push_d());

    Ok(asm.join("\n"))
}

fn push_from_address(index: u16) -> Result<String, String> {
    push_from_variable(&index.to_string())
}

fn push_from_segment(segment_name: &str, index: u16) -> Result<String, String> {
    let mut asm: Vec<String> = Vec::new();
    asm.push(formatdoc!(
        "@{index}
        D=A
        @{segment_name}
        A=D+M
        D=M"
    ));
    asm.push(push_d());

    Ok(asm.join("\n"))
}

fn push_constant(value: u16) -> Result<String, String> {
    let mut asm: Vec<String> = Vec::new();
    asm.push(format!("@{value}"));
    asm.push(format!("D=A"));
    asm.push(push_d());
    Ok(asm.join("\n"))
}

fn generate_binary_operation(op: &str) -> Result<String, String> {
    let mut asm: Vec<String> = Vec::new();
    asm.push(pop_d());
    asm.push(formatdoc!(
        "
        @SP
        AM=M-1
        D={op}"
    ));
    asm.push(push_d());
    Ok(asm.join("\n"))
}

fn generate_unary(op: &str) -> Result<String, String> {
    let mut asm: Vec<String> = Vec::new();
    asm.push(pop_d());
    asm.push(formatdoc!("D={op}"));
    asm.push(push_d());

    Ok(asm.join("\n"))
}

fn generate_eq(source_command: &SourceCommand) -> Result<String, String> {
    generate_comparison(source_command, "JEQ")
}

fn generate_gt(source_command: &SourceCommand) -> Result<String, String> {
    generate_comparison(source_command, "JGT")
}

fn generate_lt(source_command: &SourceCommand) -> Result<String, String> {
    generate_comparison(source_command, "JLT")
}

// This generates a comparison process that will
// store X - Y in the D register, where Y is the top
// element of the stack, and X is second from the top.
// Callers should pass a value for `comp` that is a
// Hack jump command, that will jump if the required
// comparison is true based on the value of D.
//
fn generate_comparison(sc: &SourceCommand, comp: &str) -> Result<String, String> {
    let file = sc.file_base();
    let line = sc.line();
    let mut asm: Vec<String> = Vec::new();
    asm.push(pop_d());
    asm.push(formatdoc!(
        "
        @SP
        AM=M-1
        D=M-D
        @COMP_TRUE_{file}.{line}
        D;{comp}
        @0
        D=A
        @COMP_END_{file}.{line}
        0;JMP
        (COMP_TRUE_{file}.{line})
        @1
        D=-A
        (COMP_END_{file}.{line})"
    ));
    asm.push(push_d());

    Ok(asm.join("\n"))
}

fn comment(source_command: &SourceCommand) -> String {
    format!(
        "// {}[{}]: {}\n",
        source_command.file_base(),
        source_command.line(),
        source_command.source()
    )
}

fn pop_d() -> String {
    formatdoc!(
        "
        @SP
        AM=M-1
        D=M"
    )
}

fn push_d() -> String {
    formatdoc!(
        "
        @SP
        A=M
        M=D
        @SP
        M=M+1"
    )
}

fn push_symbol(symbol: &str) -> String {
    formatdoc!(
        "@{symbol}
        D=M
        {}", push_d()
    )
}
