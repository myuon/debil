use crate::mysql::error::Error;
use crate::mysql::types::MySQLValue;
use async_trait::async_trait;
use crate as debil;
use crate::conn::SQLConn;
use mysql_async::prelude::*;

pub struct DebilConn {
    conn: mysql_async::Conn,
}

impl debil::HasNotFound for Error {
    fn not_found() -> Self {
        Error::NotFoundError
    }
}

fn to_params(params: debil::Params<MySQLValue>) -> params::Params {
    if params.0.len() == 0 {
        params::Params::Empty
    } else {
        params
            .0
            .into_iter()
            .map(|(k, v)| (k, v.0))
            .collect::<Vec<_>>()
            .into()
    }
}

#[async_trait]
impl debil::SQLConn<MySQLValue> for DebilConn {
    type Error = Error;

    async fn sql_exec(
        &mut self,
        query: String,
        params: debil::Params<MySQLValue>,
    ) -> Result<u64, Error> {
        self.conn
            .exec_drop(query.as_str(), to_params(params))
            .await?;

        Ok(self.conn.affected_rows())
    }

    async fn sql_query<T: debil::SQLMapper<ValueType = MySQLValue> + Sync + Send>(
        &mut self,
        query: String,
        params: debil::Params<MySQLValue>,
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
                let values = row.unwrap().into_iter().map(MySQLValue).collect::<Vec<_>>();

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

impl DebilConn {
    pub fn as_conn(self) -> mysql_async::Conn {
        self.conn
    }

    pub fn from_conn(conn: mysql_async::Conn) -> Self {
        DebilConn { conn }
    }

    pub async fn sql_query_with_map<U>(
        &mut self,
        query: impl AsRef<str>,
        parameters: impl Into<params::Params>,
        mapper: impl FnMut(mysql_async::Row) -> U,
    ) -> Result<Vec<U>, Error> {
        let result = self.conn.exec(query.as_ref(), parameters.into()).await?;

        Ok(result.into_iter().map(mapper).collect())
    }

    pub async fn drop_table<T: debil::SQLTable<ValueType = MySQLValue> + Sync + Send>(
        &mut self,
    ) -> Result<(), Error> {
        self.sql_exec(
            format!(
                "DROP TABLE IF EXISTS {}",
                debil::SQLTable::table_name(std::marker::PhantomData::<T>),
            ),
            debil::Params::<MySQLValue>::new(),
        )
        .await?;

        Ok(())
    }

    pub async fn migrate<T: debil::SQLTable<ValueType = MySQLValue> + Sync + Send>(
        &mut self,
    ) -> Result<(), Error> {
        self.create_table::<T>().await?;

        let table_name = debil::SQLTable::table_name(std::marker::PhantomData::<T>);
        let schema = debil::SQLTable::schema_of(std::marker::PhantomData::<T>);

        for (column_name, column_type, attr) in schema {
            let vs = self.sql_query_with_map("SELECT DATA_TYPE, COLUMN_TYPE, IS_NULLABLE, COLUMN_KEY FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = :table_name AND COLUMN_NAME = :column_name", mysql_async::params!{
                "table_name" => table_name.clone(),
                "column_name" => column_name.clone(),
            }, mysql_async::from_row::<(String, String, String, String)>).await?;

            if vs.is_empty() {
                self.sql_exec(
                    format!(
                        "ALTER TABLE {} ADD COLUMN {}",
                        table_name,
                        debil::create_column_query(column_name, column_type, attr)
                    ),
                    debil::Params::<MySQLValue>::new(),
                )
                .await?;
            } else if (vs[0].0 != column_type && vs[0].1 != column_type)
                || (attr.not_null.unwrap_or(false) != (vs[0].2 == "NO"))
                || (attr.unique.unwrap_or(false) != (vs[0].3 == "UNI"))
            {
                // check not only DATA_TYPE but also COLUMN_TYPE (for varchar)
                self.sql_exec(
                    format!(
                        "ALTER TABLE {} MODIFY COLUMN {}",
                        table_name,
                        debil::create_column_query(column_name, column_type, attr)
                    ),
                    debil::Params::<MySQLValue>::new(),
                )
                .await?;
            }
        }

        Ok(())
    }

    pub async fn create_all<T: debil::SQLTable<ValueType = MySQLValue> + Clone>(
        &mut self,
        datas: Vec<T>,
    ) -> Result<(), Error> {
        if datas.len() == 0 {
            return Ok(());
        }

        let (query, _) = datas[0].clone().insert_query_with_params();
        let mut parameters = Vec::new();
        for data in datas {
            let (_, ps) = data.insert_query_with_params();
            parameters.push(debil::Params(ps));
        }

        self.sql_batch_exec(query, parameters).await?;

        Ok(())
    }

    pub async fn start_transaction(&mut self) -> Result<(), Error> {
        self.conn
            .query_drop("START TRANSACTION".to_string())
            .await?;

        Ok(())
    }

    pub async fn commit(&mut self) -> Result<(), Error> {
        self.conn.query_drop("COMMIT".to_string()).await?;

        Ok(())
    }

    pub async fn rollback(&mut self) -> Result<(), Error> {
        self.conn.query_drop("ROLLBACK".to_string()).await?;

        Ok(())
    }
}
