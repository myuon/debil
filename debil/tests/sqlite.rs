#[cfg(feature = "sqlite")]
mod tests {
    use anyhow::Result;
    use debil::sqlite::*;
    use debil::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_create_table() -> Result<(), Error> {
        #[derive(Table, PartialEq, Debug)]
        #[sql(table_name = "test", sql_type = "SqliteValue", primary_key = "id")]
        struct Test {
            id: i64,
            name: String,
        }

        let mut conn = DebilConn::new(rusqlite::Connection::open_in_memory().unwrap());
        conn.sql_exec(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)".to_string(),
            Params(vec![]),
        )
        .await?;

        conn.sql_exec(
            "INSERT INTO test VALUES (:id, :name)".to_string(),
            Params(vec![
                (":id".to_string(), SqlValue::serialize(100)),
                (":name".to_string(), SqlValue::serialize("foo".to_string())),
            ]),
        )
        .await?;

        let rs = conn
            .sql_query::<Test>(
                "SELECT * FROM test WHERE id = :id".to_string(),
                Params(vec![(":id".to_string(), SqlValue::serialize(100))]),
            )
            .await?;
        assert_eq!(
            rs,
            vec![Test {
                id: 100,
                name: "foo".to_string(),
            }]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_create_and_drop_index() -> Result<()> {
        let conn = rusqlite::Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE person (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER NOT NULL
        )",
            [],
        )?;

        // test: create_index should be executed
        conn.execute(&create_index("person", "name", &["name"]), [])?;

        // test: create_index should be executed even if the index already exists
        conn.execute(&create_index("person", "name", &["name"]), [])?;

        // test: drop_index should be executed
        conn.execute(&drop_index("person", "name"), [])?;

        // test: drop_index should be executed even if the index does not exists
        conn.execute(&drop_index("person", "name"), [])?;

        Ok(())
    }
}
