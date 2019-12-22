use debil::*;

#[derive(Table, PartialEq, Debug, Clone)]
#[sql(table_name = "ex_1", sql_type = "Vec<u8>", primary_key_columns = "pk")]
struct Ex1 {
    #[sql(size = 50, unique = true, not_null = true)]
    field1: String,
    aaaa: i32,
    #[sql(primary_key = true)]
    pk: i32,
}

#[test]
fn it_derives_sql_table() {
    let ex1 = Ex1 {
        field1: "aaa".to_string(),
        aaaa: 10,
        pk: 1,
    };

    assert_eq!(table_name::<Ex1>(), "ex_1");
    assert_eq!(primary_key_columns::<Ex1>(), vec!["pk"]);
    assert_eq!(
        schema_of::<Ex1>(),
        vec![
            (
                "field1".to_string(),
                "varchar(50)".to_string(),
                FieldAttribute {
                    size: Some(50),
                    unique: Some(true),
                    not_null: Some(true),
                    ..Default::default()
                }
            ),
            ("aaaa".to_string(), "int".to_string(), Default::default()),
            (
                "pk".to_string(),
                "int".to_string(),
                FieldAttribute {
                    primary_key: Some(true),
                    ..Default::default()
                }
            ),
        ]
    );

    assert_eq!(
        ex1.clone().map_to_sql(),
        vec![
            ("field1".to_string(), SQLValue::serialize("aaa".to_string())),
            ("aaaa".to_string(), SQLValue::serialize(10)),
            ("pk".to_string(), SQLValue::serialize(1))
        ]
    );
    assert_eq!(
        ex1.clone().save_query_with_params().0,
        "INSERT INTO ex_1 (field1, aaaa, pk) VALUES (:field1, :aaaa, :pk)"
    );

    let ex2 = map_from_sql::<Ex1>(
        vec![
            (
                "field1".to_string(),
                SQLValue::serialize("piyo".to_string()),
            ),
            ("aaaa".to_string(), SQLValue::serialize(-10000)),
            ("pk".to_string(), SQLValue::serialize(200)),
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(
        ex2,
        Ex1 {
            field1: "piyo".to_string(),
            aaaa: -10000,
            pk: 200,
        }
    );

    assert_eq!(
        SQLTable::create_table_query(std::marker::PhantomData::<Ex1>),
        "CREATE TABLE IF NOT EXISTS ex_1 (field1 varchar(50) UNIQUE NOT NULL, aaaa int, pk int PRIMARY KEY)"
    );
}
