## ADDED Requirements

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

#### Scenario: Fallback mapping
- **WHEN** a query column has an unrecognized, complex, or unsupported type
- **THEN** the driver assigns `DefaultColumnType::Any` for that column

### Requirement: Canonical Value Stringification
The driver SHALL provide canonical stringification for core DuckDB values so that sqllogictest comparisons are deterministic.

The canonical formatting SHALL be:
- NULL -> `NULL`
- Text -> exact text; empty string -> `(empty)`
- Bool -> `true` or `false`
- Signed/unsigned integers -> base-10 digits with no separators
- Float/double -> Rust `Display` formatting (locale-independent)
- Blob -> lower-case hex prefixed with `0x`

#### Scenario: NULL value renders as NULL
- **WHEN** DuckDB returns a NULL cell
- **THEN** the driver renders the cell as `NULL`

#### Scenario: Empty string renders as (empty)
- **WHEN** DuckDB returns an empty string value
- **THEN** the driver renders the cell as `(empty)`

#### Scenario: Query returns zero rows
- **WHEN** the runner executes a query that returns zero rows
- **THEN** the driver returns `DBOutput::Rows` with the correct column count and types

#### Scenario: Statement returns rows without re-executing
- **WHEN** the runner executes a statement that returns rows (e.g., `INSERT ... RETURNING`)
- **THEN** the driver returns those rows without executing the statement more than once

#### Scenario: Blob renders as hex
- **WHEN** DuckDB returns a blob value
- **THEN** the driver renders the cell as `0x` followed by lower-case hex bytes

#### Scenario: Integers render as base-10 digits
- **WHEN** DuckDB returns an integer or unsigned integer value
- **THEN** the driver renders the cell using base-10 digits with no separators

#### Scenario: Floating point renders deterministically
- **WHEN** DuckDB returns a floating point value
- **THEN** the driver renders the cell using Rust `Display` formatting
