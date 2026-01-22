## ADDED Requirements

### Requirement: Stable DuckDB Error Normalization
When DuckDB returns an execution error, the driver SHALL normalize select error messages into stable, portable strings suitable for sqllogictest `expected_error` comparisons.

Normalization SHALL be conservative and pattern-based.

#### Scenario: Missing file open errors normalize to a stable message
- **WHEN** DuckDB returns a file-open error whose message includes OS-specific details (e.g., `The system cannot find the file specified. (os error 2)`)
- **THEN** the driver provides an error string that includes `Failed to open file` and excludes the OS-specific suffix

#### Scenario: Unknown errors are not rewritten
- **WHEN** DuckDB returns an error that does not match a supported normalization pattern
- **THEN** the driver preserves the original error message for comparison and reporting
