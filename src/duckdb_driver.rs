use arrow_schema::DataType as ArrowDataType;
use duckdb::types::{TimeUnit, Type, Value, ValueRef};
use duckdb::{Connection, Error};
use sqllogictest::runner::DBOutput;
use sqllogictest::{DB, DefaultColumnType};

fn normalize_duckdb_error_message(msg: &str) -> Option<String> {
    // DuckDB (and host OSes) often include environment-specific details in I/O errors.
    // Normalize known-unstable variants into stable strings suitable for sqllogictest.
    if msg.contains("Failed to open file") {
        return Some("Failed to open file".to_string());
    }

    None
}

#[derive(Debug)]
pub enum DuckdbDriverError {
    Duckdb(Error),
    Normalized { display: String, source: Error },
}

impl std::fmt::Display for DuckdbDriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DuckdbDriverError::Duckdb(e) => std::fmt::Display::fmt(e, f),
            DuckdbDriverError::Normalized { display, .. } => f.write_str(display),
        }
    }
}

impl std::error::Error for DuckdbDriverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DuckdbDriverError::Duckdb(e) => Some(e),
            DuckdbDriverError::Normalized { source, .. } => Some(source),
        }
    }
}

impl From<Error> for DuckdbDriverError {
    fn from(e: Error) -> Self {
        let msg = e.to_string();
        if let Some(display) = normalize_duckdb_error_message(&msg) {
            return DuckdbDriverError::Normalized { display, source: e };
        }

        DuckdbDriverError::Duckdb(e)
    }
}

pub struct DuckdbDriver {
    conn: Connection,
}

impl DuckdbDriver {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    fn format_value(value: ValueRef<'_>) -> Result<String, Error> {
        let s = match value {
            ValueRef::Null => "NULL".to_string(),
            ValueRef::Text(_) => {
                let s = value.as_str()?;
                if s.is_empty() {
                    "(empty)".to_string()
                } else {
                    s.to_string()
                }
            }
            ValueRef::Boolean(v) => v.to_string(),
            ValueRef::TinyInt(v) => v.to_string(),
            ValueRef::SmallInt(v) => v.to_string(),
            ValueRef::Int(v) => v.to_string(),
            ValueRef::BigInt(v) => v.to_string(),
            ValueRef::HugeInt(v) => v.to_string(),
            ValueRef::UTinyInt(v) => v.to_string(),
            ValueRef::USmallInt(v) => v.to_string(),
            ValueRef::UInt(v) => v.to_string(),
            ValueRef::UBigInt(v) => v.to_string(),
            ValueRef::Float(v) => v.to_string(),
            ValueRef::Double(v) => v.to_string(),
            ValueRef::Decimal(v) => v.to_string(),
            ValueRef::Date32(days) => Self::format_date32(days),
            ValueRef::Time64(unit, v) => Self::format_time64(unit, v),
            ValueRef::Timestamp(unit, v) => Self::format_timestamp(unit, v),
            ValueRef::Interval {
                months,
                days,
                nanos,
            } => Self::format_interval(months, days, nanos),
            ValueRef::Blob(b) => Self::format_blob_hex(b),

            v @ (ValueRef::List(..)
            | ValueRef::Struct(..)
            | ValueRef::Map(..)
            | ValueRef::Array(..)) => Self::format_advanced_value(v.to_owned()),

            other => format!("{other:?}"),
        };

        Ok(s)
    }

