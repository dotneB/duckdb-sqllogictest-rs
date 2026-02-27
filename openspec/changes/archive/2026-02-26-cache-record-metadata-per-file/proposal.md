## Why

Failure reports currently re-parse and re-scan sqllogictest files each time a mismatch is rendered to recover record identifiers. In large failing suites, that repeated work adds avoidable overhead and makes mismatch rendering paths more complex than necessary.

## What Changes

- Cache per-file record metadata for failure reporting, keyed by source line.
- Build the line-to-record map once per file and reuse it for all mismatches in that file.
- Keep mismatch report output unchanged (location, record index/name when present, SQL, and detail fields).
- Refactor mismatch rendering flow to consume cached metadata instead of repeatedly parsing files.

## Capabilities

### New Capabilities
- _None_

### Modified Capabilities
- `duckdb-slt-cli`: failure-report record identifier lookup is required to reuse per-file cached record metadata rather than reparsing on each mismatch.

## Impact

- Affected code: `src/main.rs` mismatch reporting helpers and file-run flow, with potential helper structs/maps for cached metadata.
- Tests: existing mismatch output tests must continue to pass; add targeted coverage for repeated lookups within one file.
- APIs/dependencies: no CLI flag changes and no new external dependencies expected.
