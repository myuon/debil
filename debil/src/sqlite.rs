use std::sync::{Arc, Mutex};

use crate as debil;
use crate::{HasNotFound, SQLConn, SQLValue};
use async_trait::async_trait;
use failure::Fail;

pub enum SqliteValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl SQLValue<()> for SqliteValue {
    fn column_type(_: std::marker::PhantomData<()>, _size: i32) -> String {
        "NULL".to_string()
    }

    fn serialize(_: ()) -> Self {
        SqliteValue::Null
    }

    fn deserialize(self) -> () {
        ()
    }
}

impl SQLValue<i64> for SqliteValue {
    fn column_type(_: std::marker::PhantomData<i64>, _size: i32) -> String {
        "INTEGER".to_string()
    }

    fn serialize(s: i64) -> Self {
        SqliteValue::Integer(s)
    }

    fn deserialize(self) -> i64 {
        match self {
            SqliteValue::Integer(s) => s,
            _ => panic!("Expected integer"),
        }
    }
}

impl SQLValue<f64> for SqliteValue {
    fn column_type(_: std::marker::PhantomData<f64>, _size: i32) -> String {
        "REAL".to_string()
    }

    fn serialize(s: f64) -> Self {
        SqliteValue::Real(s)
    }

    fn deserialize(self) -> f64 {
        match self {
            SqliteValue::Real(s) => s,
            _ => panic!("Expected real"),
        }
    }
}

impl SQLValue<String> for SqliteValue {
    fn column_type(_: std::marker::PhantomData<String>, _size: i32) -> String {
        "TEXT".to_string()
    }

    fn serialize(s: String) -> Self {
        SqliteValue::Text(s)
    }

    fn deserialize(self) -> String {
        match self {
            SqliteValue::Text(s) => s,
            _ => panic!("Expected text"),
        }
    }
}

impl SQLValue<Vec<u8>> for SqliteValue {
    fn column_type(_: std::marker::PhantomData<Vec<u8>>, _size: i32) -> String {
        "BLOB".to_string()
    }

    fn serialize(s: Vec<u8>) -> Self {
        SqliteValue::Blob(s)
    }

    fn deserialize(self) -> Vec<u8> {
        match self {
            SqliteValue::Blob(s) => s,
            _ => panic!("Expected blob"),
        }
    }
}

fn to_params(params: debil::Params<SqliteValue>) -> impl rusqlite::Params {
    if params.0.len() == 0 {
        &[]
    } else {
        params
            .0
            .into_iter()
            .map(|(k, v)| (&k, v.0))
            .collect::<Vec<(&str, &dyn rusqlite::ToSql)>>()
            .as_slice()
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "not_found")]
    NotFoundError,
    #[fail(display = "sqlite_error")]
    SqliteError(#[cause] rusqlite::Error),
    #[fail(display = "tokio_error")]
    TokioError(#[cause] tokio::task::JoinError),
}

impl HasNotFound for Error {
    fn not_found() -> Self {
        Self::NotFoundError
    }
}

/*
pub struct DebilConn {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

#[async_trait]
impl SQLConn<SqliteValue> for DebilConn {
    type Error = Error;

    async fn sql_exec(
        &mut self,
        query: String,
        params: debil::Params<SqliteValue>,
    ) -> Result<u64, Error> {
        let rows = tokio::task::spawn_blocking(move || {
            self.conn
                .lock()
                .unwrap()
                .execute(query.as_str(), to_params(params))
        })
        .await
        .map_err(|err| Error::TokioError(err))?
        .map_err(|err| Error::SqliteError(err))?;

        Ok(rows as u64)
    }

    async fn sql_query<T: debil::SQLMapper<ValueType = SqliteValue> + Sync + Send>(
        &mut self,
        query: String,
        params: debil::Params<SqliteValue>,
    ) -> Result<Vec<T>, Self::Error> {
        let result = self.conn.exec(query.as_str(), to_params(params)).await?;
        let vs = result
            .into_iter()
            .map(|row: mysql_async::Row| {
                let column_names = row
                    .columns_ref()
                    .iter()
                    .map(|c| c.name_str().into_owned())
                    .collect::<Vec<_>>();
                let values = row
                    .unwrap()
                    .into_iter()
                    .map(SqliteValue)
                    .collect::<Vec<_>>();

                debil::map_from_sql::<T>(
                    column_names
                        .into_iter()
                        .zip(values)
                        .collect::<std::collections::HashMap<_, _>>(),
                )
            })
            .collect();

        Ok(vs)
    }

    async fn sql_batch_exec(
        &mut self,
        query: String,
        params_vec: Vec<debil::Params<MySQLValue>>,
    ) -> Result<(), Self::Error> {
        self.conn
            .exec_batch(
                query.as_str(),
                params_vec.into_iter().map(to_params).collect::<Vec<_>>(),
            )
            .await?;

        Ok(())
    }
}
*/
