## 1. Unify Driver Row Collection

- [x] 1.1 Extract a shared internal row-collection helper that performs schema type mapping, first-row type refinement, and canonical value formatting.
- [x] 1.2 Update both result-producing execution paths to use the shared helper and remove duplicated collection logic.
- [x] 1.3 Keep query/statement branch behavior unchanged while ensuring execute-returned-results fallback also routes through the shared collection path.

## 2. Validate Behavioral Parity

- [x] 2.1 Add or update driver tests to assert identical `types` and `rows` output across query and execute-result collection paths.
- [x] 2.2 Run the existing duckdb driver test suite (including formatting/type mapping tests) and fix any regressions introduced by the refactor.

## 3. Verify Change Readiness

- [x] 3.1 Confirm proposal/design/spec artifacts still match the implemented refactor scope and update wording if implementation details shift.
- [x] 3.2 Run full project checks used for refactor validation and document any follow-up work discovered during implementation.
- [x] 3.3 `just dev`
- [x] 3.4 `just full`
