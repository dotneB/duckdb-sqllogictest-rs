## Context

`DuckdbDriver::run` currently reaches two separate row-collection functions (`collect_rows` and `collect_rows_via_query`) depending on execution flow. Both paths duplicate the same core logic for type inference fallback (`Any` -> refined type on first row) and canonical value formatting. This duplication increases maintenance cost and creates avoidable drift risk when one path changes without the other.

## Goals / Non-Goals

**Goals:**
- Use a single shared row-collection pipeline for all result-producing driver flows.
- Preserve existing sqllogictest-visible behavior for type mapping and canonical formatting.
- Add regression coverage that fails if query/execute result collection diverges.

**Non-Goals:**
- Changing canonical formatting rules or column-type semantics.
- Changing CLI behavior, SQL preprocessor behavior, or connection setup.
- Reworking unrelated driver logic (error normalization, TZ casting strategy, etc.).

## Decisions

1. **Unify row collection behind one helper**
   - Introduce a single internal helper that performs schema-based type mapping, first-row refinement, and value stringification exactly once.
   - Both execution entry points (execute-with-results and query path) route through this helper.
   - Rationale: centralizes behavior and removes duplicate loops.

2. **Keep formatting and type-mapping primitives as the source of truth**
   - Continue to use existing `map_arrow_type`, `map_duckdb_type`, and `format_value` primitives from the shared collector.
   - Rationale: preserves existing output semantics while reducing call-site duplication.

3. **Preserve run-path branching behavior, only refactor collection internals**
   - Keep select detection, TZ column wrapping, and statement/query branching decisions unchanged.
   - Rationale: limit change surface to the refactor objective and reduce regression risk.

4. **Add parity-focused tests**
   - Add tests that validate row outputs and inferred column types remain identical across result-producing paths.
   - Rationale: locks in the new single-path contract and prevents regressions.

## Risks / Trade-offs

- [Risk] Helper abstraction could accidentally alter row iteration order or first-row refinement timing -> Mitigation: preserve loop order and add golden/parity assertions.
- [Risk] Refactor may hide path-specific assumptions (e.g., statement execution state) -> Mitigation: keep path-specific setup outside the shared collector and test both entry paths.
- [Trade-off] Slightly more indirection in code flow -> Mitigation: prefer explicit helper naming and keep interfaces narrow.

## Migration Plan

1. Implement the shared collector and route both existing result-producing paths through it.
2. Run existing driver unit tests and add/adjust parity tests.
3. If regressions appear, revert to prior collection functions and reintroduce change incrementally.

Rollback strategy: revert the refactor commit; no persisted data or protocol migration is involved.

## Open Questions

- None currently; behavior-preserving refactor scope is clear.
