# Change: Remove --format flag and improve mismatch failure reporting

## Why
The current CLI includes a `--format` flag and JSON summary output, but this duplicates capabilities already available via the sqllogictest runner (human-readable diagnostics and optional structured reporting such as JUnit). Separately, when a record mismatches, failure output should be immediately actionable (file + record identifier + SQL + expected vs actual), and CI depends on stable mismatch exit codes.

## What Changes
- Remove the `--format` flag and JSON summary output; the CLI emits human-readable output only.
- Ensure expectation mismatches reliably return exit code `2`.
- Improve mismatch diagnostics to include: file path, record identifier (index and name when available), SQL snippet, and expected vs actual output.
- Add an integration test fixture that intentionally fails and assert both the exit code and the presence of key failure context in output.

## Impact
- Affected specs: `openspec/specs/duckdb-slt-cli/spec.md`
- Affected code: `src/main.rs` (CLI flags, output, failure formatting + exit-code selection), `tests/cli.rs`, `tests/fixtures/*`
