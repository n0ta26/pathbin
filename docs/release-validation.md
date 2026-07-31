# Release validation

This document records checks that depend on external release services.

## Crate name

- Name: `pathbin`
- Registry: crates.io
- Checked: 2026-07-30
- Result: available for the first release

The crates.io API request for `pathbin` returned HTTP 404 with
`crate 'pathbin' does not exist`. The project will therefore keep `pathbin` as
its package and binary name for v0.1.2.

## Package contents

The v0.1.2 package is restricted to the manifest and lockfile, license,
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
tests/homebrew_source.rs
tests/path_edge_cases.rs
tests/path_precedence.rs
```

## Publish dry run

On 2026-08-01, from a clean Git working tree, the following command completed
successfully with Rust and Cargo 1.85.0:

```console
nix develop --command cargo publish --dry-run
```

Cargo packaged the 16 reviewed files, compiled the packaged crate
successfully, and stopped before upload as required by dry-run mode. The only
warning was the expected `aborting upload due to dry run`; there were no
package metadata, content, build, or verification warnings.

## Release tag ancestry

The CD workflow accepts a release tag only when its peeled commit is reachable
from `origin/main`. This prevents a tag created from an unmerged branch or
another arbitrary commit from publishing release artifacts.

The validation fetches `main` directly from `origin` and uses
`git merge-base --is-ancestor` so annotated and signed tags are checked by their
target commit rather than by the tag object itself.
