## ADDED Requirements

### Requirement: Feature-Gated Advanced Type Formatting
The driver SHALL support feature-gating nested type (list/struct/map) formatting behind a cargo feature.

When the `advanced-type-formatting` feature is enabled, the driver SHALL render nested values using DuckDB-compatible stringification.

When the `advanced-type-formatting` feature is disabled, the driver SHALL fall back to the pre-existing debug-oriented formatting for nested values.

#### Scenario: Nested formatting enabled
- **WHEN** the driver is built with the `advanced-type-formatting` feature enabled and a query returns a nested value
- **THEN** the driver renders the nested value in DuckDB-compatible string form

#### Scenario: Nested formatting disabled
- **WHEN** the driver is built with the `advanced-type-formatting` feature disabled and a query returns a nested value
- **THEN** the driver renders the nested value using its fallback (non-DuckDB-compatible) formatting

## MODIFIED Requirements

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
- Nested values (LIST, STRUCT, MAP) -> DuckDB-compatible string form (when feature-enabled)
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

#### Scenario: Nested values render in DuckDB-compatible form
- **WHEN** DuckDB returns a LIST/STRUCT/MAP value and nested formatting is enabled
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
