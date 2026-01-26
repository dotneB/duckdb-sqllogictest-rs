## ADDED Requirements

### Requirement: Query Column Count Validation
For each `query` record, the system SHALL validate that the number of columns returned by DuckDB matches the number of columns declared by the query type string (e.g., `query III` declares 3 columns).

When the returned column count does not match the declared column count, the system SHALL fail the test.

#### Scenario: Query declares more columns than returned
- **WHEN** a test contains `query III` and the SQL returns 1 column
- **THEN** the test fails with a column-count mismatch

#### Scenario: Query declares fewer columns than returned
- **WHEN** a test contains `query I` and the SQL returns 2 columns
- **THEN** the test fails with a column-count mismatch

### Requirement: Column Count Mismatch Diagnostics
When a test fails due to a query column-count mismatch, the system SHALL include in its failure report:
- the expected column count
- the actual column count

#### Scenario: Failure report includes expected and actual counts
- **WHEN** a query fails due to a column-count mismatch
- **THEN** stderr includes text equivalent to `Expected X columns, but got Y columns`
