# Contributing to pathbin

Thank you for considering a contribution to pathbin. Bug reports,
documentation improvements, tests, and code changes are welcome.

## Before you start

- Search existing issues and pull requests before opening a new one.
- Open an issue before making a substantial change so the approach can be
  discussed.
- Keep each pull request focused on one problem.
- Never include secrets, credentials, or private data in an issue, commit, or
  pull request.

Security vulnerabilities require special handling. Follow
[SECURITY.md](SECURITY.md) instead of opening a public issue.

## Development environment

pathbin requires Rust 1.85 or newer. After cloning the repository, build and
test it with Cargo:

```console
cargo build
cargo test
```

The repository also provides a reproducible Nix development environment that
pins Rust 1.85.0. Prefix commands with `nix develop --command` when using it:

```console
nix develop --command cargo test
```

## Making a change

1. Create a branch from the latest `main`.
2. Make a focused change and add or update tests when behavior changes.
3. Update user-facing documentation when commands, output, or behavior change.
4. Run the required local checks.
5. Commit with a concise message that explains the change.
6. Open a pull request and link the related issue.

Follow the existing Rust style and prefer clear, small changes over unrelated
refactoring. Preserve compatibility with the minimum supported Rust version.

## Required local checks

At a minimum, run all of the following before opening or updating a pull
request:

```console
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

All commands must succeed. These checks match the repository's CI quality
gates. If you use Nix, run each command through `nix develop --command`.

## Issues

When reporting a bug, include:

- the pathbin version;
- the operating system and Rust version;
- the command that was run;
- the expected and actual behavior; and
- a minimal reproduction, when possible.

Remove usernames, filesystem details, tokens, and other sensitive information
from logs and command output.

Feature requests should explain the problem to solve, the proposed behavior,
and any alternatives considered.

## Pull requests

A pull request should:

- describe what changed and why;
- reference the related issue, using `Closes #<issue>` when appropriate;
- include tests for behavior changes;
- contain only relevant commits and files; and
- pass all required CI checks.

Review feedback may request changes for correctness, maintainability,
documentation, tests, or project scope.
