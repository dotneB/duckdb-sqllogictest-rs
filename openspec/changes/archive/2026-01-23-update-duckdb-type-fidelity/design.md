## Context
`duckdb-slt` compares results using sqllogictest's string-based semantics. DuckDB's own test suites commonly expect specific textual renderings for non-primitive values (date/time/timestamps, intervals, decimals, and nested values). The current driver formats only core primitives canonically and formats everything else using Rust debug output, which is not DuckDB-compatible.

## Goals / Non-Goals
- Goals:
  - Match DuckDB's canonical textual output for the requested types in a deterministic, locale-independent way.
  - Keep the implementation small and isolated to the driver.
  - Provide golden tests that prevent regressions across DuckDB upgrades.
- Non-Goals:
  - Full parity with every DuckDB type (e.g., geography, enums, unions) unless needed by suites.
  - Changing sqllogictest parsing/comparison behavior.

## Decisions
- Decision: Treat the required non-primitive types as `Text` at the sqllogictest column-type level.
  - Rationale: DuckDB suites typically assert these values textually; mapping them as `Any` can create type-mismatch failures depending on suite expectations.

- Decision: Define “DuckDB-compatible stringification” as the output of `CAST(value AS VARCHAR)` for the value as produced by DuckDB.
  - Rationale: This gives a precise reference and avoids guessing punctuation/precision rules.

- Decision: Feature-gate nested type formatting behind a cargo feature (default-enabled).
  - Rationale: Nested formatting may require additional conversion logic or dependencies; gating provides an escape hatch without changing the CLI surface.
  - Proposed feature name: `advanced-type-formatting` (enabled by default).

## Risks / Trade-offs
- Performance: If nested formatting requires conversions beyond the existing `ValueRef` matching, it must avoid per-cell SQL roundtrips.
- DuckDB version drift: Output rules for some types (notably `INTERVAL` and `DECIMAL`) can change across DuckDB versions; golden tests should reflect the version pinned via `duckdb` crate.
- Timestamp semantics: DuckDB has multiple timestamp flavors (with/without timezone). The implementation should be explicit about which variants are supported and how they render.

## Migration Plan
- No user migration steps expected.
- If callers relied on debug formatting for nested types, they can disable the `advanced-type-formatting` cargo feature.

## Open Questions
- Should nested type formatting be enabled unconditionally once implemented, or should the project default to the opt-out feature as proposed? (Default proposed: enabled.)
- Which timestamp variants are required by the target suites (`TIMESTAMP`, `TIMESTAMPTZ`, etc.)?
