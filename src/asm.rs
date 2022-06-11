use crate::vm::{Command, Segment, SourceCommand};
use indoc::formatdoc;

pub fn generate_code(commands: Vec<SourceCommand>) -> Result<Vec<String>, String> {
    commands
        .iter()
        .map(|command| generate_code_for_command(&command))
        .collect()
}

fn generate_code_for_command(source_command: &SourceCommand) -> Result<String, String> {
    let code = match source_command.command() {
        Command::Add => generate_add(),
        Command::And => generate_and(),
        Command::Eq => generate_eq(source_command),
        Command::Gt => generate_gt(source_command),
        Command::Goto(label) => generate_goto(source_command, label),
        Command::IfGoto(label) => generate_if_goto(source_command, label),
        Command::Label(label) => generate_label(source_command, label),
        Command::Lt => generate_lt(source_command),
        Command::Neg => generate_neg(),
        Command::Not => generate_not(),
        Command::Or => generate_or(),
        Command::Pop { segment, index } => generate_pop(source_command, segment, *index),
        Command::Push { segment, index } => generate_push(source_command, segment, *index),
        Command::Sub => generate_sub(),
        _ => Err(format!(
            "Code generation not implemented for [{}]: '{}'",
            source_command.line(),
            source_command.source()
        )),
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

fn generate_if_goto(source_command: &SourceCommand, label: &str) -> Result<String, String> {
    let file = source_command.file_base();
    let mut asm: Vec<String> = Vec::new();
    asm.push(pop_d());
    asm.push(formatdoc!(
            "@{file}.{label}
            D;JNE"));
    Ok(asm.join("\n"))
}

fn generate_goto(source_command: &SourceCommand, label: &str) -> Result<String, String> {
    let file = source_command.file_base();

    Ok(formatdoc!(
            "@{file}.{label}
             0;JMP"))
}

fn generate_label(source_command: &SourceCommand, label: &str) -> Result<String, String> {
    let file = source_command.file_base();

    Ok(format!("({file}.{label})"))
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

fn generate_eq(source_command: &SourceCommand) -> Result<String, String> {
    generate_comparison(source_command, "JEQ")
}

fn generate_gt(source_command: &SourceCommand) -> Result<String, String> {
    // This is JLT (less than) because the semantics
    // are x > y where y is the top of the stack, and
    // x is below it. Since D = Y and M = X, and D=D-M
    // we need to reverse the comparison.
    generate_comparison(source_command, "JLT")
}
fn generate_lt(source_command: &SourceCommand) -> Result<String, String> {
    generate_comparison(source_command, "JGT")
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
        M=M-1
        A=M
        D=D-M
        @COMP_TRUE_{file}.{line}
        D;{comp}
        @0
        D=A
        @COMP_END_{file}.{line}
        0;JMP
        (COMP_TRUE_{file}.{line})
        @1
        D=-A
        (COMP_END_{file}.{line})
        "
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
