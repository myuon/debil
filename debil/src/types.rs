#[derive(Clone, Debug, PartialEq)]
pub struct FieldAttribute {
    pub size: Option<i32>,
    pub unique: Option<bool>,
    pub not_null: Option<bool>,
}

impl Default for FieldAttribute {
    fn default() -> Self {
        FieldAttribute {
            size: None,
            unique: None,
            not_null: None,
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

pub trait SqlMapper: Sized {
    type ValueType: Clone;
    fn map_from_sql(_: std::collections::HashMap<String, Self::ValueType>) -> Self;
}

pub trait SqlTable: SqlMapper {
    fn table_name(_: std::marker::PhantomData<Self>) -> String;
    fn schema_of(_: std::marker::PhantomData<Self>) -> Vec<(String, String, FieldAttribute)>;

    fn primary_key_columns(_: std::marker::PhantomData<Self>) -> Vec<String>;

    fn constraint_primary_key_query(ty: std::marker::PhantomData<Self>) -> String {
        let columns = SqlTable::primary_key_columns(ty);
        format!("CONSTRAINT primary_key PRIMARY KEY({})", columns.join(","))
    }

    fn map_to_sql(self) -> Vec<(String, Self::ValueType)>;

    fn create_index_query(
        ty: std::marker::PhantomData<Self>,
        index_name: &'static str,
        index_keys: Vec<&'static str>,
    ) -> String {
        let schema = SqlTable::schema_of(ty);
        let table_name = SqlTable::table_name(ty);
        // search if specified keys exist
        for key in index_keys.iter() {
            if !schema
                .iter()
                .map(|(column_name, _, _)| column_name.as_str())
                .collect::<Vec<&str>>()
                .contains(key)
            {
                panic!("index: column {} is not field of {}", key, table_name)
            }
        }
        format!(
            "CREATE INDEX IF NOT EXISTS {} ON {}({});",
            index_name,
            table_name,
            index_keys.join(","),
        )
    }

    fn create_unique_index_query(
        ty: std::marker::PhantomData<Self>,
        index_name: &'static str,
        index_keys: Vec<&'static str>,
    ) -> String {
        let schema = SqlTable::schema_of(ty);
        let table_name = SqlTable::table_name(ty);
        // search if specified keys exist
        for key in index_keys.iter() {
            if !schema
                .iter()
                .map(|(column_name, _, _)| column_name.as_str())
                .collect::<Vec<&str>>()
                .contains(key)
            {
                panic!("index: column {} is not field of {}", key, table_name)
            }
        }

        format!(
            "CREATE UNIQUE INDEX IF NOT EXISTS {} ON {}({});",
            index_name,
            table_name,
            index_keys.join(","),
        )
    }

    fn create_table_query(ty: std::marker::PhantomData<Self>) -> String {
        let schema = SqlTable::schema_of(ty);

        format!(
            "CREATE TABLE IF NOT EXISTS {} ({}, {})",
            SqlTable::table_name(ty),
            schema
                .into_iter()
                .map(|(name, typ, attr)| create_column_query(name, typ, attr))
                .collect::<Vec<_>>()
                .as_slice()
                .join(", "),
            SqlTable::constraint_primary_key_query(ty),
        )
    }

    fn insert_query_with_params(self) -> (String, Vec<(String, Self::ValueType)>) {
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

    fn update_query_with_params(self) -> (String, Vec<(String, Self::ValueType)>) {
        let pairs = self.map_to_sql();
        let keys = pairs.iter().map(|(k, _)| k).collect::<Vec<_>>();
        let primary_keys_cond = Self::primary_key_columns(std::marker::PhantomData::<Self>)
            .into_iter()
            .map(|v| format!("{} = :{}", v, v))
            .collect::<Vec<_>>()
            .join(" and ");

        (
            format!(
                "UPDATE {} SET {} WHERE {}",
                Self::table_name(std::marker::PhantomData::<Self>),
                keys.iter()
                    .map(|k| format!("{} = :{}", k, k))
                    .collect::<Vec<_>>()
                    .join(", "),
                primary_keys_cond
            ),
            pairs,
        )
    }
}

pub fn table_name<T: SqlTable>() -> String {
    SqlTable::table_name(std::marker::PhantomData::<T>)
}

pub fn schema_of<T: SqlTable>() -> Vec<(String, String, FieldAttribute)> {
    SqlTable::schema_of(std::marker::PhantomData::<T>)
}

pub fn primary_key_columns<T: SqlTable>() -> Vec<String> {
    SqlTable::primary_key_columns(std::marker::PhantomData::<T>)
}

pub fn create_index_query<T: SqlTable>(
    index_name: &'static str,
    index_keys: Vec<&'static str>,
) -> String {
    SqlTable::create_index_query(std::marker::PhantomData::<T>, index_name, index_keys)
}

pub fn create_unique_index_query<T: SqlTable>(
    index_name: &'static str,
    index_keys: Vec<&'static str>,
) -> String {
    SqlTable::create_unique_index_query(std::marker::PhantomData::<T>, index_name, index_keys)
}

pub fn map_from_sql<T: SqlMapper>(h: std::collections::HashMap<String, T::ValueType>) -> T {
    SqlMapper::map_from_sql(h)
}

pub fn create_table_query<T: SqlTable>() -> String {
    SqlTable::create_table_query(std::marker::PhantomData::<T>)
}

pub trait SqlValue<Type> {
    // Varchar type requires the size in the type representation, so we need size argument here
    fn column_type(_: std::marker::PhantomData<Type>, size: i32) -> String;

    fn serialize(_: Type) -> Self;
    fn deserialize(self) -> Type;
}
