use debil::{postgres::*, PgTable};
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    FromRow, Row,
};

macro_rules! binds {
    ($q:expr,$e:expr,$($name:ident),* $(,)?) => {
        $q$(.bind($e.$name))*
    };
}

#[tokio::test]
async fn test_table() -> Result<(), sqlx::Error> {
    #[derive(Debug, PartialEq, PgTable, Clone, Default)]
    #[sql(table_name = "test")]
    struct Test {
        #[sql(size = 256)]
        id: String,
        #[sql(size = 1024)]
        name: String,
        created_at: i64,
    }

    let pool = PgPoolOptions::new()
        .connect("postgres://postgres:password@localhost/test")
        .await?;

    sqlx::query(&create_table_query::<Test>())
        .execute(&pool)
        .await?;

    let t1 = Test {
        id: "1".to_string(),
        name: "test1".to_string(),
        created_at: 1,
    };

    binds_Test!(
        sqlx::query(&format!(
            "INSERT INTO {} ({}) VALUES ($1, $2, $3)",
            table_name::<Test>(),
            column_names::<Test>().join(","),
        )),
        t1.clone(),
    )
    .execute(&pool)
    .await?;

    let one = sqlx::query_as::<_, Test>("SELECT * FROM test WHERE id = $1")
        .bind(t1.id.clone())
        .fetch_one(&pool)
        .await?;

    assert_eq!(one, t1);

    binds!(
        sqlx::query("UPDATE test SET name = $1 WHERE id = $2"),
        Test {
            name: "updated".to_string(),
            ..Default::default()
        },
        name,
    )
    .bind(t1.id.clone())
    .execute(&pool)
    .await?;

    let two = sqlx::query_as::<_, Test>("SELECT * FROM test WHERE id = $1")
        .bind(t1.id.clone())
        .fetch_one(&pool)
        .await?;

    assert_eq!(two.name, "updated");

    sqlx::query(&drop_table_query::<Test>())
        .execute(&pool)
        .await?;

    Ok(())
}
