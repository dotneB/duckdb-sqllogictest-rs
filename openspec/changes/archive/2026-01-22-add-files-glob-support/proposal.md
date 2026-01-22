# Change: Add FILES Glob Support

## Why
On some platforms (notably Windows), shells often do not expand glob patterns for positional arguments. Supporting glob expansion inside `duckdb-slt` makes it easier to run suites like `tests/**/*.slt` consistently across environments.

## What Changes
- Extend `<FILES...>` to accept glob patterns in addition to literal file paths.
- Expand glob patterns relative to the effective working directory (current directory by default; `--workdir` when set).
- Define deterministic expansion order so CI runs are stable.

## Impact
- Affected specs:
  - `openspec/specs/duckdb-slt-cli/spec.md`
- Affected code (expected):
  - `src/main.rs`
  - `tests/cli.rs`
