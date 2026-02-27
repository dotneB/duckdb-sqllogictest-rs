## MODIFIED Requirements

### Requirement: Mismatch Failure Diagnostics
When an expectation mismatch occurs while running `duckdb-slt`, the system SHALL write a human-readable failure report to stderr that includes:
- the input file path
- a record identifier (record index; and record name when available)
- the SQL statement (or a recognizable snippet of it)
- the expected output
- the actual output

For each input file execution, the system SHALL parse record metadata for mismatch diagnostics at most once and SHALL reuse that metadata for every mismatch reported for the same file.

Record metadata reuse SHALL preserve the same record identifier and SQL context that would be produced by location-based lookup.

#### Scenario: Query mismatch prints file, record identifier, and SQL
- **WHEN** a query record mismatches between expected and actual output
- **THEN** stderr includes the file path, the record identifier, and the SQL statement for the failing record

#### Scenario: Statement mismatch prints expected vs actual
- **WHEN** a statement record mismatches due to an unexpected error or unexpected success
- **THEN** stderr includes both the expected outcome and the actual outcome

#### Scenario: Multiple mismatches in one file reuse metadata cache
- **WHEN** two or more mismatches are reported while executing the same sqllogictest file
- **THEN** record metadata is parsed once for that file and reused for each mismatch report
