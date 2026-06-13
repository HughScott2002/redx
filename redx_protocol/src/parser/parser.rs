use crate::Token;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Ping,
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
    parse_parts(tokens.iter().map(Token::value))
}

pub(crate) fn parse_parts<'a>(
    parts: impl IntoIterator<Item = &'a str>,
) -> Result<Command, ParseError> {
    let parts: Vec<&str> = parts.into_iter().collect();
    let command = parts.first().copied().ok_or(ParseError::EmptyInput)?;

    if command.eq_ignore_ascii_case("exit") {
        return parse_exit(&parts);
    }

    if command.eq_ignore_ascii_case("PING") {
        return parse_ping(&parts);
    }

    if command.eq_ignore_ascii_case("KEYS") {
        return parse_keys(&parts);
    }

    Err(ParseError::UnknownCommand(command.to_string()))
}

fn parse_exit(parts: &[&str]) -> Result<Command, ParseError> {
    if parts.len() == 1 {
        Ok(Command::Exit)
    } else {
        Err(ParseError::InvalidArity {
            command: "exit".into(),
            expected: 0,
            got: parts.len() - 1,
        })
    }
}

fn parse_ping(parts: &[&str]) -> Result<Command, ParseError> {
    if parts.len() == 1 {
        Ok(Command::Ping)
    } else {
        Err(ParseError::InvalidArity {
            command: "PING".into(),
            expected: 0,
            got: parts.len() - 1,
        })
    }
}

fn parse_keys(parts: &[&str]) -> Result<Command, ParseError> {
    if parts.len() != 2 {
        return Err(ParseError::InvalidArity {
            command: "KEYS".into(),
            expected: 1,
            got: parts.len().saturating_sub(1),
        });
    }

    Ok(Command::Keys {
        pattern: parts[1].to_string(),
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
    fn parses_ping_case_insensitive() {
        let tokens = tokenize("ping");

        assert_eq!(parse(&tokens).unwrap(), Command::Ping);
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
