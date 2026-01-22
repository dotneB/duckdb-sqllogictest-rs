## 1. Implementation
- [x] 1.1 Add a real `README.md` and document the CLI at a high level (purpose, basic usage, exit codes)
- [x] 1.2 Implement `clap` CLI surface and defaults (flags + `<FILES...>` positional)
- [x] 1.2.1 Ensure `--help` and `--version` exit with code `0`
- [x] 1.2.2 Ensure clap usage/parse errors exit with code `1` (do not use `2`)
- [x] 1.3 Implement `--workdir` behavior for file resolution (at minimum: chdir before running)
- [x] 1.4 Implement extension actions: `--install` and `--load` in the documented order
- [x] 1.5 Implement output formats: `--format text|json` (ensure JSON mode keeps stdout machine-readable)
- [x] 1.6 Implement fail-fast behavior: `--fail-fast` / `--no-fail-fast`
- [x] 1.7 Implement exit-code mapping so that mismatches=2, runtime errors=1, success=0

## 2. Validation
- [x] 2.1 Add integration tests for exit code contract (pass fixture => 0, mismatch fixture => 2, invalid path/parse error => 1)
- [x] 2.2 Add tests for `--format json` output shape (smoke test: valid JSON with status + counts)
- [x] 2.3 Add a test that `--help` exits 0 and does not print a clap error
- [x] 2.4 Add an integration test for `--no-fail-fast` (a mismatch in the first file does not prevent attempting the second)
- [x] 2.5 Add an integration test that `--install` actions run before `--load` actions
- [x] 2.6 Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test`

## 3. Documentation
- [x] 3.1 Add `duckdb-slt --help` and `duckdb-slt --version` examples to `README.md`
- [x] 3.2 Document the stable exit codes in `README.md`

## 4. Revision: Extensions Flag Simplification
- [x] 4.1 Replace `--install` and `--load` with `--extensions <EXT>` in the clap CLI
- [x] 4.2 Define `--extensions` behavior as `INSTALL` then `LOAD` for each entry, in CLI order
- [x] 4.3 Update `README.md` examples to use `--extensions`
- [x] 4.4 Update integration tests to use `--extensions` and assert ordering (`INSTALL a; LOAD a; INSTALL b; LOAD b`)
- [x] 4.5 Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test`

## 5. Revision: Short Flags
- [x] 5.1 Add `-e` as a short form for `--extensions`
- [x] 5.2 Add `-w` as a short form for `--workdir`
- [x] 5.3 Update `README.md` examples to mention short forms (optional)
- [x] 5.4 Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test`
