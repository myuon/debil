#[cfg(feature = "mysql")]
mod tests {
    use debil::mysql::*;
    use debil::*;
    use mysql_async::OptsBuilder;

    #[derive(Table)]
    #[sql(table_name = "migration_test", primary_key = "pk")]
    struct Before {
        n: i32,
        #[sql(size = 10)]
        pk: String,
        still_remaining: i32,
    }

    #[derive(Table)]
    #[sql(table_name = "migration_test", primary_key = "pk")]
    struct After {
        n: i64,
        #[sql(size = 100)]
        extra: String,
        #[sql(size = 11)]
        pk: String,
    }

    #[tokio::test]
    async fn it_should_migrate() -> Result<(), Error> {
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

        // setup
        conn.drop_table::<After>().await?;

        // migration creates table
        conn.migrate::<Before>().await?;

        // migrate
        conn.migrate::<After>().await?;

        Ok(())
    }
}
