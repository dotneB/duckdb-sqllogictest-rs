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
            ValueRef::Text(_) => value.as_str()?.to_string(),
            ValueRef::Blob(b) => {
                // Avoid introducing a new encoding contract; this is enough for
                // our current fixture coverage.
                format!("{:?}", b)
            }
            ValueRef::Timestamp(_, v) => v.to_string(),
            other => format!("{other:?}"),
        };

        Ok(s)
    }

    fn infer_column_type(value_type: Type) -> DefaultColumnType {
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
}

impl DB for DuckdbDriver {
    type Error = Error;
    type ColumnType = DefaultColumnType;

    fn run(&mut self, sql: &str) -> Result<DBOutput<Self::ColumnType>, Self::Error> {
        let mut stmt = self.conn.prepare(sql)?;

        match stmt.query([]) {
            Ok(mut rows) => {
                let mut types: Vec<DefaultColumnType> = Vec::new();
                let mut out_rows: Vec<Vec<String>> = Vec::new();
                let mut col_count: usize = 0;

                if let Some(row) = rows.next()? {
                    let mut out_row: Vec<String> = Vec::new();
                    for i in 0.. {
                        match row.get_ref(i) {
                            Ok(value) => {
                                col_count += 1;
                                types.push(Self::infer_column_type(value.data_type()));
                                out_row.push(Self::format_value(value)?);
                            }
                            Err(Error::InvalidColumnIndex(_)) => break,
                            Err(e) => return Err(e),
                        }
                    }
                    out_rows.push(out_row);
                }

                // If the query returned no rows, the statement has still been executed
                // at least once. Get the column count from the statement.
                if col_count == 0 {
                    col_count = rows
                        .as_ref()
                        .expect("rows should keep statement")
                        .column_count();
                    types = vec![DefaultColumnType::Any; col_count];
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
            Err(Error::InvalidQuery) => {
                let rows_changed = self.conn.execute(sql, [])? as u64;
                Ok(DBOutput::StatementComplete(rows_changed))
            }
            Err(e) => Err(e),
        }
    }
}
