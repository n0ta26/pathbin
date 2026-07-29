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
