use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Tui,
    Web {
        local: bool,
        passthrough: Vec<String>,
    },
}

pub fn parse_invocation(program_name: &str, args: &[String]) -> Result<Command, String> {
    match binary_name(program_name).as_deref() {
        Some("fzts") => parse_fzts(args),
        _ => parse_play(args),
    }
}

fn parse_play(args: &[String]) -> Result<Command, String> {
    match args.split_first() {
        None => Ok(Command::Tui),
        Some((command, rest)) if command == "web" => Ok(Command::Web {
            local: false,
            passthrough: rest.to_vec(),
        }),
        Some((other, _)) => Err(format!("unknown command: {other}. try `play web`")),
    }
}

fn parse_fzts(args: &[String]) -> Result<Command, String> {
    match args.split_first() {
        None => Ok(Command::Web {
            local: true,
            passthrough: Vec::new(),
        }),
        Some((first, _)) if first.starts_with('-') => Ok(Command::Web {
            local: true,
            passthrough: args.to_vec(),
        }),
        Some((command, rest)) if command == "web" => Ok(Command::Web {
            local: true,
            passthrough: rest.to_vec(),
        }),
        Some((command, rest)) if command == "play" => match rest.split_first() {
            None => Ok(Command::Web {
                local: true,
                passthrough: Vec::new(),
            }),
            Some((nested, remainder)) if nested == "web" => Ok(Command::Web {
                local: true,
                passthrough: remainder.to_vec(),
            }),
            Some((nested, _)) => Err(format!("unknown command: {nested}. try `fzts play web`")),
        },
        Some((other, _)) => Err(format!("unknown command: {other}. try `fzts play web`")),
    }
}

fn binary_name(program_name: &str) -> Option<String> {
    Path::new(program_name)
        .file_stem()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
}
