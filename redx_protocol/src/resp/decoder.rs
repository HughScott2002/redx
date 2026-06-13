use std::fmt;

const MAX_NESTING_DEPTH: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RespFrame {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Vec<u8>),
    NullBulkString,
    Array(Vec<RespFrame>),
    NullArray,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeResult {
    Complete { frame: RespFrame, consumed: usize },
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RespError {
    InvalidPrefix(u8),
    MissingCrlf,
    InvalidSimpleString,
    InvalidErrorString,
    InvalidInteger,
    InvalidBulkLength,
    InvalidArrayLength,
    NestingLimitExceeded { limit: usize },
}

enum FrameStatus {
    Complete { frame: RespFrame, next_index: usize },
    Incomplete,
}

pub fn decode(input: &[u8]) -> Result<DecodeResult, RespError> {
    match decode_frame(input, 0, 0)? {
        FrameStatus::Complete { frame, next_index } => Ok(DecodeResult::Complete {
            frame,
            consumed: next_index,
        }),
        FrameStatus::Incomplete => Ok(DecodeResult::Incomplete),
    }
}

fn decode_frame(input: &[u8], start: usize, depth: usize) -> Result<FrameStatus, RespError> {
    if start >= input.len() {
        return Ok(FrameStatus::Incomplete);
    }

    if depth > MAX_NESTING_DEPTH {
        return Err(RespError::NestingLimitExceeded {
            limit: MAX_NESTING_DEPTH,
        });
    }

    match input[start] {
        b'+' => decode_text_frame(
            input,
            start,
            RespFrame::SimpleString,
            RespError::InvalidSimpleString,
        ),
        b'-' => decode_text_frame(
            input,
            start,
            RespFrame::Error,
            RespError::InvalidErrorString,
        ),
        b':' => decode_integer_frame(input, start),
        b'$' => decode_bulk_string_frame(input, start),
        b'*' => decode_array_frame(input, start, depth + 1),
        byte => Err(RespError::InvalidPrefix(byte)),
    }
}

fn decode_text_frame(
    input: &[u8],
    start: usize,
    build: impl FnOnce(String) -> RespFrame,
    invalid_error: RespError,
) -> Result<FrameStatus, RespError> {
    let Some((line, next_index)) = read_line(input, start + 1)? else {
        return Ok(FrameStatus::Incomplete);
    };

    if line.iter().any(|byte| matches!(byte, b'\r' | b'\n')) {
        return Err(invalid_error);
    }

    let text = String::from_utf8(line.to_vec()).map_err(|_| invalid_error)?;
    Ok(FrameStatus::Complete {
        frame: build(text),
        next_index,
    })
}

fn decode_integer_frame(input: &[u8], start: usize) -> Result<FrameStatus, RespError> {
    let Some((line, next_index)) = read_line(input, start + 1)? else {
        return Ok(FrameStatus::Incomplete);
    };

    let value = parse_i64(line).map_err(|_| RespError::InvalidInteger)?;

    Ok(FrameStatus::Complete {
        frame: RespFrame::Integer(value),
        next_index,
    })
}

fn decode_bulk_string_frame(input: &[u8], start: usize) -> Result<FrameStatus, RespError> {
    let Some((line, next_index)) = read_line(input, start + 1)? else {
        return Ok(FrameStatus::Incomplete);
    };

    let length = parse_length(line).map_err(|_| RespError::InvalidBulkLength)?;

    if length == -1 {
        return Ok(FrameStatus::Complete {
            frame: RespFrame::NullBulkString,
            next_index,
        });
    }

    let payload_length = usize::try_from(length).map_err(|_| RespError::InvalidBulkLength)?;
    let payload_end = next_index
        .checked_add(payload_length)
        .ok_or(RespError::InvalidBulkLength)?;

    if payload_end + 2 > input.len() {
        return Ok(FrameStatus::Incomplete);
    }

    if &input[payload_end..payload_end + 2] != b"\r\n" {
        return Err(RespError::MissingCrlf);
    }

    Ok(FrameStatus::Complete {
        frame: RespFrame::BulkString(input[next_index..payload_end].to_vec()),
        next_index: payload_end + 2,
    })
}

fn decode_array_frame(input: &[u8], start: usize, depth: usize) -> Result<FrameStatus, RespError> {
    let Some((line, mut next_index)) = read_line(input, start + 1)? else {
        return Ok(FrameStatus::Incomplete);
    };

    let length = parse_length(line).map_err(|_| RespError::InvalidArrayLength)?;

    if length == -1 {
        return Ok(FrameStatus::Complete {
            frame: RespFrame::NullArray,
            next_index,
        });
    }

    let length = usize::try_from(length).map_err(|_| RespError::InvalidArrayLength)?;
    let mut elements = Vec::with_capacity(length);

    for _ in 0..length {
        match decode_frame(input, next_index, depth)? {
            FrameStatus::Complete {
                frame,
                next_index: consumed,
            } => {
                elements.push(frame);
                next_index = consumed;
            }
            FrameStatus::Incomplete => return Ok(FrameStatus::Incomplete),
        }
    }

    Ok(FrameStatus::Complete {
        frame: RespFrame::Array(elements),
        next_index,
    })
}

fn read_line(input: &[u8], start: usize) -> Result<Option<(&[u8], usize)>, RespError> {
    let mut index = start;

    while index < input.len() {
        if input[index] == b'\r' {
            if index + 1 >= input.len() {
                return Ok(None);
            }

            if input[index + 1] == b'\n' {
                return Ok(Some((&input[start..index], index + 2)));
            }

            return Err(RespError::MissingCrlf);
        }

        index += 1;
    }

    Ok(None)
}

fn parse_i64(line: &[u8]) -> Result<i64, ()> {
    let line = std::str::from_utf8(line).map_err(|_| ())?;
    line.parse::<i64>().map_err(|_| ())
}

fn parse_length(line: &[u8]) -> Result<i64, ()> {
    let line = std::str::from_utf8(line).map_err(|_| ())?;

    if line == "-1" {
        return Ok(-1);
    }

    if line.starts_with('-') || line.starts_with('+') {
        return Err(());
    }

    line.parse::<i64>().map_err(|_| ())
}

impl fmt::Display for RespError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RespError::InvalidPrefix(byte) => {
                write!(formatter, "invalid RESP frame prefix: 0x{byte:02x}")
            }
            RespError::MissingCrlf => write!(formatter, "missing CRLF terminator"),
            RespError::InvalidSimpleString => write!(formatter, "invalid RESP simple string"),
            RespError::InvalidErrorString => write!(formatter, "invalid RESP error string"),
            RespError::InvalidInteger => write!(formatter, "invalid RESP integer"),
            RespError::InvalidBulkLength => write!(formatter, "invalid RESP bulk string length"),
            RespError::InvalidArrayLength => write!(formatter, "invalid RESP array length"),
            RespError::NestingLimitExceeded { limit } => {
                write!(formatter, "RESP nesting limit exceeded ({limit})")
            }
        }
    }
}

