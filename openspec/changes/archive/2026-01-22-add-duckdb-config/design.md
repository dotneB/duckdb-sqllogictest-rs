## Context
The runner embeds DuckDB (via `duckdb-rs`) and will open a connection for executing sqllogictest files. DuckDB supports configuration at open time via `duckdb::Config`, including an explicit opt-in for allowing unsigned extensions.

## Goals / Non-Goals
- Goals:
- Ensure connection creation is configurable and uses `duckdb::Config`.
- Ensure unsigned extensions are opt-in.
- Non-Goals:
- Implementing extension `INSTALL`/`LOAD` behavior (tracked under the CLI contract change).

## Decisions
- Decision: Use `duckdb::Config` for all connections.
- Rationale: Centralizes settings and makes configuration explicit and testable.

## Risks / Trade-offs
- Allowing unsigned extensions increases risk if a caller loads untrusted extensions.
