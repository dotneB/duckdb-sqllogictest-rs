## ADDED Requirements

### Requirement: Additional DuckDB Compatibility Directives
The CLI preprocessing pipeline SHALL support the additional DuckDB compatibility directives required by the pinned corpus subset, including `skipif` and `onlyif` conditions.

#### Scenario: `skipif duckdb` skips the following record
- **WHEN** preprocessing encounters `skipif duckdb` before a record
- **THEN** the directive and its associated record are excluded from execution for the DuckDB runner

#### Scenario: `onlyif` non-DuckDB target skips the following record
- **WHEN** preprocessing encounters `onlyif <target>` where `<target>` is not `duckdb`
- **THEN** the directive and its associated record are excluded from execution for the DuckDB runner

#### Scenario: `onlyif duckdb` preserves execution
- **WHEN** preprocessing encounters `onlyif duckdb` before a record
- **THEN** the associated record remains executable by `sqllogictest`

### Requirement: Directive Rewrite Line Stability
Directive preprocessing SHALL preserve source line numbering so mismatch and parse diagnostics continue to report stable file and line locations.

#### Scenario: Rewritten directives keep line count stable
- **WHEN** preprocessing rewrites compatibility directives into parser-safe lines
- **THEN** the generated script preserves original line count and relative line mapping for downstream error reporting

### Requirement: Required Directive Coverage Validation
Compatibility runs SHALL fail with an actionable runtime error if a corpus-selected file contains a directive required by the harness but not yet implemented in the preprocessor pipeline.

#### Scenario: Unsupported required directive is reported
- **WHEN** a selected corpus file contains an unsupported required directive
- **THEN** the run fails with the directive keyword and source location in the error output
