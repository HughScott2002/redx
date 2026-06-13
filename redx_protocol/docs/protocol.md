# Redis Protocol Reference

This crate should model the Redis Serialization Protocol (RESP), the wire
format used between Redis clients and servers.

Start with RESP2. Redis connections default to RESP2, and RESP2 is enough for
basic command parsing, command encoding, pipelining, and replies.

RESP3 can be added later. It is mostly a RESP2 superset with more explicit
value types, attributes, and push messages.

## Project Context

The root crate is `redx_protocol`.

The current public API exports a whitespace tokenizer and parser:

```rust
pub use parser::{Command, ParseError, parse};
pub use tokenizer::{Token, tokenize};
```

That parser handles inline text like `KEYS *`. It is not a RESP wire parser.

Treat inline parsing as a CLI convenience layer. The network protocol should
parse bytes into RESP frames, then convert command arrays into `Command`.

## Scope

This document covers the protocol rules needed to implement a Redis-compatible
RESP parser and encoder.

It does not define Redis command behavior, storage semantics, Cluster bus
messages, Pub/Sub behavior, or server configuration.

## Wire Rules

RESP runs over a stream transport, normally TCP or a Unix socket.

The parser must be incremental. One read may contain a partial frame, exactly
one frame, or several pipelined frames.

The first byte of every frame selects the frame type.

Most frame headers end in CRLF: `\r\n`.

Bulk payload lengths are byte counts. The payload may contain any bytes,
including `\r`, `\n`, and `\r\n`.

After a bulk payload, the frame must contain a final CRLF.

Do not decode bulk strings as UTF-8 in the frame parser. Keep them as bytes.
Command parsing can decode command names and string arguments later.

## RESP2 Frames

| Type | Prefix | Shape | Rust value |
| --- | --- | --- | --- |
| Simple string | `+` | `+OK\r\n` | text without CR or LF |
| Simple error | `-` | `-ERR message\r\n` | error text |
| Integer | `:` | `:123\r\n` | signed 64-bit integer |
| Bulk string | `$` | `$5\r\nhello\r\n` | byte string |
| Null bulk string | `$` | `$-1\r\n` | null bulk value |
| Array | `*` | `*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n` | nested frames |
| Null array | `*` | `*-1\r\n` | null array value |

Recommended initial model:

```rust
pub enum RespFrame {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Vec<u8>),
    NullBulkString,
    Array(Vec<RespFrame>),
    NullArray,
}
```

Keep `NullBulkString` and `NullArray` separate if round-tripping matters.

If the command layer only needs semantic values, it can normalize both to a
single `Null` later.

## Command Encoding

Redis commands are sent as RESP arrays of bulk strings.

The first bulk string identifies the command. Some Redis commands use the
second bulk string as part of the command name, such as subcommands.

Keep the raw argument vector available to the command layer so it can interpret
subcommands without losing bytes.

Command names are case-insensitive. Arguments are binary-safe.

Example command:

```text
SET key value
```

RESP2 encoding:

```text
*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n
```

`GET key` is:

```text
*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n
```

`PING` is:

```text
*1\r\n$4\r\nPING\r\n
```

The encoder should calculate lengths from bytes, not characters.

## Inline Commands

Redis also accepts inline commands such as `PING\r\n` for telnet-style use.

Do not make inline commands the primary wire format.

If this crate keeps `tokenize("KEYS *")`, use it as an input convenience for
tests, REPLs, or CLI code. Convert the parsed command to RESP before sending it
over the network.

## Pipelining

A client may write multiple RESP commands without waiting for replies.

The server replies in the same order.

In RESP3, push frames can be interleaved with normal replies. Push frames do not
consume request reply slots, so the next non-push reply still belongs to the
oldest pending command.

The decoder should expose every complete frame found in the buffer and retain
the trailing incomplete bytes.

Example buffer with two commands:

```text
*1\r\n$4\r\nPING\r\n*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n
```

That buffer contains two complete command arrays.

## Parser Contract

