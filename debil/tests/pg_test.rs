use debil::{postgres::*, PgTable};
use sqlx::{
    postgres::{PgArguments, PgPoolOptions, PgRow},
    query::Query,
    FromRow, Postgres, Row,
};

macro_rules! binds {
    ($q:expr,$e:expr,$($name:ident),* $(,)?) => {{
        let expr = $e;
        $q$(.bind(expr.$name))*
    }};
}

macro_rules! binds_cond {
    ($q:expr,$e:expr,$ns:expr,$($name:ident),* $(,)?) => {{
        let expr = $e;
        let mut query = $q;

        $(if $ns.contains(&stringify!($name)) {
            query = query.bind(expr.$name);
        })*

        query
    }};
}

macro_rules! partial {
    ($name:ident, {$($body:tt)*}) => {
        partial!(@wrapper, $name, $($body)*)
    };

    (@wrapper, $name:ident, $($body:tt)*) => {
        {
            let mut result = $name::default();
            let mut columns = vec![];

            partial!(@record_expr result, columns, $($body)*)
        }
    };

    (@record_expr $result:ident, $params:ident, $field:ident : $e:expr $(,)?) => {
        {
            $result.$field = $e;
            $params.push(stringify!($field));

            Partial {
                data: $result,
                columns: $params,
            }
        }
    };

    (@record_expr $result:ident, $params:ident, $field:ident : $e:expr, $($tails:tt)*) => {
        {
            $result.$field = $e;
            $params.push(stringify!($field));

            record_expr!(@record_expr $result, $params, $($tails)*)
        }
    };
}

pub trait BindQuery {
    fn binds<'q>(self, query: Query<'q, Postgres, PgArguments>)
        -> Query<'q, Postgres, PgArguments>;
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

    impl BindQuery for Test {
        fn binds<'q>(
            self,
            query: Query<'q, Postgres, PgArguments>,
        ) -> Query<'q, Postgres, PgArguments> {
            binds_Test!(query, self)
        }
    }

    impl BindQuery for Partial<Test> {
        fn binds<'q>(
            self,
            query: Query<'q, Postgres, PgArguments>,
        ) -> Query<'q, Postgres, PgArguments> {
            binds_cond_Test!(query, self.data, self.columns,)
        }
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

    t1.clone()
        .binds(sqlx::query(&format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table_name::<Test>(),
            column_names::<Test>().join(","),
            column_names::<Test>()
                .iter()
                .enumerate()
                .map(|(i, _)| format!("${}", i + 1))
                .collect::<Vec<_>>()
                .join(","),
        )))
        .execute(&pool)
        .await?;

    let one = sqlx::query_as::<_, Test>("SELECT * FROM test WHERE id = $1")
        .bind(t1.id.clone())
        .fetch_one(&pool)
        .await?;

    assert_eq!(one, t1);

    partial!(Test, {
        id: "updated".to_string(),
    })
    .binds(sqlx::query("UPDATE test SET name = $1 WHERE id = $2"))
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
