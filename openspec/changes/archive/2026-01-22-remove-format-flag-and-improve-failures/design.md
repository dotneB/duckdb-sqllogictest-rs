## Context
The CLI currently includes a `--format` flag and JSON summary output, while also forwarding `sqllogictest` failure strings to stderr. We want to remove the extra output mode and make mismatch failures deterministic and easy-to-grep for both local runs and CI.

## Goals / Non-Goals
- Goals:
  - Remove `--format` and JSON summary output to simplify the CLI contract.
  - Make mismatch failures self-contained: a user can copy/paste the SQL and see expected vs actual.
  - Include a stable record identifier (index and name when present) to quickly locate the failing record.
  - Preserve the existing human-readable output style (progress and summary).
- Non-Goals:
  - Adding new output modes (e.g. JUnit flags) as part of this change.
  - Replacing `sqllogictest` comparison semantics.

## Decisions
- Failure report format (text): print a structured block to stderr on each mismatch, with explicit section labels.
- Data source: use `sqllogictest::TestErrorKind` fields for SQL + expected/actual (when present), and `TestError::location()` (file + line + include stack) for pinpointing the failing record.
- Record identifier: derive a stable `record <index>` by parsing the failing script (`sqllogictest::parse_file`) and matching the failing `Location.line()` to a record start line; include an optional record label/name when available (e.g. `QueryExpect::Results.label` or `Record::Subtest.name`).
- SQL snippet: include the SQL string from `TestErrorKind` variants that carry it.
- Expected vs actual: include the exact expected and actual strings carried by mismatch variants (e.g. `QueryResultMismatch { expected, actual }`).

## Risks / Trade-offs
- `sqllogictest` may not expose all desired fields directly; parsing human strings is less robust across dependency upgrades.
- Large SQL statements or outputs could make stderr noisy; initial implementation should remain straightforward and can add truncation controls later if needed.

## Notes
- In `sqllogictest` v0.29.0, `Location` provides `file()` and `line()`; `TestError` prints `"{kind}\nat {loc}"`.
- `TestErrorKind` mismatch variants already include `sql` and expected/actual strings, so we can avoid parsing formatted error strings.

## Migration Plan
- This change is a breaking CLI change: `--format` is removed and JSON summary output is removed.
- Tests will pin the most important substrings (record identifier + SQL) to avoid over-constraining formatting.

## Open Questions
- None.
