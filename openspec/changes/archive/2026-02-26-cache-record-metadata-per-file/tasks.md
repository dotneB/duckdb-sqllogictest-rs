## 1. Build per-file record metadata cache

- [x] 1.1 Add a helper data structure for `line -> (record_index_1_based, optional_label)` metadata derived from parsed sqllogictest records.
- [x] 1.2 Implement a builder that parses a file once, collects record index/label metadata, and preserves current location matching semantics.

## 2. Integrate cache into mismatch reporting

- [x] 2.1 Update file execution flow to construct record metadata once per input file run and make it available to mismatch rendering.
- [x] 2.2 Refactor failure-report lookup/render helpers to read record identifiers from cached metadata instead of reparsing/scanning on each mismatch.
- [x] 2.3 Keep fallback behavior unchanged when a failure location has no matching cached record metadata.

## 3. Validate behavior and regressions

- [x] 3.1 Ensure existing mismatch diagnostics coverage still passes (record index/name and SQL remain in stderr output).
- [x] 3.2 Add or update tests to exercise repeated mismatch lookups within the same file execution and confirm stable diagnostics behavior.
- [x] 3.3 Run the relevant test suite and fix any failures introduced by the caching refactor.
