## Context
The driver currently implements its own formatting for several date/time-like values. For `TIME WITH TIME ZONE`, DuckDB’s canonical string form includes the offset (e.g., `12:00:00+00`), but the current formatting path drops that suffix.

The spec defines canonical formatting for date/time-like values as equivalent to `CAST(value AS VARCHAR)`.

## Goals / Non-Goals
- Goals:
  - Ensure `TIMETZ` results include the offset as DuckDB prints it.
  - Keep behavior consistent with the spec’s `CAST(value AS VARCHAR)` definition.
- Non-Goals:
  - Expand type mapping/validation beyond stringification.

## Decisions
- Prefer using DuckDB’s own stringification for time zone aware values rather than reimplementing formatting rules.

## Risks / Trade-offs
- Any existing tests that accidentally relied on the offset being dropped will start failing; those fixtures should be updated to match DuckDB’s canonical output.
