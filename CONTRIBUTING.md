# Contributing

Thanks for taking a look at Redx.

## Project Status

Redx is an early learning project. Changes should stay small, easy to review, and aligned with the goal of building a Redis-inspired server in Rust.

## Development

Before opening a pull request, run:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test
```

## Pull Requests

- Keep each pull request focused on one change.
- Include tests for behavior changes when practical.
- Update documentation when behavior or commands change.
- Explain the motivation for the change clearly.

## Issues

Bug reports and feature ideas are welcome. Please include enough detail for someone else to reproduce the issue or understand the proposed improvement.
