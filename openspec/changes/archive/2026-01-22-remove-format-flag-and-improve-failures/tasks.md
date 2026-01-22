## 1. Implementation
- [x] 1.1 Use `sqllogictest::TestErrorKind` + `TestError::location()` for SQL/expected/actual + file/line, and (on failure) parse the failing script to derive record index and optional label/name.
- [x] 1.2 Remove the `--format` flag and JSON summary output; update CLI, output behavior, and existing integration tests.
- [x] 1.3 Implement structured mismatch diagnostics that print:
      - file path
      - record index and record name (when available)
      - SQL snippet
      - expected output
      - actual output
- [x] 1.4 Ensure mismatch outcomes reliably map to process exit code `2` (including `--no-fail-fast` runs).
- [x] 1.5 Add a failing integration fixture under `tests/fixtures/` with a deterministic mismatch and an unambiguous SQL statement.
- [x] 1.6 Extend `tests/cli.rs` to assert, for the failing fixture:
      - exit code is `2`
      - output (stderr) contains the record identifier and the SQL snippet.

## 2. Validation
- [x] 2.1 Run `cargo test`.
- [x] 2.2 Run `cargo fmt` (and `cargo fmt --check` if CI requires it).
- [x] 2.3 Run `cargo clippy -- -D warnings`.
