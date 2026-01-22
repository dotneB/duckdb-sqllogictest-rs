use arrow_schema::DataType as ArrowDataType;
use duckdb::types::{Type, ValueRef};
use duckdb::{Connection, Error};
use sqllogictest::runner::DBOutput;
use sqllogictest::{DB, DefaultColumnType};

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
            ValueRef::Blob(b) => Self::format_blob_hex(b),
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
    type Error = Error;
    type ColumnType = DefaultColumnType;

    fn run(&mut self, sql: &str) -> Result<DBOutput<Self::ColumnType>, Self::Error> {
        let mut stmt = self.conn.prepare(sql)?;

        match stmt.execute([]) {
            Ok(rows_changed) => {
                let has_result_set = stmt.column_count() > 0;
                if has_result_set {
                    Self::collect_rows(&mut stmt)
                } else {
                    Ok(DBOutput::StatementComplete(rows_changed as u64))
                }
            }
            Err(Error::ExecuteReturnedResults) => Self::collect_rows_via_query(&mut stmt),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
