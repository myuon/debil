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

pub fn create_column_query(
    column_name: String,
    column_type: String,
    attr: FieldAttribute,
) -> String {
    [
        &[column_name.as_str(), column_type.as_str()],
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
}

pub trait SQLMapper: Sized {
    type ValueType;
    fn map_from_sql(_: std::collections::HashMap<String, Self::ValueType>) -> Self;
}

pub trait SQLTable: SQLMapper {
    fn table_name(_: std::marker::PhantomData<Self>) -> String;
    fn schema_of(_: std::marker::PhantomData<Self>) -> Vec<(String, String, FieldAttribute)>;

    fn primary_key_columns(_: std::marker::PhantomData<Self>) -> Vec<String>;

    fn add_primary_key_query(ty: std::marker::PhantomData<Self>) -> String {
        let columns = SQLTable::primary_key_columns(ty);
        format!(
            "ALTER TABLE {} ADD PRIMARY KEY({})",
            SQLTable::table_name(ty),
            columns.join(",")
        )
    }

    fn map_to_sql(self) -> Vec<(String, Self::ValueType)>;

    fn create_table_query(ty: std::marker::PhantomData<Self>) -> String {
        let schema = SQLTable::schema_of(ty);

        format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            SQLTable::table_name(ty),
            schema
                .into_iter()
                .map(|(name, typ, attr)| create_column_query(name, typ, attr))
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

pub fn table_name<T: SQLTable>() -> String {
    SQLTable::table_name(std::marker::PhantomData::<T>)
}

pub fn schema_of<T: SQLTable>() -> Vec<(String, String, FieldAttribute)> {
    SQLTable::schema_of(std::marker::PhantomData::<T>)
}

pub fn primary_key_columns<T: SQLTable>() -> Vec<String> {
    SQLTable::primary_key_columns(std::marker::PhantomData::<T>)
}

pub fn map_from_sql<T: SQLMapper>(h: std::collections::HashMap<String, T::ValueType>) -> T {
    SQLMapper::map_from_sql(h)
}

pub fn create_table_query<T: SQLTable>() -> String {
    SQLTable::create_table_query(std::marker::PhantomData::<T>)
}

pub trait SQLValue<Type> {
    // Varchar type requires the size in the type representation, so we need size argument here
    fn column_type(_: std::marker::PhantomData<Type>, size: i32) -> String;

    fn serialize(_: Type) -> Self;
    fn deserialize(self) -> Type;
}
