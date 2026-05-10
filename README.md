# pathbin
`pathbin` is a CLI tool for inspecting executable binaries available through the `PATH` environment variable.

It extracts executable binaries from `PATH` and helps you understand command resolution, duplicates, shadowed binaries, and broken paths.

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

## Concept
pathbin scans each directory listed in the PATH environment variable in order.

For each entry, it checks executable files and groups them by command name.
This makes it possible to detect which binary is actually executed first, which binaries are shadowed, and whether there are duplicate or broken entries.