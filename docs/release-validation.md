# Release validation

This document records checks that depend on external release services.

## Crate name

- Name: `pathbin`
- Registry: crates.io
- Checked: 2026-07-30
- Result: available for the first release

The crates.io API request for `pathbin` returned HTTP 404 with
`crate 'pathbin' does not exist`. The project will therefore keep `pathbin` as
its package and binary name for v0.1.0.

## Package contents

The v0.1.0 package is restricted to the manifest and lockfile, license,
README, Rust sources, and integration tests. Cargo also generates its
normalized manifest and VCS metadata during packaging.

Reviewed `cargo package --list` output:

```text
.cargo_vcs_info.json
Cargo.lock
Cargo.toml
Cargo.toml.orig
LICENSE
README.md
src/cli.rs
src/commands.rs
src/main.rs
src/model.rs
src/output.rs
src/scanner.rs
tests/cli_contract.rs
tests/path_edge_cases.rs
tests/path_precedence.rs
```