A frame parser should return one of three outcomes:

```rust
enum DecodeResult {
    Complete { frame: RespFrame, consumed: usize },
    Incomplete,
    Invalid(RespError),
}
```

`Incomplete` means the current bytes are a valid prefix of a frame.

`Invalid` means more bytes cannot make the current frame valid.

After `Complete`, the caller drains `consumed` bytes and tries to decode the
next frame.

## Parsing Steps

Read the first byte and dispatch by prefix.

For simple strings and errors, scan to CRLF and decode the line as text.

For integers, scan to CRLF and parse a signed base-10 `i64`.

For bulk strings, parse the length line after `$`.

If the length is `-1`, return `NullBulkString`.

If the length is non-negative, require exactly that many payload bytes followed
by CRLF.

For arrays, parse the element count after `*`.

If the count is `-1`, return `NullArray`.

If the count is non-negative, parse exactly that many child frames in order.

Apply a maximum nesting depth to avoid stack overflow.

Apply a maximum frame size to avoid unbounded memory use.

## Validation Rules

Reject unknown frame prefixes.

Reject malformed integers and lengths.

Reject integer overflow.

Reject signs on bulk lengths except the exact null marker `-1`.

Reject signs on array lengths except the exact null marker `-1`.

Reject missing CRLF after headers and bulk payloads.

Reject simple strings and simple errors that contain bare CR or LF before the
terminating CRLF.

Reject arrays deeper than the configured nesting limit.

Reject frames larger than the configured byte limit.

## Errors

RESP simple errors are wire values, not parser failures.

`-ERR unknown command\r\n` should decode to `RespFrame::Error(...)`.

Parser failures are malformed protocol input, incomplete input, or configured
limit violations.

Redis error text usually starts with an uppercase prefix such as `ERR` or
`WRONGTYPE`. Treat that prefix as convention, not syntax.

## Null And Empty

Null and empty values are distinct.

Empty bulk string:

```text
$0\r\n\r\n
```

Null bulk string:

```text
$-1\r\n
```

Empty array:

```text
*0\r\n
```

Null array:

```text
*-1\r\n
```

Do not collapse these during frame parsing.

## RESP3 Notes

RESP3 is enabled by sending `HELLO 3` after connecting and receiving a
successful reply.

Redis remains in RESP2 by default unless the connection is upgraded.

If `HELLO` is unknown, rejected, or fails authentication, continue treating the
connection as RESP2.

Redis documents these RESP3 frame prefixes:

| Type | Prefix |
| --- | --- |
| Null | `_` |
| Boolean | `#` |
| Double | `,` |
| Big number | `(` |
| Bulk error | `!` |
| Verbatim string | `=` |
| Map | `%` |
| Attribute | `|` |
| Set | `~` |
| Push | `>` |

This is not a complete RESP3 implementation plan. RESP3 streamed strings and
streamed aggregates are out of scope for the first parser.

Attributes are metadata for the following value.

Push frames are out-of-band messages. A connection can receive them outside the
normal request-response flow.

Add RESP3 only after the RESP2 parser, encoder, and pipelining behavior are
well tested.

## Implementation Checklist

Implement a byte-oriented `RespFrame` parser before adding command semantics.

Add an encoder for arrays of bulk strings.

Parse pipelined buffers by repeatedly decoding frames until input is
incomplete.

Keep command parsing separate from frame parsing.

Support binary-safe bulk strings.

Preserve null bulk strings and null arrays.

Add configurable limits for frame size, bulk size, and nesting depth.

Test frames split at every byte boundary.

Test multiple frames in one buffer.

Test bulk payloads containing CRLF.

Test empty values separately from null values.

Test invalid prefixes, malformed lengths, missing CRLF, and over-limit frames.

## Sources

This reference is checked against the Redis protocol specification and the
RESP3 specification:

- <https://redis.io/docs/latest/develop/reference/protocol-spec/>
- <https://github.com/redis/redis-specifications/blob/master/protocol/RESP3.md>
