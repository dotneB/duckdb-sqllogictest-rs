## Context
`duckdb-slt` currently normalizes file arguments into absolute paths and buffers PASS/FAIL printing until after execution completes. Meanwhile, mismatch diagnostics are printed to stderr at the time they occur. This combination produces:

- machine-specific absolute paths in PASS/FAIL output and in failure report locations
- mismatch details appearing before the `FAIL <file>` line
- DuckDB error strings containing OS-specific prefixes/suffixes (not suitable for portable `expected_error` assertions)

## Goals / Non-Goals
- Goals:
- Produce stable, workdir-relative, CI-friendly output.
- Keep error normalization conservative and targeted at known-unstable patterns.
- Preserve exit code semantics.

- Non-Goals:
- Re-implement sqllogictest comparison logic.
- Introduce a new structured output format.

## Decisions
- Decision: Prefer workdir/current-dir relative paths in user-facing output.
  - Rationale: The suite is typically executed under a chosen `--workdir`; relative paths reduce noise and improve reproducibility.
  - Approach: Compute a display path by stripping the current working directory prefix when possible; otherwise display the original/absolute path.

- Decision: Print per-file PASS/FAIL/ERROR lines during execution.
  - Rationale: Enables emitting mismatch diagnostics directly under the corresponding FAIL line while still supporting `--no-fail-fast`.
  - Approach: For each file, print the status line to stdout, and for FAIL print the detailed mismatch report to stderr immediately after.

- Decision: Normalize select DuckDB error messages before sqllogictest compares them.
  - Rationale: Many DuckDB error strings include OS-specific details (e.g., Win32 messages, errno codes) that make portable tests fail.
  - Approach: Add a small normalization layer that maps known patterns (starting with file-open failures) to stable substrings (e.g., `Failed to open file`). Unknown errors pass through unchanged.
  - Trade-off: Over-normalization can mask meaningful differences; mitigate by limiting normalization to explicit patterns and adding unit tests.

## Risks / Trade-offs
- Relative path rendering depends on current directory state; mitigate by basing it on the post-`--workdir` process working directory and falling back to absolute paths.
- Output ordering across stdout/stderr can still interleave under some terminals; mitigate by printing the `FAIL` line before emitting the stderr report.

## Open Questions
- Which DuckDB error patterns beyond file-open failures should be normalized (if any)?
