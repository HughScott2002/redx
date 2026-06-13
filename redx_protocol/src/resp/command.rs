use crate::parser::parse_parts;
use crate::{Command, ParseError, RespFrame};
use std::{borrow::Cow, fmt, str};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespCommand {
    name: Vec<u8>,
    arguments: Vec<Vec<u8>>,
}

impl RespCommand {
    pub fn from_frame(frame: &RespFrame) -> Result<Self, RespCommandError> {
        let elements = match frame {
            RespFrame::Array(elements) => elements,
            _ => return Err(RespCommandError::ExpectedArray),
        };

        let (name, arguments) = elements
            .split_first()
            .ok_or(RespCommandError::EmptyCommand)?;
        let name = bulk_string_bytes(name, 0)?.to_vec();
        let arguments = arguments
            .iter()
            .enumerate()
            .map(|(index, argument)| bulk_string_bytes(argument, index + 1).map(ToOwned::to_owned))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { name, arguments })
    }

    pub fn name(&self) -> &[u8] {
        &self.name
    }

    pub fn arguments(&self) -> &[Vec<u8>] {
        &self.arguments
    }

    pub fn name_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.name)
    }

    pub fn arguments_lossy(&self) -> Vec<String> {
        self.arguments
            .iter()
            .map(|argument| String::from_utf8_lossy(argument).into_owned())
            .collect()
    }

    pub fn to_command(&self) -> Result<Command, RespCommandParseError> {
        let mut parts = Vec::with_capacity(self.arguments.len() + 1);
        parts.push(
            str::from_utf8(&self.name)
                .map_err(|_| RespCommandParseError::InvalidUtf8 { index: 0 })?,
        );

        for (index, argument) in self.arguments.iter().enumerate() {
            parts.push(
                str::from_utf8(argument)
                    .map_err(|_| RespCommandParseError::InvalidUtf8 { index: index + 1 })?,
            );
        }

        parse_parts(parts).map_err(RespCommandParseError::Parse)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RespCommandError {
    ExpectedArray,
    EmptyCommand,
    NonBulkString { index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RespCommandParseError {
    InvalidUtf8 { index: usize },
    Parse(ParseError),
}

fn bulk_string_bytes(frame: &RespFrame, index: usize) -> Result<&[u8], RespCommandError> {
    match frame {
        RespFrame::BulkString(bytes) => Ok(bytes),
        _ => Err(RespCommandError::NonBulkString { index }),
    }
}

impl fmt::Display for RespCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RespCommandError::ExpectedArray => {
                write!(formatter, "RESP command frames must be arrays")
            }
            RespCommandError::EmptyCommand => {
                write!(formatter, "RESP command array cannot be empty")
            }
            RespCommandError::NonBulkString { index } => {
                write!(
                    formatter,
                    "RESP command element {index} must be a bulk string"
                )
            }
        }
    }
}

impl std::error::Error for RespCommandError {}

impl fmt::Display for RespCommandParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RespCommandParseError::InvalidUtf8 { index } => {
                write!(formatter, "RESP command element {index} is not valid UTF-8")
            }
            RespCommandParseError::Parse(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for RespCommandParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_known_command() {
        let frame = RespFrame::Array(vec![
            RespFrame::BulkString(b"PING".to_vec()),
            RespFrame::BulkString(b"ignored".to_vec()),
        ]);

        let command = RespCommand::from_frame(&frame).unwrap();

        assert_eq!(command.name(), b"PING");
        assert_eq!(command.arguments(), &[b"ignored".to_vec()]);
    }

    #[test]
    fn parses_resp_command_into_known_command() {
        let command = RespCommand::from_frame(&RespFrame::Array(vec![RespFrame::BulkString(
            b"PING".to_vec(),
        )]))
        .unwrap();

        assert_eq!(command.to_command().unwrap(), Command::Ping);
    }

    #[test]
    fn preserves_unknown_command_shape() {
        let command = RespCommand::from_frame(&RespFrame::Array(vec![
            RespFrame::BulkString(b"GET".to_vec()),
            RespFrame::BulkString(b"key".to_vec()),
        ]))
        .unwrap();

        assert_eq!(
            command.to_command().unwrap_err(),
            RespCommandParseError::Parse(ParseError::UnknownCommand("GET".into()))
        );
    }

    #[test]
    fn rejects_non_bulk_command_elements() {
        let error = RespCommand::from_frame(&RespFrame::Array(vec![
            RespFrame::BulkString(b"PING".to_vec()),
            RespFrame::SimpleString("oops".into()),
        ]))
        .unwrap_err();

        assert_eq!(error, RespCommandError::NonBulkString { index: 1 });
    }
}
