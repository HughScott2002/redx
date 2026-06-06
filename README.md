# Redx

Redx is a Redis-inspired server implementation written in Rust.

This project is early and experimental. The goal is to learn systems programming, protocol parsing, command handling, and async server design by building a small Redis-like database from the ground up.

## Status

Redx is not production-ready. It currently focuses on the project structure and protocol parsing foundation.

## Workspace

- `redx_core`: shared domain logic for the database engine.
- `redx_protocol`: tokenizer and parser code for Redis-like commands.
- `redx_server`: command-line server shell and runtime entrypoint.

## Getting Started

Install Rust with [rustup](https://rustup.rs/), then clone the repository and run:

```sh
cargo run -p redx_server
```

Run the test suite with:

```sh
cargo test
```

Check formatting and linting with:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets
```

## Roadmap

- Build out command parsing for common Redis commands.
- Add an in-memory key-value storage engine.
- Implement TCP server support.
- Add integration tests for client/server behavior.
- Document protocol behavior and supported commands.

## Contributing

Contributions, issues, and suggestions are welcome. See `CONTRIBUTING.md` for the basic workflow.

## License

Redx is licensed under the MIT License. See `LICENSE` for details.