    fn format_blob_hex(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";

        let mut out = String::with_capacity(2 + (bytes.len() * 2));
        out.push_str("0x");
        for &b in bytes {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0x0f) as usize] as char);
        }
        out
    }

    fn format_date32(days_since_epoch: i32) -> String {
        let (y, m, d) = Self::civil_from_days(days_since_epoch as i64);
        format!("{y:04}-{m:02}-{d:02}")
    }

    fn format_time64(unit: TimeUnit, value: i64) -> String {
        let nanos = Self::time_unit_value_to_nanos(unit, value);
        Self::format_hms_nanos(nanos, unit)
    }

    fn format_timestamp(unit: TimeUnit, value: i64) -> String {
        let nanos = Self::time_unit_value_to_nanos(unit, value);
        let (secs, sub_nanos) = Self::div_mod_floor_i128(nanos, 1_000_000_000);
        let (days, secs_of_day) = Self::div_mod_floor_i128(secs, 86_400);

        let (y, m, d) = Self::civil_from_days(days as i64);
        let (hh, mm, ss) = Self::split_hms(secs_of_day as i64);

        let frac = Self::format_fractional_nanos(sub_nanos as u32, unit);
        if frac.is_empty() {
            format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{ss:02}")
        } else {
            format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{ss:02}.{frac}")
        }
    }

    fn format_interval(months: i32, days: i32, nanos: i64) -> String {
        let mut parts: Vec<String> = Vec::new();

        if months != 0 {
            let years = months / 12;
            let rem_months = months % 12;

            if years != 0 {
                parts.push(Self::pluralize(years, "year"));
            }
            if rem_months != 0 {
                parts.push(Self::pluralize(rem_months, "month"));
            }
        }

        if days != 0 {
            parts.push(Self::pluralize(days, "day"));
        }

        if nanos != 0 || parts.is_empty() {
            parts.push(Self::format_interval_time(nanos));
        }

        parts.join(" ")
    }

    fn pluralize(value: i32, unit: &str) -> String {
        let abs = value.unsigned_abs();
        if abs == 1 {
            format!("{value} {unit}")
        } else {
            format!("{value} {unit}s")
        }
    }

    fn format_interval_time(nanos: i64) -> String {
        let sign = if nanos < 0 { "-" } else { "" };
        let abs = nanos.unsigned_abs() as u128;

        let total_secs = abs / 1_000_000_000;
        let sub_nanos = (abs % 1_000_000_000) as u32;

        let hh = (total_secs / 3600) as u32;
        let mm = ((total_secs % 3600) / 60) as u32;
        let ss = (total_secs % 60) as u32;

        let frac = Self::format_fractional_nanos(sub_nanos, TimeUnit::Nanosecond);
        if frac.is_empty() {
            format!("{sign}{hh:02}:{mm:02}:{ss:02}")
        } else {
            format!("{sign}{hh:02}:{mm:02}:{ss:02}.{frac}")
        }
    }

    fn time_unit_value_to_nanos(unit: TimeUnit, value: i64) -> i128 {
        match unit {
            TimeUnit::Second => i128::from(value) * 1_000_000_000,
            TimeUnit::Millisecond => i128::from(value) * 1_000_000,
            TimeUnit::Microsecond => i128::from(value) * 1_000,
            TimeUnit::Nanosecond => i128::from(value),
        }
    }

    fn format_hms_nanos(nanos: i128, unit: TimeUnit) -> String {
        // TIME values are expected to be within a day; handle negatives/flooring defensively.
        let (secs, sub_nanos) = Self::div_mod_floor_i128(nanos, 1_000_000_000);
        let (_, secs_of_day) = Self::div_mod_floor_i128(secs, 86_400);
        let (hh, mm, ss) = Self::split_hms(secs_of_day as i64);

        let frac = Self::format_fractional_nanos(sub_nanos as u32, unit);
        if frac.is_empty() {
            format!("{hh:02}:{mm:02}:{ss:02}")
        } else {
            format!("{hh:02}:{mm:02}:{ss:02}.{frac}")
        }
    }

    fn split_hms(secs_of_day: i64) -> (u32, u32, u32) {
        let s = secs_of_day as u64;
        let hh = (s / 3600) as u32;
        let mm = ((s % 3600) / 60) as u32;
        let ss = (s % 60) as u32;
        (hh, mm, ss)
    }

    fn format_fractional_nanos(nanos: u32, unit: TimeUnit) -> String {
        if nanos == 0 {
            return String::new();
        }

        let (width, value) = match unit {
            TimeUnit::Second => return String::new(),
            TimeUnit::Millisecond => (3, nanos / 1_000_000),
            TimeUnit::Microsecond => (6, nanos / 1_000),
            TimeUnit::Nanosecond => (9, nanos),
        };

        let mut s = format!("{value:0width$}", width = width as usize);
        while s.ends_with('0') {
            s.pop();
        }
        s
    }

    fn div_mod_floor_i128(a: i128, b: i128) -> (i128, i128) {
        debug_assert!(b > 0);
        let mut q = a / b;
        let mut r = a % b;
        if r < 0 {
            q -= 1;
            r += b;
        }
        (q, r)
    }

    // Convert days since 1970-01-01 to a civil date (year, month, day).
    // Algorithm adapted from Howard Hinnant's civil_from_days.
    fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
        let z = days_since_epoch + 719_468;
        let era = if z >= 0 {
            z / 146_097
        } else {
            (z - 146_096) / 146_097
        };
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = mp + if mp < 10 { 3 } else { -9 };
        let year = y + if m <= 2 { 1 } else { 0 };
        (year as i32, m as u32, d as u32)
    }

    fn format_advanced_value(value: Value) -> String {
        Self::format_advanced_value_inner(&value)
    }

    fn format_advanced_value_inner(value: &Value) -> String {
        match value {
            Value::Null => "NULL".to_string(),
            Value::Boolean(v) => v.to_string(),
            Value::TinyInt(v) => v.to_string(),
            Value::SmallInt(v) => v.to_string(),
            Value::Int(v) => v.to_string(),
            Value::BigInt(v) => v.to_string(),
            Value::HugeInt(v) => v.to_string(),
            Value::UTinyInt(v) => v.to_string(),
            Value::USmallInt(v) => v.to_string(),
            Value::UInt(v) => v.to_string(),
            Value::UBigInt(v) => v.to_string(),
            Value::Float(v) => v.to_string(),
            Value::Double(v) => v.to_string(),
            Value::Decimal(v) => v.to_string(),
            Value::Timestamp(unit, v) => Self::format_timestamp(*unit, *v),
            Value::Date32(days) => Self::format_date32(*days),
            Value::Time64(unit, v) => Self::format_time64(*unit, *v),
            Value::Interval {
                months,
                days,
                nanos,
            } => Self::format_interval(*months, *days, *nanos),
            Value::Text(s) => Self::format_duckdb_nested_string(s),
            Value::Blob(b) => Self::format_blob_hex(b),
            Value::Enum(s) => Self::format_duckdb_nested_string(s),
            Value::List(items) => {
                let mut out = String::new();
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&Self::format_advanced_value_inner(item));
                }
                out.push(']');
                out
            }
            Value::Array(items) => {
                let mut out = String::new();
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&Self::format_advanced_value_inner(item));
                }
                out.push(']');
                out
            }
            Value::Struct(fields) => {
                let mut out = String::new();
                out.push('{');
                for (i, (name, val)) in fields.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&Self::quote_duckdb_string(name));
                    out.push_str(": ");
                    out.push_str(&Self::format_advanced_value_inner(val));
                }
                out.push('}');
                out
            }
            Value::Map(entries) => {
                let mut out = String::new();
                out.push('{');
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&Self::format_advanced_value_inner(k));
                    out.push('=');
                    out.push_str(&Self::format_advanced_value_inner(v));
                }
                out.push('}');
                out
            }
            Value::Union(v) => Self::format_advanced_value_inner(v),
        }
    }

    fn quote_duckdb_string(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 2);
        out.push('\'');
        for ch in s.chars() {
            if ch == '\'' {
                out.push('\'');
                out.push('\'');
            } else {
                out.push(ch);
            }
        }
        out.push('\'');
        out
    }

    fn format_duckdb_nested_string(s: &str) -> String {
        if s.is_empty() {
            return "''".to_string();
        }

        let can_be_bare = s.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_');
        if can_be_bare {
            return s.to_string();
        }

        Self::quote_duckdb_string(s)
    }

    fn map_duckdb_type(value_type: Type) -> DefaultColumnType {
        match value_type {
            Type::TinyInt
            | Type::SmallInt
            | Type::Int
            | Type::BigInt
            | Type::HugeInt
            | Type::UTinyInt
            | Type::USmallInt
            | Type::UInt
            | Type::UBigInt => DefaultColumnType::Integer,
            Type::Float | Type::Double | Type::Decimal => DefaultColumnType::FloatingPoint,
            Type::Text => DefaultColumnType::Text,
            Type::Date32 | Type::Time64 | Type::Timestamp | Type::Interval => {
                DefaultColumnType::Text
            }
            Type::List(_) | Type::Struct(_) | Type::Map(_, _) | Type::Array(_, _) => {
                DefaultColumnType::Text
            }
            _ => DefaultColumnType::Any,
        }
    }

    fn map_arrow_type(data_type: &ArrowDataType) -> DefaultColumnType {
        use ArrowDataType::*;

        match data_type {
            Int8 | Int16 | Int32 | Int64 | UInt8 | UInt16 | UInt32 | UInt64 => {
                DefaultColumnType::Integer
            }
            Float16 | Float32 | Float64 | Decimal128(_, _) | Decimal256(_, _) => {
                DefaultColumnType::FloatingPoint
            }
            Utf8 | LargeUtf8 => DefaultColumnType::Text,
            Date32
            | Time64(_)
            | Timestamp(_, _)
            | Interval(_)
            | List(_)
            | LargeList(_)
            | FixedSizeList(_, _)
            | Struct(_)
            | Map(_, _) => DefaultColumnType::Text,
            _ => DefaultColumnType::Any,
        }
    }

    fn collect_rows(
        stmt: &mut duckdb::Statement<'_>,
    ) -> Result<DBOutput<DefaultColumnType>, Error> {
        // The statement is assumed to already be executed; using raw_query avoids
        // executing it again (important for statements with side effects).
        let mut rows = stmt.raw_query();
        let stmt_ref = rows.as_ref().expect("rows should keep statement");
        let col_count = stmt_ref.column_count();

        let mut types: Vec<DefaultColumnType> = (0..col_count)
            .map(|i| {
                let dt: ArrowDataType = stmt_ref.column_type(i);
                Self::map_arrow_type(&dt)
            })
            .collect();

        let mut out_rows: Vec<Vec<String>> = Vec::new();

        if let Some(row) = rows.next()? {
            let mut out_row: Vec<String> = Vec::with_capacity(col_count);
            for (i, ty) in types.iter_mut().enumerate().take(col_count) {
                let value = row.get_ref(i)?;
                if *ty == DefaultColumnType::Any {
                    *ty = Self::map_duckdb_type(value.data_type());
                }
                out_row.push(Self::format_value(value)?);
            }
            out_rows.push(out_row);
        }

        while let Some(row) = rows.next()? {
            let mut out_row: Vec<String> = Vec::with_capacity(col_count);
            for i in 0..col_count {
                out_row.push(Self::format_value(row.get_ref(i)?)?);
            }
            out_rows.push(out_row);
        }

        Ok(DBOutput::Rows {
            types,
            rows: out_rows,
        })
    }

    fn collect_rows_via_query(
        stmt: &mut duckdb::Statement<'_>,
    ) -> Result<DBOutput<DefaultColumnType>, Error> {
        let mut rows = stmt.query([])?;
        let stmt_ref = rows.as_ref().expect("rows should keep statement");
        let col_count = stmt_ref.column_count();

        let mut types: Vec<DefaultColumnType> = (0..col_count)
            .map(|i| {
                let dt: ArrowDataType = stmt_ref.column_type(i);
                Self::map_arrow_type(&dt)
            })
            .collect();

        let mut out_rows: Vec<Vec<String>> = Vec::new();

        if let Some(row) = rows.next()? {
            let mut out_row: Vec<String> = Vec::with_capacity(col_count);
            for (i, ty) in types.iter_mut().enumerate().take(col_count) {
                let value = row.get_ref(i)?;
                if *ty == DefaultColumnType::Any {
                    *ty = Self::map_duckdb_type(value.data_type());
                }
                out_row.push(Self::format_value(value)?);
            }
            out_rows.push(out_row);
        }

        while let Some(row) = rows.next()? {
            let mut out_row: Vec<String> = Vec::with_capacity(col_count);
            for i in 0..col_count {
                out_row.push(Self::format_value(row.get_ref(i)?)?);
            }
            out_rows.push(out_row);
        }

        Ok(DBOutput::Rows {
            types,
            rows: out_rows,
        })
    }
}

