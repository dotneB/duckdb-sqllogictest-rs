## 1. Implementation
- [x] 1.1 Configure the `sqllogictest::Runner` to validate query column counts (count-only validator).
- [x] 1.2 Improve mismatch output for column-count mismatches to include expected vs actual column counts.
- [x] 1.3 Update/replace any fixtures that currently contain invalid column specs but are used as passing fixtures.
- [x] 1.4 Add regression tests that prove column-count mismatches fail and that diagnostics include the expected/actual column counts.

## 2. Validation
- [x] 2.1 `cargo fmt --check`
- [x] 2.2 `cargo clippy -- -D warnings`
- [x] 2.3 `cargo test`
