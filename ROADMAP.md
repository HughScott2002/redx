# Roadmap

Redx is a Redis-inspired Rust project. The roadmap is intentionally simple while the foundation is still being built.

## Near Term

- Expand tokenizer and parser coverage.
- Add tests for supported commands.
- Define the first storage API in `redx_core`.

## Next

- Implement basic key-value commands.
- Add a TCP server loop.
- Add integration tests that exercise the server through a client connection.

## Later

- Add persistence experiments.
- Add command documentation.
- Benchmark basic command paths.