impl DB for DuckdbDriver {
    type Error = DuckdbDriverError;
    type ColumnType = DefaultColumnType;

    fn run(&mut self, sql: &str) -> Result<DBOutput<Self::ColumnType>, Self::Error> {
        let mut stmt = self.conn.prepare(sql)?;

        match stmt.execute([]) {
            Ok(rows_changed) => {
                let has_result_set = stmt.column_count() > 0;
                if has_result_set {
                    Self::collect_rows(&mut stmt).map_err(Into::into)
                } else {
                    Ok(DBOutput::StatementComplete(rows_changed as u64))
                }
            }
            Err(Error::ExecuteReturnedResults) => {
                Self::collect_rows_via_query(&mut stmt).map_err(Into::into)
            }
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_duckdb_error_message_table() {
        let cases: &[(&str, Option<&str>)] = &[
            (
                "Invalid Input Error: Failed to open file 'x': The system cannot find the file specified. (os error 2)",
                Some("Failed to open file"),
            ),
            (
                "IO Error: Failed to open file 'x': Permission denied",
                Some("Failed to open file"),
            ),
            ("Catalog Error: Table with name t does not exist", None),
        ];

        for (input, expected) in cases {
            let actual = normalize_duckdb_error_message(input);
            assert_eq!(actual.as_deref(), *expected);
        }
    }

    #[test]
    fn map_duckdb_types() {
        assert_eq!(
            DuckdbDriver::map_duckdb_type(Type::Int),
            DefaultColumnType::Integer
        );
        assert_eq!(
            DuckdbDriver::map_duckdb_type(Type::Double),
            DefaultColumnType::FloatingPoint
        );
        assert_eq!(
            DuckdbDriver::map_duckdb_type(Type::Text),
            DefaultColumnType::Text
        );
        assert_eq!(
            DuckdbDriver::map_duckdb_type(Type::Date32),
            DefaultColumnType::Text
        );
        assert_eq!(
            DuckdbDriver::map_duckdb_type(Type::Time64),
            DefaultColumnType::Text
        );
        assert_eq!(
            DuckdbDriver::map_duckdb_type(Type::Timestamp),
            DefaultColumnType::Text
        );
        assert_eq!(
            DuckdbDriver::map_duckdb_type(Type::Interval),
            DefaultColumnType::Text
        );
        assert_eq!(
            DuckdbDriver::map_duckdb_type(Type::List(Box::new(Type::Int))),
            DefaultColumnType::Text
        );
        assert_eq!(
            DuckdbDriver::map_duckdb_type(Type::Struct(vec![("x".to_string(), Type::Int)])),
            DefaultColumnType::Text
        );
        assert_eq!(
            DuckdbDriver::map_duckdb_type(Type::Map(Box::new(Type::Text), Box::new(Type::Int))),
            DefaultColumnType::Text
        );
        assert_eq!(
            DuckdbDriver::map_duckdb_type(Type::Boolean),
            DefaultColumnType::Any
        );
        assert_eq!(
            DuckdbDriver::map_duckdb_type(Type::Blob),
            DefaultColumnType::Any
        );
    }

    #[test]
    fn map_arrow_types() {
        assert_eq!(
            DuckdbDriver::map_arrow_type(&ArrowDataType::Int32),
            DefaultColumnType::Integer
        );
        assert_eq!(
            DuckdbDriver::map_arrow_type(&ArrowDataType::UInt64),
            DefaultColumnType::Integer
        );
        assert_eq!(
            DuckdbDriver::map_arrow_type(&ArrowDataType::Float64),
            DefaultColumnType::FloatingPoint
        );
        assert_eq!(
            DuckdbDriver::map_arrow_type(&ArrowDataType::Utf8),
            DefaultColumnType::Text
        );
        assert_eq!(
            DuckdbDriver::map_arrow_type(&ArrowDataType::Date32),
            DefaultColumnType::Text
        );
        assert_eq!(
            DuckdbDriver::map_arrow_type(&ArrowDataType::Time64(
                arrow_schema::TimeUnit::Microsecond
            )),
            DefaultColumnType::Text
        );
        assert_eq!(
            DuckdbDriver::map_arrow_type(&ArrowDataType::Timestamp(
                arrow_schema::TimeUnit::Microsecond,
                None
            )),
            DefaultColumnType::Text
        );
        assert_eq!(
            DuckdbDriver::map_arrow_type(&ArrowDataType::Boolean),
            DefaultColumnType::Any
        );
    }

    #[test]
    fn format_values_golden() {
        assert_eq!(DuckdbDriver::format_value(ValueRef::Null).unwrap(), "NULL");
        assert_eq!(
            DuckdbDriver::format_value(ValueRef::Text(b""))
                .unwrap()
                .as_str(),
            "(empty)"
        );
        assert_eq!(
            DuckdbDriver::format_value(ValueRef::Text(b"hello")).unwrap(),
            "hello"
        );
        assert_eq!(
            DuckdbDriver::format_value(ValueRef::Boolean(true)).unwrap(),
            "true"
        );
        assert_eq!(DuckdbDriver::format_value(ValueRef::Int(-7)).unwrap(), "-7");
        assert_eq!(
            DuckdbDriver::format_value(ValueRef::UBigInt(7)).unwrap(),
            "7"
        );
        assert_eq!(
            DuckdbDriver::format_value(ValueRef::Float(1.5)).unwrap(),
            "1.5"
        );
        assert_eq!(
            DuckdbDriver::format_value(ValueRef::Double(2.25)).unwrap(),
            "2.25"
        );
        assert_eq!(
            DuckdbDriver::format_value(ValueRef::Blob(&[])).unwrap(),
            "0x"
        );
        assert_eq!(
            DuckdbDriver::format_value(ValueRef::Blob(&[0x00, 0x0f, 0x10, 0xff])).unwrap(),
            "0x000f10ff"
        );
    }

    #[test]
    fn format_values_match_cast_varchar() -> Result<(), Error> {
        let conn = Connection::open_in_memory()?;

        let cases = [
            "SELECT v, CAST(v AS VARCHAR) FROM (SELECT DATE '2020-01-02' AS v)",
            "SELECT v, CAST(v AS VARCHAR) FROM (SELECT TIME '03:04:05.006007' AS v)",
            "SELECT v, CAST(v AS VARCHAR) FROM (SELECT TIMESTAMP '2020-01-02 03:04:05.006007' AS v)",
            "SELECT v, CAST(v AS VARCHAR) FROM (SELECT CAST(1.50 AS DECIMAL(4,2)) AS v)",
            "SELECT v, CAST(v AS VARCHAR) FROM (SELECT INTERVAL '1 month 2 days 03:04:05.006007' AS v)",
        ];

        for sql in cases {
            let mut stmt = conn.prepare(sql)?;
            let mut rows = stmt.query([])?;
            let row = rows.next()?.expect("expected one row");

            let actual = DuckdbDriver::format_value(row.get_ref(0)?)?;
            let expected: String = row.get(1)?;
            assert_eq!(actual, expected, "sql: {sql}");
        }

        Ok(())
    }

    #[test]
    fn format_nested_values_match_cast_varchar() -> Result<(), Error> {
        let conn = Connection::open_in_memory()?;

        let cases = [
            "SELECT v, CAST(v AS VARCHAR) FROM (SELECT [1, 2, NULL] AS v)",
            "SELECT v, CAST(v AS VARCHAR) FROM (SELECT ['a', 'b', 'c'] AS v)",
            "SELECT v, CAST(v AS VARCHAR) FROM (SELECT {'a': 1, 'b': 2} AS v)",
            "SELECT v, CAST(v AS VARCHAR) FROM (SELECT map(['a', 'b'], [1, 2]) AS v)",
        ];

        for sql in cases {
            let mut stmt = conn.prepare(sql)?;
            let mut rows = stmt.query([])?;
            let row = rows.next()?.expect("expected one row");

            let actual = DuckdbDriver::format_value(row.get_ref(0)?)?;
            let expected: String = row.get(1)?;
            assert_eq!(actual, expected, "sql: {sql}");
        }

        Ok(())
    }
}
