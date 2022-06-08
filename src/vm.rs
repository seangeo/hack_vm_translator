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
pub enum Command {
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
}

impl FromStr for Command {
    type Err = String;

    fn from_str(line: &str) -> Result<Command, String> {
        Command::from_str(line)
    }
}

impl Command {
    fn from_str(line: &str) -> Result<Command, String> {
        if let Some(s) = line.strip_prefix("push") {
            Command::parse_push(s.trim())
        } else if let Some(s) = line.strip_prefix("pop") {
            Command::parse_pop(s.trim())
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
        } else {
            Err(format!("Parser not implemented for '{}'", line))
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
                index: index,
            }),
            Err(e) => Err(e),
        }
    }

    fn parse_stack_arguments(s: &str) -> Result<(Segment, u16), String> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() == 2 {
            let segment = parts[0].parse::<Segment>();
            let index = parts[1].parse::<u16>();

            if segment.is_err() {
                Err(segment.unwrap_err())
            } else if index.is_err() {
                Err(format!("Error parsing index: {}", index.unwrap_err()))
            } else {
                Ok((segment.unwrap(), index.unwrap()))
            }
        } else {
            Err(format!("push expected format 'push <segment> <index>'"))
        }
    }
}

#[derive(Debug)]
pub struct SourceCommand<'a> {
    line: usize,
    command: Command,
    source: &'a str,
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
}

pub fn parse_source(source: &str) -> Result<Vec<SourceCommand>, String> {
    source
        .lines()
        .enumerate()
        .filter_map(|(i, line)| match strip_comments(line) {
            Some(s) => Some(parse_source_command(i, s)),
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

fn parse_source_command<'a>(i: usize, source: &'a str) -> Result<SourceCommand<'a>, String> {
    match source.parse::<Command>() {
        Ok(command) => Ok(SourceCommand {
            line: i,
            command: command,
            source: source,
        }),
        Err(e) => Err(format!("Parse error at line {} ({}): {}", i, source, e)),
    }
}
