use crate as debil;
use crate::{HasNotFound, SqlConn, SqlValue};
use async_trait::async_trait;

pub struct SqliteValue(rusqlite::types::Value);

impl rusqlite::ToSql for SqliteValue {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl SqlValue<()> for SqliteValue {
    fn column_type(_: std::marker::PhantomData<()>, _size: i32) -> String {
        "NULL".to_string()
    }

    fn serialize(_: ()) -> Self {
        SqliteValue(rusqlite::types::Value::Null)
    }

    fn deserialize(self) -> () {
        ()
    }
}

impl SqlValue<i64> for SqliteValue {
    fn column_type(_: std::marker::PhantomData<i64>, _size: i32) -> String {
        "INTEGER".to_string()
    }

    fn serialize(s: i64) -> Self {
        SqliteValue(rusqlite::types::Value::Integer(s))
    }

    fn deserialize(self) -> i64 {
        match self {
            SqliteValue(rusqlite::types::Value::Integer(s)) => s,
            _ => panic!("SqliteValue::deserialize: invalid type"),
        }
    }
}

impl SqlValue<f64> for SqliteValue {
    fn column_type(_: std::marker::PhantomData<f64>, _size: i32) -> String {
        "REAL".to_string()
    }

    fn serialize(s: f64) -> Self {
        SqliteValue(rusqlite::types::Value::Real(s))
    }

    fn deserialize(self) -> f64 {
        match self {
            SqliteValue(rusqlite::types::Value::Real(s)) => s,
            _ => panic!("SqliteValue::deserialize: invalid type"),
        }
    }
}

impl SqlValue<String> for SqliteValue {
    fn column_type(_: std::marker::PhantomData<String>, _size: i32) -> String {
        "TEXT".to_string()
    }

    fn serialize(s: String) -> Self {
        SqliteValue(rusqlite::types::Value::Text(s))
    }

    fn deserialize(self) -> String {
        match self {
            SqliteValue(rusqlite::types::Value::Text(s)) => s,
            _ => panic!("SqliteValue::deserialize: invalid type"),
        }
    }
}

impl SqlValue<Vec<u8>> for SqliteValue {
    fn column_type(_: std::marker::PhantomData<Vec<u8>>, _size: i32) -> String {
        "BLOB".to_string()
    }

    fn serialize(s: Vec<u8>) -> Self {
        SqliteValue(rusqlite::types::Value::Blob(s))
    }

    fn deserialize(self) -> Vec<u8> {
        match self {
            SqliteValue(rusqlite::types::Value::Blob(s)) => s,
            _ => panic!("SqliteValue::deserialize: invalid type"),
        }
    }
}

fn to_params(params: &debil::Params<SqliteValue>) -> Vec<(&str, &dyn rusqlite::ToSql)> {
    if params.0.len() == 0 {
        vec![]
    } else {
        params
            .0
            .iter()
            .map(|(k, v)| (k.as_str(), &v.0 as &dyn rusqlite::ToSql))
            .collect::<Vec<_>>()
    }
}

#[derive(Debug)]
pub enum Error {
    NotFoundError,
    SqliteError(rusqlite::Error),
    TokioError(tokio::task::JoinError),
}

impl HasNotFound for Error {
    fn not_found() -> Self {
        Self::NotFoundError
    }
}

pub struct DebilConn {
    conn: rusqlite::Connection,
}

// This impl uses tokio::task::block_in_place, which could lead to a problem in some specific situations;
#[async_trait]
impl SqlConn<SqliteValue> for DebilConn {
    type Error = Error;

    async fn sql_exec(
        &mut self,
        query: String,
        params: debil::Params<SqliteValue>,
    ) -> Result<u64, Error> {
        let rows = tokio::task::block_in_place(move || {
            self.conn
                .execute(query.as_str(), to_params(&params).as_slice())
        })
        .map_err(|err| Error::SqliteError(err))?;

        Ok(rows as u64)
    }

    async fn sql_query<T: debil::SqlMapper<ValueType = SqliteValue> + Sync + Send>(
        &mut self,
        query: String,
        params: debil::Params<SqliteValue>,
    ) -> Result<Vec<T>, Self::Error> {
        let vs = tokio::task::block_in_place(move || {
            self.conn
                .query_row(query.as_str(), to_params(&params).as_slice(), |row| todo!())
        })
        .map_err(|err| Error::SqliteError(err))?;

        Ok(vs)
    }

    async fn sql_batch_exec(
        &mut self,
        query: String,
        params_vec: Vec<debil::Params<SqliteValue>>,
    ) -> Result<(), Self::Error> {
        for params in params_vec {
            self.sql_exec(query.clone(), params).await?;
        }

        Ok(())
    }
}
