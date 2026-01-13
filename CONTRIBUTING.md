# Contributing to Planit

Thanks for your interest in improving Planit. We welcome contributions from
anyone who wants to expand features, improve quality, or refine the UX.

Please read this guide before starting work.

## Getting Started

- Install Rust (stable): https://rustup.rs/
- Build: `cargo build`
- Run tests: `cargo test -q`

## Coding Standards

- Use `cargo fmt` before submitting changes.
- Prefer clear, descriptive names and error messages.
- Avoid `unsafe` unless there is no reasonable alternative.
- Avoid introducing `Arc` unless shared ownership is required across threads or
  for long-lived shared state; prefer simpler ownership when possible.
- Keep changes focused and scoped to one concern.

## Project Conventions

- CLI input flows live under `src/prompter`.
- Commands and manual pages live under `src/command`.
- Entity parsing is driven by specs in `src/command/entity_spec`.
- When you add or change commands, update the manual (`ManualCatalog`) and
  `README.md` where relevant.
- When you add or change logging behavior, keep file logging optional and
  respect the `FILE_LOGGING_ENABLED` config.
- Keep code coverage healthy. We use `cargo llvm-cov` to measure coverage, and
  PRs may be gated by a minimum coverage threshold.

## Reporting Bugs

Open a GitHub issue with:
- Steps to reproduce
- Expected vs actual behavior
- Any relevant logs or screenshots
- OS and Rust version

## Submitting Pull Requests

1. Open an issue or feature request first, so we can align on scope.
2. Create a branch from `main`.
3. Make your changes with tests where appropriate.
4. Run `cargo test -q` and ensure it passes.
5. Open a PR and link the related issue.
