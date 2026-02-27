## Context

`duckdb-slt` currently resolves mismatch record identifiers by reparsing the relevant sqllogictest file every time a mismatch report is rendered. The lookup path scans records to find the record whose location matches the failing line, then extracts an index and optional label. This is correct but inefficient for large files with multiple mismatches.

The change must preserve existing stderr diagnostics (file location, record identifier, SQL context, expected/actual details) while reducing repeated parsing and simplifying the mismatch rendering path.

## Goals / Non-Goals

**Goals:**
- Parse record metadata once per file execution and reuse it for all mismatch reports in that file.
- Provide O(1) or near O(1) line-to-record lookup during mismatch rendering.
- Keep existing mismatch report content and formatting unchanged.

**Non-Goals:**
- Changing user-facing CLI flags or output format.
- Introducing cross-file or cross-process persistent caches.
- Refactoring unrelated sqllogictest execution paths.

## Decisions

1. Build a per-file metadata index keyed by line number.
   - Decision: create a map `line -> (record_index_1_based, optional_label)` for each parsed file.
   - Rationale: direct lookup by failure line removes repeated record scans and keeps record lookup deterministic.
   - Alternative considered: keep the existing scan and memoize only the latest lookup. Rejected because failures can occur on many different lines and this still repeats parsing/scanning work.

2. Populate metadata once for each file run and pass it into failure rendering.
   - Decision: construct metadata in the file execution flow and reuse it when rendering every mismatch for that file.
   - Rationale: explicit ownership avoids hidden global state and makes lifecycle clear (cache lives for one file run).
   - Alternative considered: lazy static/global cache keyed by path. Rejected due invalidation complexity and unnecessary memory retention.

3. Preserve current matching semantics and fallback behavior.
   - Decision: keep source-location matching semantics and continue gracefully when metadata for a line is unavailable.
   - Rationale: avoids regressions in diagnostics while still improving the common case.
   - Alternative considered: hard-fail if metadata is missing. Rejected because it would make diagnostics brittle.

## Risks / Trade-offs

- [Path normalization mismatch between parsed records and failure locations] -> Mitigation: use the same location comparison semantics currently used by mismatch reporting and add tests that assert record/name output remains present.
- [Additional transient memory usage for line metadata maps] -> Mitigation: scope cache lifetime to a single file execution and store only small metadata tuples.
- [Potential behavior drift in label extraction] -> Mitigation: reuse current label extraction logic (`QueryExpect::Results`) unchanged while only moving when it is computed.

## Migration Plan

- Implement the metadata cache behind existing mismatch-report APIs without changing CLI behavior.
- Validate with existing mismatch diagnostics tests and new coverage for repeated mismatches in one file.
- Rollback path: revert to the prior direct parse/scan lookup implementation if regressions appear.

## Open Questions

- None.