impl std::error::Error for RespError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_complete_ping_command() {
        let result = decode(b"*1\r\n$4\r\nPING\r\n").unwrap();

        assert_eq!(
            result,
            DecodeResult::Complete {
                frame: RespFrame::Array(vec![RespFrame::BulkString(b"PING".to_vec())]),
                consumed: 14,
            }
        );
    }

    #[test]
    fn reports_incomplete_frame_until_payload_arrives() {
        assert_eq!(
            decode(b"*2\r\n$4\r\nKEYS\r\n$").unwrap(),
            DecodeResult::Incomplete
        );

        assert_eq!(
            decode(b"*2\r\n$4\r\nKEYS\r\n$1\r\n*\r\n").unwrap(),
            DecodeResult::Complete {
                frame: RespFrame::Array(vec![
                    RespFrame::BulkString(b"KEYS".to_vec()),
                    RespFrame::BulkString(b"*".to_vec()),
                ]),
                consumed: 21,
            }
        );
    }

    #[test]
    fn decodes_pipelined_commands_one_frame_at_a_time() {
        let input = b"*1\r\n$4\r\nPING\r\n*2\r\n$4\r\nKEYS\r\n$1\r\n*\r\n";

        let first = decode(input).unwrap();
        let DecodeResult::Complete {
            frame: first_frame,
            consumed,
        } = first
        else {
            panic!("expected a complete frame");
        };

        assert_eq!(
            first_frame,
            RespFrame::Array(vec![RespFrame::BulkString(b"PING".to_vec())])
        );

        assert_eq!(
            decode(&input[consumed..]).unwrap(),
            DecodeResult::Complete {
                frame: RespFrame::Array(vec![
                    RespFrame::BulkString(b"KEYS".to_vec()),
                    RespFrame::BulkString(b"*".to_vec()),
                ]),
                consumed: input.len() - consumed,
            }
        );
    }

    #[test]
    fn rejects_invalid_prefix() {
        assert_eq!(
            decode(b"!1\r\n").unwrap_err(),
            RespError::InvalidPrefix(b'!')
        );
    }

    #[test]
    fn rejects_invalid_bulk_length() {
        assert_eq!(
            decode(b"$x\r\nhello\r\n").unwrap_err(),
            RespError::InvalidBulkLength
        );
    }

    #[test]
    fn rejects_missing_bulk_payload_crlf() {
        assert_eq!(decode(b"$3\r\nabc!!").unwrap_err(), RespError::MissingCrlf);
    }
}
