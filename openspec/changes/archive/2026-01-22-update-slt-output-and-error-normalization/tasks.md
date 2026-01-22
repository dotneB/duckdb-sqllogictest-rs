## 1. Implementation
- [x] 1.1 Add a small path formatting helper for user-facing output (prefer workdir/current-dir relative; fall back to absolute).
- [x] 1.2 Change CLI output sequencing so each file prints its `PASS|FAIL|ERROR <path>` line as it is processed.
- [x] 1.3 Ensure mismatch failure reports are emitted immediately after their `FAIL <path>` line.
- [x] 1.4 Update the failure report renderer to use the same user-facing relative path format for `file:` and `at:` (when feasible).
- [x] 1.5 Add DuckDB error normalization for comparison (target: file open errors; keep rules conservative and opt-in per pattern).
- [x] 1.6 Add unit tests for error normalization (table-driven inputs/outputs).
- [x] 1.7 Add/extend CLI integration tests to assert:
- [x] 1.8 Validate output uses relative paths (no absolute prefixes) when run from a fixture directory.
- [x] 1.9 Validate failure report placement: `FAIL <file>` appears before `at:`/`sql:` lines.
- [x] 1.10 Run `cargo test` and ensure Windows + non-Windows behavior is stable.

## 2. Documentation
- [x] 2.1 Update `README.md` if it documents example output that now changes (paths/order).

## 3. Validation
- [x] 3.1 Run `openspec validate update-slt-output-and-error-normalization --strict --no-interactive`.
