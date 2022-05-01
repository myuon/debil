use debil::{postgres::*, PgTable};
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    FromRow, Row,
};

#[tokio::test]
async fn test_table() -> Result<(), sqlx::Error> {
    #[derive(Debug, PartialEq, PgTable)]
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

    sqlx::query("INSERT INTO test (id, name, created_at) VALUES ($1, $2, $3)")
        .bind(t1.id.clone())
        .bind(t1.name.clone())
        .bind(t1.created_at.clone())
        .execute(&pool)
        .await?;

    let one = sqlx::query_as::<_, Test>("SELECT * FROM test WHERE id = $1")
        .bind(t1.id.clone())
        .fetch_one(&pool)
        .await?;

    assert_eq!(one, t1);

    sqlx::query(&drop_table_query::<Test>())
        .execute(&pool)
        .await?;

    Ok(())
}
