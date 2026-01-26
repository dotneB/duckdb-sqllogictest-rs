## Context
The upstream `sqllogictest` crate used by `duckdb-slt` does not support DuckDB's `require` directive. DuckDB's documentation defines `require` as a gate: when the required extension is not loaded, statements after the directive are skipped.

However, DuckDB’s Python sqllogictest runner behavior in practice is that `require` is effectively ignored (it does not itself fail), and subsequent statements are still executed; those statements then fail normally if they call functions from an extension that was not loaded.

This project also wants to avoid implementing a custom sqllogictest parser; therefore the solution needs to (1) keep using `sqllogictest` parsing/comparison logic, while (2) allowing `require` lines to exist in input files.

## Goals / Non-Goals
- Goals:
  - Support DuckDB-style `require <EXT>` directives.
  - Preserve stable line numbers for failure reporting.
  - Preserve include expansion behavior provided by `sqllogictest::parse_file` (if suites use `include`).
  - Reuse existing extension spec parsing (`name`, `name@repository`, `path/to.ext.duckdb_extension`).
- Non-Goals:
  - Full parity with every DuckDB `require` variant (e.g., `require vector_size N`) in the first iteration.
  - Adding new sqllogictest syntax beyond handling `require` lines for DuckDB compatibility.

## Decisions
- Decision: Implement `require` as a pre-run step plus a preprocessing step.
  - First pass: read the test file as text, extract `require` directives (in order), and produce a preprocessed script where each `require ...` line is rewritten as a comment line (same number of lines).
  - Second pass: run the preprocessed script through the existing `sqllogictest` runner.
  - Third pass: ensure user-facing diagnostics map back to the original file path (even if the runner parsed a preprocessed temp file).

## Implementation Sketch
- `require` recognition:
  - Match lines whose first non-whitespace token is `require`.
  - Parse the remainder of the line (trimmed) as an extension name.
  - Support `require quack` and `require 'quack'` (strip optional single quotes).
  - Multiple `require` lines are allowed; order matters.
- Preprocessing strategy:
  - Rewrite each `require ...` line to `# require ...` to make it valid for the upstream parser without changing line numbering.
  - Write the preprocessed content to a temporary file located alongside the original input file (same directory) so `include` expansion continues to resolve relative includes correctly.
  - Implement preprocessing in a small, extensible directive preprocessor module (currently only `require`).
- Runtime behavior:
  - Attempt to `LOAD` each required extension before executing any records.
  - Do not attempt `INSTALL` for `require` (avoids guessing install locations).
  - If any required extension fails to load:
    - Treat the `require` directive as a no-op and continue.
    - Execute the sqllogictest runner for that file; subsequent statements/queries will fail normally if they depend on the missing extension.
    - Optionally emit a concise note to stderr (exact output format to be chosen to minimize disruption).
- Diagnostics mapping:
  - The sqllogictest library will report locations using the preprocessed temp file path.
  - Map the temp filename back to the original input file path for display, while preserving the reported line number.
  - Ensure record indexing (`record:`) remains stable by re-parsing the same preprocessed content when computing record ids.

## Alternatives Considered
- Use `Runner::run_script_with_name(...)` with in-memory preprocessing:
  - Pros: locations can use the original filename directly.
  - Cons: upstream include expansion (`include`) is not performed for in-memory scripts.
- Rewrite `require` into a `statement ok` record that runs `INSTALL/LOAD`:
  - Pros: no pre-run extension step.
  - Cons: requires inserting extra lines and would shift reported line numbers.

## Risks / Trade-offs
- If suites depend on `require vector_size`, treating it as unsupported may cause remaining parse errors; this may need a follow-up change.
- Emitting a “require failed” note may require aligning with existing CLI output expectations and tests.

## Migration Plan
- No user action is required.
- Existing `--extensions` behavior remains; `require` adds per-file extension requirements.

## Open Questions
- Should the CLI print anything when `require` fails to load (silent vs warning line)?
No
- Should `require` accept only extension specs, or also support DuckDB’s `require vector_size N` variant?
