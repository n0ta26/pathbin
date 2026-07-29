# pathbin
`pathbin` is a CLI tool for inspecting executable binaries available through the `PATH` environment variable.

It extracts executable binaries from `PATH` and helps you understand command resolution, duplicates, shadowed binaries, and broken paths.

## Installation

Install Rust 1.85 or newer, including Cargo, and then install pathbin from
crates.io:

```console
cargo install pathbin
```

## Minimum supported Rust version

pathbin v0.1.1 supports Rust 1.85 and newer.

The reproducible development environment pins Rust 1.85.0. Run project commands through Nix:

```console
nix develop --command cargo build
```

## Stability

v0.1.1 is an early, pre-1.0 release. Command behavior, human-readable output
formats, and exit-code semantics may change in breaking ways before 1.0.
Scripts should pin the pathbin version and verify behavior when upgrading.

## Platform support

pathbin v0.1.1 officially supports:

- Linux
- macOS

Windows support is experimental in v0.1.1. Other operating systems are
untested and not officially supported.

On Windows, pathbin currently:

- recognizes `.exe`, `.cmd`, `.bat`, `.com`, and `.ps1` files regardless of
  extension case;
- ignores the user's `PATHEXT` value;
- requires lookup names to include the filename extension; and
- groups command names case-sensitively, unlike normal Windows command
  resolution.

The Windows code is compile-checked for `x86_64-pc-windows-gnu`, but the
release is not yet covered by native Windows integration tests.

## Known limitations

pathbin inspects filesystem entries in `PATH`; it does not fully emulate a
shell's command-resolution rules.

- Shell aliases, functions, built-ins, keywords, and command hashes are not
  discovered.
- Each `PATH` directory is scanned only at its top level. Subdirectories are
  not searched recursively.
- On Unix, a regular file is considered executable when any executable
  permission bit is set. ACLs, mount options, and the current user's effective
  access are not evaluated.
- On Windows, executable detection uses a fixed set of recognized extensions
  and does not yet reproduce all `PATHEXT` and case-insensitive lookup rules.
- Output describes the filesystem at scan time and can become stale if files
  or `PATH` change while the command is running.

## Features
- List executable binaries in `PATH`
- Show where a command is located
- Show all binaries with the same name
- Detect shadowed binaries hidden by `PATH` priority
- Detect duplicate command names
- Detect broken symlinks and missing `PATH` entries
- Show basic statistics about binaries in `PATH`
- Diagnose common `PATH` problems

## Usage
`pathbin <COMMAND>`

## Commands
- list        List executable binaries in PATH
- where       Show where a command is located
- all         Show all matching binaries with the same name
- shadowed    Show binaries hidden by PATH priority
- duplicates  Show duplicate binary names
- broken      Show broken symlinks and missing PATH entries
- stats       Show PATH binary statistics
- doctor      Diagnose PATH-related problems

## Examples
List binaries in PATH.
`pathbin list`

Show where a command is located.
`pathbin where cargo`

Show all binaries with the same name.
`pathbin all python`

Show shadowed binaries.
`pathbin shadowed`

Show duplicate binaries.
`pathbin duplicates`

Check broken PATH entries or symlinks.
`pathbin broken`

Show PATH statistics.
`pathbin stats`

Diagnose PATH problems.
`pathbin doctor`

### Doctor exit codes

`pathbin doctor` writes its diagnostic report to standard output. It exits
with status 0 when no problems are detected and status 1 when it reports any
warning or error. This makes the command suitable for both interactive use
and automated checks.

## Concept
pathbin scans each directory listed in the PATH environment variable in order.

For each entry, it checks executable files and groups them by command name.
This makes it possible to detect which binary is actually executed first, which binaries are shadowed, and whether there are duplicate or broken entries.
