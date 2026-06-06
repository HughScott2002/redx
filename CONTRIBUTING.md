# Contributing

Thanks for taking a look at Redx.

## Project Status

Redx is an early learning project. Changes should stay small, easy to review, and aligned with the goal of building a Redis-inspired server in Rust.

## Development

Before opening a pull request, run:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test
```

## Backlog and Planning

GitHub Issues are the canonical backlog for Redx. Use issues for bugs, feature ideas, documentation work, and planned implementation tasks.

- Use milestones for release targets, such as `v0.2.0`.
- Use labels to describe area and type, such as `bug`, `enhancement`, `docs`, `core`, `protocol`, and `server`.
- Link pull requests to issues with `Closes #N` when the pull request completes the issue.
- Keep `ROADMAP.md` focused on high-level direction instead of duplicating the issue list.
- Do not add a committed `TODO.md` for tracked project work.

## Pull Requests

- Keep each pull request focused on one change.
- Include tests for behavior changes when practical.
- Update documentation when behavior or commands change.
- Explain the motivation for the change clearly.

## Issues

Bug reports and feature ideas are welcome. Please include enough detail for someone else to reproduce the issue or understand the proposed improvement.
