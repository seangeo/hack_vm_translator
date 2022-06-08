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
        Command::Push { segment, index } => generate_push(source_command, segment, *index),

        _ => Err(format!(
            "Code generation not implemented for [{}]: '{}'",
            source_command.line(),
            source_command.source()
        )),
    }
}

fn generate_add(source_command: &SourceCommand) -> Result<String, String> {
    let mut asm: Vec<String> = Vec::new();
    asm.push(comment(source_command));
    asm.push(pop_to_d());
    asm.push(formatdoc!(
        "
        @SP
        M=M-1
        A=M
        D=D+M",
    ));
    asm.push(push_d_onto_stack());
    Ok(asm.join("\n"))
}

fn generate_push(sc: &SourceCommand, _segment: &Segment, index: u16) -> Result<String, String> {
    let mut asm: Vec<String> = Vec::new();
    asm.push(comment(sc));
    asm.push(format!("@{}", index));
    asm.push(format!("D=A"));
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
