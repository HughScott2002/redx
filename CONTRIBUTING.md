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

## Branching Model

Redx uses a Git Flow Lite workflow.

- `main` is the release branch. It should represent production-ready history.
- `develop` is the integration branch for upcoming work.
- `feature/*` branches start from `develop` and merge back into `develop`.
- `hotfix/*` branches start from `main` and merge back into `main`.
- After a hotfix lands on `main`, merge `main` back into `develop`.

Do not push directly to `main` or `develop`. Use pull requests for both branches.

## Backlog and Planning

GitHub Issues are the canonical backlog for Redx. Use issues for bugs, feature ideas, documentation work, and planned implementation tasks.

- Use milestones for release targets, such as `v0.2.0`.
- Use labels to describe area and type, such as `bug`, `enhancement`, `docs`, `core`, `protocol`, and `server`.
- Link pull requests to issues with `Closes #N` when the pull request completes the issue.
- Keep `ROADMAP.md` focused on high-level direction instead of duplicating the issue list.
- Do not add a committed `TODO.md` for tracked project work.

## Pull Requests

- Keep each pull request focused on one change.
- Target feature work at `develop`.
- Target release and hotfix work at `main`.
- Require passing CI before merge.
- Require at least one approval before merge.
- Include tests for behavior changes when practical.
- Update documentation when behavior or commands change.
- Update `CHANGELOG.md` for user-visible changes.
- Explain the motivation for the change clearly.
- Link related issues with `Closes #N` when the pull request completes the issue.

Protected branches should block force pushes and branch deletion.

## Versioning

Redx uses manual SemVer.

- Patch versions are for bug fixes, docs release hygiene, and compatible fixes.
- Minor versions are for new commands, protocol features, and server capabilities.
- Major versions are for breaking public API or wire behavior changes.

Keep workspace crate versions aligned across `redx_core`, `redx_protocol`, and `redx_server`.

Do not bump versions in normal feature pull requests. Bump versions in the release pull request.

## Changelog

`CHANGELOG.md` is the release audit trail.

Keep an `Unreleased` section with these headings when needed:

- `Added`
- `Changed`
- `Fixed`
- `Security`

Move entries from `Unreleased` into the release version section during the release pull request.

## Release Flow

Use this flow for normal releases:

1. Finish feature work in `develop`.
2. Open a release pull request from `develop` into `main`.
3. Bump workspace crate versions.
4. Move `CHANGELOG.md` entries from `Unreleased` to `X.Y.Z - YYYY-MM-DD`.
5. Run formatting, Clippy, and tests.
6. Merge the pull request into `main`.
7. Create an annotated tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`.
8. Push the tag.
9. Publish a GitHub Release named `vX.Y.Z` using the changelog notes.
10. Merge `main` back into `develop`.

## Hotfix Flow

Use this flow when production history needs an urgent fix:

1. Branch from `main` with `hotfix/vX.Y.Z-short-name`.
2. Make the fix, add tests, update docs if needed, and update `CHANGELOG.md`.
3. Bump workspace crate versions for the hotfix release.
4. Open a pull request into `main`.
5. Merge after CI passes and review approval is complete.
6. Tag and publish the GitHub Release.
7. Merge `main` back into `develop`.

## Issues

Bug reports and feature ideas are welcome. Please include enough detail for someone else to reproduce the issue or understand the proposed improvement.
