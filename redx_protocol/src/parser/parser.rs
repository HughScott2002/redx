use crate::Token;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Keys { pattern: String },
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    EmptyInput,
    UnknownCommand(String),
    InvalidArity {
        command: String,
        expected: usize,
        got: usize,
    },
}

pub fn parse(tokens: &[Token]) -> Result<Command, ParseError> {
    let first = tokens.first().ok_or(ParseError::EmptyInput)?;
    let command = first.value();

    if command.eq_ignore_ascii_case("exit") {
        return parse_exit(tokens);
    }

    if command.eq_ignore_ascii_case("KEYS") {
        return parse_keys(tokens);
    }

    Err(ParseError::UnknownCommand(command.to_string()))
}

fn parse_exit(tokens: &[Token]) -> Result<Command, ParseError> {
    if tokens.len() == 1 {
        Ok(Command::Exit)
    } else {
        Err(ParseError::InvalidArity {
            command: "exit".into(),
            expected: 0,
            got: tokens.len() - 1,
        })
    }
}

fn parse_keys(tokens: &[Token]) -> Result<Command, ParseError> {
    if tokens.len() != 2 {
        return Err(ParseError::InvalidArity {
            command: "KEYS".into(),
            expected: 1,
            got: tokens.len().saturating_sub(1),
        });
    }

    Ok(Command::Keys {
        pattern: tokens[1].value().to_string(),
    })
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EmptyInput => write!(formatter, "empty input"),
            ParseError::UnknownCommand(command) => write!(formatter, "unknown command: {command}"),
            ParseError::InvalidArity {
                command,
                expected,
                got,
            } => write!(
                formatter,
                "{command} expected {expected} argument(s), got {got}"
            ),
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenize;

    #[test]
    fn parses_keys_pattern() {
        let tokens = tokenize("KEYS *");

        assert_eq!(
            parse(&tokens).unwrap(),
            Command::Keys {
                pattern: "*".into()
            }
        );
    }

    #[test]
    fn parses_exit_case_insensitive() {
        let tokens = tokenize("exit");

        assert_eq!(parse(&tokens).unwrap(), Command::Exit);
    }

    #[test]
    fn rejects_empty_input() {
        let tokens = tokenize("");

        assert_eq!(parse(&tokens).unwrap_err(), ParseError::EmptyInput);
    }

    #[test]
    fn rejects_unknown_command() {
        let tokens = tokenize("GET key");

        assert_eq!(
            parse(&tokens).unwrap_err(),
            ParseError::UnknownCommand("GET".into())
        );
    }
}
