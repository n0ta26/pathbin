# pathbin v0.1.2

This release adds Homebrew-focused command discovery, makes terminal output
safe for untrusted filesystem names, and strengthens release automation.

## Release highlights

- A new `--homebrew` option restricts every command to Homebrew's common
  `bin` and `sbin` directories. It honors an absolute `HOMEBREW_PREFIX` and
  otherwise uses the platform's official default prefix.
- Command names and paths containing control characters are escaped before
  display. On Unix, non-UTF-8 names remain distinct and can be looked up using
  their original OS-native bytes.
- Repeated `PATH` directories no longer cause an identical executable to be
  reported as a duplicate, while entry-level statistics still reflect the
  original `PATH` structure.
- Abnormal inputs and filesystem states now have explicit integration-test
  contracts across every public command.
- Required CI, immutable GitHub Actions pins, automated dependency updates,
  and main-branch ancestry validation harden the build and release pipeline.
