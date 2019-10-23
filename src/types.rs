#[derive(Clone, Debug, PartialEq)]
pub struct FieldAttribute {
    pub size: Option<i32>,
    pub unique: Option<bool>,
    pub not_null: Option<bool>,
    pub primary_key: Option<bool>,
}

impl Default for FieldAttribute {
    fn default() -> Self {
        FieldAttribute {
            size: None,
            unique: None,
            not_null: None,
            primary_key: None,
        }
    }
}

pub trait SQLTable: Sized {
    type ValueType;
    fn table_name(_: std::marker::PhantomData<Self>) -> String;
    fn schema_of(_: std::marker::PhantomData<Self>) -> Vec<(String, String, FieldAttribute)>;

    fn map_to_sql(self) -> Vec<(String, Self::ValueType)>;
    fn map_from_sql(_: std::collections::HashMap<String, Self::ValueType>) -> Self;

    fn create_table_query(ty: std::marker::PhantomData<Self>) -> String {
        let schema = SQLTable::schema_of(ty);

        format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            SQLTable::table_name(ty),
            schema
                .into_iter()
                .map(|(name, typ, attr)| {
                    [
                        &[name.as_str(), typ.as_str()],
                        vec![
                            if attr.primary_key.unwrap_or(false) {
                                Some("PRIMARY KEY")
                            } else {
                                None
                            },
                            if attr.unique.unwrap_or(false) {
                                Some("UNIQUE")
                            } else {
                                None
                            },
                            if attr.not_null.unwrap_or(false) {
                                Some("NOT NULL")
                            } else {
                                None
                            },
                        ]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .as_slice(),
                    ]
                    .concat()
                    .join(" ")
                })
                .collect::<Vec<_>>()
                .as_slice()
                .join(", ")
        )
    }

    fn save_query_with_params(self) -> (String, Vec<(String, Self::ValueType)>) {
        let pairs = self.map_to_sql();
        let keys = pairs.iter().map(|(k, _)| k).collect::<Vec<_>>();

        (
            format!(
                "INSERT INTO {} ({}) VALUES ({})",
                Self::table_name(std::marker::PhantomData::<Self>),
                keys.iter()
                    .map(|k| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                keys.iter()
                    .map(|s| format!(":{}", s))
                    .collect::<Vec<_>>()
                    .as_slice()
                    .join(", "),
            ),
            pairs,
        )
    }
}

pub trait SQLValue<Type> {
    // Varchar type requires the size in the type representation, so we need size argument here
    fn column_type(_: std::marker::PhantomData<Self>, size: i32) -> String;

    fn serialize(self) -> Type;
    fn deserialize(_: Type) -> Self;
}
