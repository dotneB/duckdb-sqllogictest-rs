## MODIFIED Requirements

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

## REMOVED Requirements

### Requirement: Feature-Gated Advanced Type Formatting
**Reason**: Nested type formatting is part of canonical value stringification and must not vary by cargo feature selection.
**Migration**: Remove use of `advanced-type-formatting` from downstream `Cargo.toml` feature selections. Nested formatting is always enabled.
