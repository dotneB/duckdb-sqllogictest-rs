## Context
This repository currently contains a placeholder `duckdb-slt` binary that only parses a single optional file argument. The project intent (see `openspec/project.md`) calls out a CI-friendly CLI runner, including extension handling and an explicit exit code contract.

The change defines the first stable CLI surface that downstream tooling can depend on.

## Goals / Non-Goals
- Goals:
- Provide a stable `clap`-based CLI surface with the requested flags.
- Make exit codes unambiguous and stable for CI.
- Provide a machine-readable JSON output mode for tooling.
- Non-Goals:
- Full parity with DuckDB's internal harness features (parallelism, environment matrices).
- Guaranteeing a final JSON schema for per-record details beyond a minimal summary (can evolve later under explicit versioning).

## Decisions
- Decision: Reserve exit code `2` exclusively for sqllogictest mismatches.
- Rationale: Many CI systems interpret `1` as generic failure; using `2` specifically for mismatches allows automated retry/triage.

- Decision: Treat CLI parse/usage errors as runtime errors (exit code `1`).
- Rationale: `clap` defaults to exit code `2` for usage errors; that would collide with the mismatch code. Implementation should override clap's default exit code behavior so that `2` remains reserved for test failures.

- Decision: `--help` and `--version` exit successfully (exit code `0`).
- Rationale: Help/version are not failures and must be safe to call in CI discovery scripts.

- Decision: Default output format is `text`; JSON mode emits a single JSON document to stdout.
- Rationale: Keep the default experience human-friendly while allowing CI tooling to ingest structured output.

- Decision: Default `fail-fast` to enabled.
- Rationale: Minimal, CI-friendly behavior; users can opt out with `--no-fail-fast` to collect all mismatches.

- Decision: `--workdir` is applied before any file/extension path resolution.
- Rationale: Keeps relative paths reproducible across environments.

- Decision: Prefer a single `--extensions <EXT>` flag over separate `--install` / `--load` flags.
- Rationale: The common case is "ensure extension is ready"; a single flag removes ordering pitfalls.

## Output Shape (JSON)
The initial JSON output is intentionally minimal and should include:
- `status`: `"pass" | "fail" | "error"`
- `exit_code`: `0 | 1 | 2`
- `files`: array of `{ path, status }` (status at file granularity)
- `counts`: `{ files_total, files_passed, files_failed, files_errored }`

If any human-readable diagnostics are printed while in JSON mode, they should go to stderr so stdout remains valid JSON.

Additional details (first failure diagnostics, per-record failures, timings) can be added later as optional fields.

## Risks / Trade-offs
- Overriding clap's default exit behavior requires care to preserve helpful help/usage output while still returning exit code `1`.
- JSON mode must avoid mixing human output into stdout; diagnostics should go to stderr.

## Migration Plan
- No migration required: there is no existing stable CLI contract.
