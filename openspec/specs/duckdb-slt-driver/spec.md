# duckdb-slt-driver Specification

## Purpose
TBD - created by archiving change update-duckdb-driver-canonical-formatting. Update Purpose after archive.
## Requirements
### Requirement: DuckDB sqllogictest Driver
The system SHALL provide a DuckDB-backed implementation of `sqllogictest::DB` used by `duckdb-slt`.

The driver SHALL execute input SQL and return either statement completion or query results using sqllogictest runner types.

#### Scenario: Driver executes a statement
- **WHEN** the runner calls `DB::run` with a SQL statement that does not return rows
- **THEN** the driver returns `DBOutput::StatementComplete(rows_changed)`

#### Scenario: Driver executes a query
- **WHEN** the runner calls `DB::run` with a SQL query that returns rows
- **THEN** the driver returns `DBOutput::Rows { types, rows }` containing all returned rows and per-column types

#### Scenario: Driver distinguishes query from statement
- **WHEN** DuckDB indicates results are available during execution (e.g., "execute returned results")
- **THEN** the driver falls back to querying and collecting rows

### Requirement: Column Type Mapping
The driver SHALL map DuckDB (and Arrow schema types where available) into `sqllogictest::DefaultColumnType` such that sqllogictest comparisons use stable column expectations.

The mapping SHALL be:
- Integer-like types -> `DefaultColumnType::Integer`
- Floating-like types (including decimal) -> `DefaultColumnType::FloatingPoint`
- UTF-8 text-like types -> `DefaultColumnType::Text`
- Date/time-like types (including date, time, timestamps, and intervals) -> `DefaultColumnType::Text`
- Nested types (including lists, structs, and maps) -> `DefaultColumnType::Text`
- Any other type -> `DefaultColumnType::Any`

#### Scenario: Integer mapping
- **WHEN** a query column has an integer-like type (e.g., i32/u64)
- **THEN** the driver assigns `DefaultColumnType::Integer` for that column

#### Scenario: Floating mapping
- **WHEN** a query column has a floating-like type (e.g., f32/f64/decimal)
- **THEN** the driver assigns `DefaultColumnType::FloatingPoint` for that column

#### Scenario: Text mapping
- **WHEN** a query column has a UTF-8 text-like type
- **THEN** the driver assigns `DefaultColumnType::Text` for that column

#### Scenario: Date/time mapping
- **WHEN** a query column has a date/time-like type (e.g., DATE/TIME/TIMESTAMP/INTERVAL)
- **THEN** the driver assigns `DefaultColumnType::Text` for that column

#### Scenario: Nested mapping
- **WHEN** a query column has a nested type (e.g., LIST/STRUCT/MAP)
- **THEN** the driver assigns `DefaultColumnType::Text` for that column

#### Scenario: Fallback mapping
- **WHEN** a query column has an unrecognized, complex, or unsupported type
- **THEN** the driver assigns `DefaultColumnType::Any` for that column

### Requirement: Canonical Value Stringification
The driver SHALL provide canonical stringification for DuckDB values so that sqllogictest comparisons are deterministic and compatible with DuckDB suite expectations.

The canonical formatting SHALL be:
- NULL -> `NULL`
- Text -> exact text; empty string -> `(empty)`
- Bool -> `true` or `false`
- Signed/unsigned integers -> base-10 digits with no separators
- Float/double -> Rust `Display` formatting (locale-independent)
- Decimal -> DuckDB-compatible decimal string form
- Date/time-like values (DATE, TIME, TIMESTAMP, INTERVAL) -> DuckDB-compatible string form
- Nested values (LIST, STRUCT, MAP) -> DuckDB-compatible string form
- Blob -> lower-case hex prefixed with `0x`

DuckDB-compatible string form SHALL be defined as the result of DuckDB stringification equivalent to `CAST(value AS VARCHAR)`.

#### Scenario: NULL value renders as NULL
- **WHEN** DuckDB returns a NULL cell
- **THEN** the driver renders the cell as `NULL`

#### Scenario: Empty string renders as (empty)
- **WHEN** DuckDB returns an empty string value
- **THEN** the driver renders the cell as `(empty)`

#### Scenario: Decimal renders in DuckDB-compatible form
- **WHEN** DuckDB returns a DECIMAL value
- **THEN** the driver renders the cell using DuckDB-compatible decimal stringification

#### Scenario: Date/time values render in DuckDB-compatible form
- **WHEN** DuckDB returns a DATE/TIME/TIMESTAMP/INTERVAL value
- **THEN** the driver renders the cell using DuckDB-compatible date/time stringification

#### Scenario: Time with time zone renders in DuckDB-compatible form
- **WHEN** DuckDB returns a `TIME WITH TIME ZONE` value
- **THEN** the driver renders the cell including the time zone offset exactly as DuckDB would for `CAST(value AS VARCHAR)`

#### Scenario: Nested values render in DuckDB-compatible form
- **WHEN** DuckDB returns a LIST/STRUCT/MAP value
- **THEN** the driver renders the cell using DuckDB-compatible nested stringification

#### Scenario: Blob renders as hex
- **WHEN** DuckDB returns a blob value
- **THEN** the driver renders the cell as `0x` followed by lower-case hex bytes

#### Scenario: Integers render as base-10 digits
- **WHEN** DuckDB returns an integer or unsigned integer value
- **THEN** the driver renders the cell using base-10 digits with no separators

#### Scenario: Floating point renders deterministically
- **WHEN** DuckDB returns a floating point value
- **THEN** the driver renders the cell using Rust `Display` formatting

### Requirement: Stable DuckDB Error Normalization
When DuckDB returns an execution error, the driver SHALL normalize select error messages into stable, portable strings suitable for sqllogictest `expected_error` comparisons.

Normalization SHALL be conservative and pattern-based.

#### Scenario: Missing file open errors normalize to a stable message
- **WHEN** DuckDB returns a file-open error whose message includes OS-specific details (e.g., `The system cannot find the file specified. (os error 2)`)
- **THEN** the driver provides an error string that includes `Failed to open file` and excludes the OS-specific suffix

#### Scenario: Unknown errors are not rewritten
- **WHEN** DuckDB returns an error that does not match a supported normalization pattern
- **THEN** the driver preserves the original error message for comparison and reporting

