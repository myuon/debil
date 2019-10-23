use debil::binary::*;
use debil::*;

#[derive(Table, PartialEq, Debug)]
#[sql(table_name = "ex_1", sql_type = "Vec<u8>")]
struct Ex1 {
    #[sql(size = 50)]
    field1: String,
    aaaa: i32,
}

#[test]
fn it_derives_sql_table() {
    let ex1 = Ex1 {
        field1: "aaa".to_string(),
        aaaa: 10,
    };

    assert_eq!(
        SQLTable::table_name(std::marker::PhantomData::<Ex1>),
        "ex_1"
    );
    assert_eq!(
        SQLTable::schema_of(std::marker::PhantomData::<Ex1>),
        vec![
            (
                "field1".to_string(),
                "varchar(50)".to_string(),
                FieldAttribute {
                    size: Some(50),
                    ..Default::default()
                }
            ),
            ("aaaa".to_string(), "int".to_string(), Default::default())
        ]
    );

    assert_eq!(
        ex1.map_to_sql(),
        vec![
            ("field1".to_string(), SQLValue::serialize("aaa".to_string())),
            ("aaaa".to_string(), SQLValue::serialize(10))
        ]
    );

    let ex2: Ex1 = SQLTable::map_from_sql(
        vec![
            (
                "field1".to_string(),
                SQLValue::serialize("piyo".to_string()),
            ),
            ("aaaa".to_string(), SQLValue::serialize(-10000)),
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(
        ex2,
        Ex1 {
            field1: "piyo".to_string(),
            aaaa: -10000,
        }
    )
}
