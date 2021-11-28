#[cfg(feature = "mysql")]
mod tests {

    use debil::mysql::*;
    use debil::*;
    use mysql_async::OptsBuilder;

    #[derive(Clone)]
    struct R {
        s: String,
        n: i32,
    }

    // custom implementation
    impl SQLMapper for R {
        type ValueType = MySQLValue;

        fn map_from_sql(values: std::collections::HashMap<String, Self::ValueType>) -> Self {
            R {
                s: <Self::ValueType as SQLValue<String>>::deserialize(values["s"].clone()),
                n: <Self::ValueType as SQLValue<i32>>::deserialize(values["n"].clone()),
            }
        }
    }

    impl SQLTable for R {
        fn table_name(_: std::marker::PhantomData<Self>) -> String {
            "r_table".to_string()
        }

        fn schema_of(_: std::marker::PhantomData<Self>) -> Vec<(String, String, FieldAttribute)> {
            vec![
                (
                    "s".to_string(),
                    "varchar(50)".to_string(),
                    Default::default(),
                ),
                ("n".to_string(), "int".to_string(), Default::default()),
            ]
        }

        fn primary_key_columns(_: std::marker::PhantomData<Self>) -> Vec<String> {
            vec!["s".to_string(), "n".to_string()]
        }

        fn map_to_sql(self) -> Vec<(String, Self::ValueType)> {
            let mut result = Vec::new();
            result.push((
                "s".to_string(),
                <Self::ValueType as SQLValue<String>>::serialize(self.s),
            ));
            result.push((
                "n".to_string(),
                <Self::ValueType as SQLValue<i32>>::serialize(self.n),
            ));

            result
        }
    }

    #[tokio::test]
    async fn migrate_save_and_select() -> Result<(), mysql_async::Error> {
        let raw_conn = mysql_async::Conn::new(
            OptsBuilder::default()
                .ip_or_hostname("127.0.0.1")
                .user(Some("root"))
                .pass(Some("password"))
                .db_name(Some("db"))
                .prefer_socket(Some(false))
                .pool_opts(Some(mysql_async::PoolOpts::default().with_constraints(
                    mysql_async::PoolConstraints::new(1, 1).unwrap(),
                )))
                .clone(),
        )
        .await?;
        let mut conn = DebilConn::from_conn(raw_conn);

        conn.create_table::<R>().await.unwrap();

        // This is not working for update (#7)
        conn.save(R {
            s: "foo".to_string(),
            n: 100,
        })
        .await
        .unwrap();

        // check thread safety
        tokio::spawn(conn_load::<R>(conn)).await.unwrap();

        Ok(())
    }

    async fn conn_load<R: debil::SQLTable<ValueType = MySQLValue> + Sync + Send>(
        mut conn: DebilConn,
    ) {
        conn.load::<R>(QueryBuilder::new()).await.unwrap();
        conn.first::<R>(QueryBuilder::new()).await.unwrap();
    }
}
