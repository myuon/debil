use debil::*;

#[derive(Table, PartialEq, Debug, Clone)]
#[sql(table_name = "ex_1", sql_type = "Vec<u8>", primary_key = "pk")]
struct Ex1 {
    #[sql(size = 50, unique = true, not_null = true)]
    field1: String,
    aaaa: i32,
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
        ex1.clone().insert_query_with_params().0,
        "INSERT INTO ex_1 (field1, aaaa, pk) VALUES (:field1, :aaaa, :pk)"
    );
    assert_eq!(
        ex1.clone().update_query_with_params().0,
        "UPDATE ex_1 SET field1 = :field1, aaaa = :aaaa, pk = :pk WHERE pk = :pk"
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
        "CREATE TABLE IF NOT EXISTS ex_1 (field1 varchar(50) UNIQUE NOT NULL, aaaa int, pk int, CONSTRAINT primary_key PRIMARY KEY(pk))"
    );

    assert_eq!(
        SQLTable::constraint_primary_key_query(std::marker::PhantomData::<Ex1>),
        "CONSTRAINT primary_key PRIMARY KEY(pk)"
    )
}

#[test]
fn composite_primary_key() {
    #[derive(Table, PartialEq, Debug, Clone)]
    #[sql(table_name = "ex_1", sql_type = "Vec<u8>", primary_key = "pk,pk2")]
    struct Ex2 {
        #[sql(size = 50, unique = true, not_null = true)]
        field1: String,
        aaaa: i32,
        pk: i32,
        pk2: i32,
    }

    assert_eq!(
        SQLTable::constraint_primary_key_query(std::marker::PhantomData::<Ex2>),
        "CONSTRAINT primary_key PRIMARY KEY(pk,pk2)"
    );

    #[derive(Table, PartialEq, Debug, Clone)]
    #[sql(table_name = "ex_1", sql_type = "Vec<u8>", primary_key = "pk ,pk2")]
    struct Ex3 {
        #[sql(size = 50, unique = true, not_null = true)]
        field1: String,
        aaaa: i32,
        pk: i32,
        pk2: i32,
    }
    assert_eq!(primary_key_columns::<Ex3>(), vec!["pk", "pk2"]);
}

#[test]
fn add_index() {
    #[derive(Table, PartialEq, Debug, Clone)]
    #[sql(table_name = "ex_1", sql_type = "Vec<u8>", primary_key = "pk,pk2")]
    struct Ex4 {
        #[sql(size = 50, unique = true, not_null = true)]
        field1: String,
        aaaa: i32,
        pk: i32,
        pk2: i32,
    }

    assert_eq!(
        create_unique_index_query::<Ex4>("hoge", vec!["aaaa"]),
        "CREATE UNIQUE INDEX IF NOT EXISTS hoge ON ex_1(aaaa);"
    );

    assert_eq!(
        create_index_query::<Ex4>("hoge", vec!["aaaa"]),
        "CREATE INDEX IF NOT EXISTS hoge ON ex_1(aaaa);"
    );
}

#[test]
#[should_panic]
fn add_index_key_not_found() {
    #[derive(Table, PartialEq, Debug, Clone)]
    #[sql(table_name = "ex_1", sql_type = "Vec<u8>", primary_key = "pk,pk2")]
    struct Ex5 {
        #[sql(size = 50, unique = true, not_null = true)]
        field1: String,
        aaaa: i32,
        pk: i32,
        pk2: i32,
    }
    create_index_query::<Ex5>("hoge", vec!["field5"]);
}

#[derive(Accessor)]
struct Foo {
    hoge: i32,
    piyo: String,
}

#[test]
fn test_accessor() {
    assert_eq!(Foo::hoge(), "hoge");
    assert_eq!(Foo::piyo(), "piyo");
}
