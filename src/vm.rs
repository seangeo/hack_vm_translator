use std::str::FromStr;

#[derive(Debug)]
pub enum Segment {
    Argument,
    Constant,
    Local,
    Pointer,
    Static,
    Temp,
    That,
    This,
}

impl FromStr for Segment {
    type Err = String;

    fn from_str(s: &str) -> Result<Segment, String> {
        match s {
            "argument" => Ok(Segment::Argument),
            "constant" => Ok(Segment::Constant),
            "local" => Ok(Segment::Local),
            "pointer" => Ok(Segment::Pointer),
            "static" => Ok(Segment::Static),
            "temp" => Ok(Segment::Temp),
            "that" => Ok(Segment::That),
            "this" => Ok(Segment::This),
            _ => Err(format!("Unknown segment name: '{}'", s)),
        }
    }
}

#[derive(Debug)]
pub enum Command<'a> {
    Push { segment: Segment, index: u16 },
    Pop { segment: Segment, index: u16 },
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
    Goto(&'a str),
    IfGoto(&'a str),
    Label(&'a str ),
    Function { name: &'a str, nvars: u16 },
    Return,
}

impl<'a> Command<'a> {
    fn from_str(line: &'a str) -> Result<Command<'a>, String> {
        if let Some(s) = line.strip_prefix("push") {
            Command::parse_push(s.trim())
        } else if let Some(s) = line.strip_prefix("pop") {
            Command::parse_pop(s.trim())
        } else if let Some(s) = line.strip_prefix("label") {
            Command::parse_label(s.trim())
        } else if let Some(s) = line.strip_prefix("if-goto") {
            Command::parse_if_goto(s)
        } else if let Some(s) = line.strip_prefix("goto") {
            Command::parse_goto(s)
        } else if let Some(s) = line.strip_prefix("function") {
            Command::parse_function(s)
        } else if line == "add" {
            Ok(Command::Add)
        } else if line == "sub" {
            Ok(Command::Sub)
        } else if line == "neg" {
            Ok(Command::Neg)
        } else if line == "eq" {
            Ok(Command::Eq)
        } else if line == "lt" {
            Ok(Command::Lt)
        } else if line == "gt" {
            Ok(Command::Gt)
        } else if line == "and" {
            Ok(Command::And)
        } else if line == "or" {
            Ok(Command::Or)
        } else if line == "not" {
            Ok(Command::Not)
        } else if line == "return" {
            Ok(Command::Return)
        } else {
            Err(format!("Parser not implemented for '{}'", line))
        }
    }

    fn parse_label(s: &str) -> Result<Command, String> {
        match Self::parse_label_name(s) {
            Ok(name) => Ok(Command::Label(name)),
            Err(e) => Err(e),
        }
    }

    fn parse_if_goto(s: &str) -> Result<Command, String> {
        match Self::parse_label_name(s) {
            Ok(name) => Ok(Command::IfGoto(name)),
            Err(e) => Err(e),
        }
    }

    fn parse_goto(s: &str) -> Result<Command, String> {
        match Self::parse_label_name(s) {
            Ok(name) => Ok(Command::Goto(name)),
            Err(e) => Err(e),
        }
    }

    fn parse_function(s: &str) -> Result<Command, String> {
        match Self::parse_label_and_n(s) {
            Ok((name, n)) => Ok(Command::Function {
                name: name,
                nvars: n
            }),
            Err(e) => Err(e)
        }
    }

    fn parse_label_name(s: &str) -> Result<&str, String> {
        let s = s.trim();
        if s.is_empty() {
            Err(format!("Label must have a name"))
        } else {
            Ok(s)
        }
    }

    fn parse_pop(s: &str) -> Result<Command, String> {
        match Command::parse_stack_arguments(s) {
            Ok((segment, index)) => Ok(Command::Pop {
                segment: segment,
                index: index,
            }),
            Err(e) => Err(e),
        }
    }

    fn parse_push(s: &str) -> Result<Command, String> {
        match Command::parse_stack_arguments(s) {
            Ok((segment, index)) => Ok(Command::Push {
                segment: segment,
                index: index, // TODO validate index based on segment
            }),
            Err(e) => Err(e),
        }
    }

    fn parse_stack_arguments(s: &str) -> Result<(Segment, u16), String> {
        match Self::parse_label_and_n(s) {
            Ok((label, n)) => {
                let segment  = label.parse::<Segment>();

                if segment.is_err() {
                    Err(segment.unwrap_err())
                } else {
                    Ok((segment.unwrap(), n))
                }
            },
            Err(e) => Err(e)
        }
    }

    fn parse_label_and_n(s: &str) -> Result<(&str, u16), String> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() == 2 {
            let name = parts[0];
            let index = parts[1].parse::<u16>();

            if index.is_err() {
                Err(format!("Error parsing index: {}", index.unwrap_err()))
            } else {
                Ok((name, index.unwrap()))
            }
        } else {
            Err(format!("expected format '<string> <int>'"))
        }
    }
}

#[derive(Debug)]
pub struct SourceCommand<'a> {
    line: usize,
    command: Command<'a>,
    source: &'a str,
    file_base: &'a str,
}

impl<'a> SourceCommand<'a> {
    pub fn line(&self) -> usize {
        self.line
    }

    pub fn source(&self) -> &str {
        self.source
    }

    pub fn command(&self) -> &Command {
        &self.command
    }

    pub fn file_base(&self) -> &str {
        &self.file_base
    }
}

pub fn parse_source<'a>(
    file_base: &'a str,
    source: &'a str,
) -> Vec<Result<SourceCommand<'a>, String>> {
    source
        .lines()
        .enumerate()
        .filter_map(|(i, line)| match strip_comments(line) {
            Some(s) => Some(parse_source_command(file_base, i, s)),
            None => None,
        })
        .collect()
}

fn strip_comments(line: &str) -> Option<&str> {
    let code = match line.find("//") {
        None => line,
        Some(i) => {
            let (code, _) = line.split_at(i);
            code
        }
    }
    .trim();

    if code.is_empty() {
        None
    } else {
        Some(code)
    }
}

fn parse_source_command<'a>(
    file_base: &'a str,
    i: usize,
    source: &'a str,
) -> Result<SourceCommand<'a>, String> {
    match Command::from_str(source) {
        Ok(command) => Ok(SourceCommand {
            file_base: file_base,
            line: i,
            command: command,
            source: source,
        }),
        Err(e) => Err(format!(
            "Parse error at line {file_base}:{i} ({source}): {e}"
        )),
    }
}
