use crate::vm::{Command, Segment, SourceCommand};
use indoc::formatdoc;

pub fn generate_code(commands: Vec<SourceCommand>) -> Result<Vec<String>, String> {
    commands
        .iter()
        .map(|command| generate_code_for_command(&command))
        .collect()
}

fn generate_code_for_command(source_command: &SourceCommand) -> Result<String, String> {
    match source_command.command() {
        Command::Add => generate_add(source_command),
        Command::And => generate_and(source_command),
        Command::Eq => generate_eq(source_command),
        Command::Gt => generate_gt(source_command),
        Command::Lt => generate_lt(source_command),
        Command::Neg => generate_neg(source_command),
        Command::Not => generate_not(source_command),
        Command::Or => generate_or(source_command),
        Command::Push { segment, index } => generate_push(source_command, segment, *index),
        Command::Sub => generate_sub(source_command),
        _ => Err(format!(
            "Code generation not implemented for [{}]: '{}'",
            source_command.line(),
            source_command.source()
        )),
    }
}

fn generate_and(source_command: &SourceCommand) -> Result<String, String> {
    generate_binary_operation(source_command, "D&M")
}

fn generate_or(source_command: &SourceCommand) -> Result<String, String> {
    generate_binary_operation(source_command, "D|M")
}

fn generate_add(source_command: &SourceCommand) -> Result<String, String> {
    generate_binary_operation(source_command, "D+M")
}

fn generate_sub(source_command: &SourceCommand) -> Result<String, String> {
    generate_binary_operation(source_command, "M-D")
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

fn generate_neg(sc: &SourceCommand) -> Result<String, String> {
    generate_unary(sc, "-D")
}

fn generate_not(sc: &SourceCommand) -> Result<String, String> {
    generate_unary(sc, "!D")
}

fn generate_push(sc: &SourceCommand, _segment: &Segment, index: u16) -> Result<String, String> {
    let mut asm: Vec<String> = Vec::new();
    asm.push(comment(sc));
    asm.push(format!("@{index}"));
    asm.push(format!("D=A"));
    asm.push(push_d_onto_stack());
    Ok(asm.join("\n"))
}

fn generate_binary_operation(source_command: &SourceCommand, op: &str) -> Result<String, String> {
    let mut asm: Vec<String> = Vec::new();
    asm.push(comment(source_command));
    asm.push(pop_to_d());
    asm.push(formatdoc!(
        "
        @SP
        M=M-1
        A=M
        D={op}"
    ));
    asm.push(push_d_onto_stack());
    Ok(asm.join("\n"))
}

fn generate_unary(sc: &SourceCommand, op: &str) -> Result<String, String> {
    let mut asm: Vec<String> = Vec::new();
    asm.push(comment(sc));
    asm.push(pop_to_d());
    asm.push(formatdoc!("D={op}"));
    asm.push(push_d_onto_stack());

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
    let line = sc.line();
    let mut asm: Vec<String> = Vec::new();
    asm.push(comment(sc));
    asm.push(pop_to_d());
    asm.push(formatdoc!(
        "
        @SP
        M=M-1
        A=M
        D=D-M
        @COMP_TRUE_{line}
        D;{comp}
        @0
        D=A
        @COMP_END_{line}
        0;JMP
        (COMP_TRUE_{line})
        @1
        D=-A
        (COMP_END_{line})
        "
    ));
    asm.push(push_d_onto_stack());

    Ok(asm.join("\n"))
}

fn comment(source_command: &SourceCommand) -> String {
    format!("// [{}] {}", source_command.line(), source_command.source())
}

fn pop_to_d() -> String {
    formatdoc!(
        "
        @SP
        M=M-1
        A=M
        D=M"
    )
}

fn push_d_onto_stack() -> String {
    formatdoc!(
        "
        @SP
        A=M
        M=D
        @SP
        M=M+1"
    )
}
