# pathbin v0.1.0

pathbin is a command-line tool for inspecting executable files available
through `PATH`. This first public release helps explain command precedence,
duplicates, shadowed binaries, invalid PATH entries, and broken symlinks.

## Install

Rust 1.85 or newer and Cargo are required.

```console
cargo install pathbin
```

## Commands

- `pathbin list` lists executable files found in PATH order.
- `pathbin where <name>` shows the active match for a command.
- `pathbin all <name>` shows every match and labels shadowed entries.
- `pathbin shadowed` lists binaries hidden by earlier PATH entries.
- `pathbin duplicates` reports command names with multiple matches.
- `pathbin broken` reports missing or invalid PATH entries and broken
  symlinks.
- `pathbin stats` summarizes PATH entries, executables, and duplicates.
- `pathbin doctor` reports common PATH problems. It exits with status 0 when
  no problem is found and status 1 when it reports a warning or error.

## Platform support

Linux and macOS are officially supported.

Windows support is experimental. The Windows implementation recognizes a
fixed set of executable extensions and is compile-checked, but does not yet
match normal Windows command resolution in every case.

## Known limitations

- Shell aliases, functions, built-ins, keywords, and command hashes are not
  discovered.
- PATH directories are scanned only at their top level.
- Unix executable detection checks permission bits and does not evaluate ACLs,
  mount options, or the current user's effective access.
- Windows lookup does not honor `PATHEXT`, requires filename extensions, and
  groups command names case-sensitively.
- Results describe the filesystem at scan time.

## Stability

v0.1.0 is an early pre-1.0 release. Command behavior, output formats, and exit
codes may change in breaking ways before 1.0. Pin the pathbin version and
verify behavior when upgrading scripts.
