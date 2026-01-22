# Change: Add Stable CLI Contract (Flags + Exit Codes)

## Why
CI and developer workflows need a stable, documented CLI surface and a predictable exit-code contract so that failures can be detected and categorized reliably (pass vs test mismatch vs runtime error).

## What Changes
- Define the public CLI surface for `duckdb-slt` using `clap`.
- Define stable exit codes: `0` pass, `2` test failure/mismatch, `1` runtime error (including invalid input / CLI usage errors).
- Ensure `--help` / `--version` exit successfully (exit code `0`) and do not conflict with the mismatch contract.
- (Revision) replace `--install` / `--load` with a single repeatable flag `--extensions <EXT>`.
- Add `--help` and `--version` examples to `README.md`.

## Impact
- Affected specs: new capability spec `duckdb-slt-cli`.
- Affected code: `src/main.rs` (and new runner/output modules as needed).
- Affected docs: `README.md` (new file; currently absent).
