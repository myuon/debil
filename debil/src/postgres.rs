use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::{
    encode::IsNull,
    postgres::{PgArgumentBuffer, PgTypeInfo},
    Encode, PgPool, Postgres, Type,
};

pub enum PgType {
    Bool,
    Int,
    BigInt,
    VarChar,
    Text,
}

impl PgType {
    pub fn to_name(&self, size: Option<usize>) -> String {
        match self {
            PgType::Bool => "BOOL".to_string(),
            PgType::Int => "INT".to_string(),
            PgType::BigInt => "BIGINT".to_string(),
            PgType::VarChar => {
                if let Some(s) = size {
                    format!("VARCHAR({})", s)
                } else {
                    panic!("VarChar is selected, but size is not specified")
                }
            }
            PgType::Text => "TEXT".to_string(),
        }
    }
}

pub enum PgValue {
    Bool(bool),
    Int(i32),
    BigInt(i64),
    VarChar(String),
    Text(String),
}

impl<'q> Encode<'q, Postgres> for PgValue {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        match self {
            PgValue::Bool(b) => b.encode(buf),
            PgValue::Int(i) => i.encode(buf),
            PgValue::BigInt(i) => i.encode(buf),
            PgValue::VarChar(s) => s.encode(buf),
            PgValue::Text(t) => t.encode(buf),
        }
    }
}

impl Type<Postgres> for PgValue {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("")
    }
}

pub trait FromValues {
    fn from_values(values: HashMap<String, PgValue>) -> Self;
}

pub trait IntoValues {
    fn into_map_values(self) -> HashMap<String, PgValue>;
}

pub trait PgTypeSelector {
    fn select_type(_: std::marker::PhantomData<Self>) -> PgType;
}

macro_rules! select_type {
    ($t:ty,$e:expr) => {
        impl PgTypeSelector for $t {
            fn select_type(_: std::marker::PhantomData<Self>) -> PgType {
                $e
            }
        }
    };
}

select_type!(bool, PgType::Bool);
select_type!(i32, PgType::Int);
select_type!(i64, PgType::BigInt);
select_type!(String, PgType::VarChar);
select_type!(&str, PgType::VarChar);

pub struct FieldAttribute {
    pub size: Option<usize>,
    pub other: String,
}

pub trait PgTable {
    fn table_name(_: std::marker::PhantomData<Self>) -> &'static str;
    fn schema_of(_: std::marker::PhantomData<Self>) -> Vec<(&'static str, PgType, FieldAttribute)>;
}

pub fn table_name<T: PgTable>() -> &'static str {
    T::table_name(std::marker::PhantomData::<T>)
}

pub fn schema_of<T: PgTable>() -> Vec<(&'static str, PgType, FieldAttribute)> {
    T::schema_of(std::marker::PhantomData::<T>)
}

pub fn column_names<T: PgTable>() -> Vec<&'static str> {
    T::schema_of(std::marker::PhantomData::<T>)
        .into_iter()
        .map(|(name, _, _)| name)
        .collect()
}

pub fn create_table_query<T: PgTable>() -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        PgTable::table_name(std::marker::PhantomData::<T>),
        PgTable::schema_of(std::marker::PhantomData::<T>)
            .into_iter()
            .map(|v| format!("{} {} {}", v.0, v.1.to_name(v.2.size), v.2.other))
            .collect::<Vec<String>>()
            .join(",")
    )
}

pub fn drop_table_query<T: PgTable>() -> String {
    format!(
        "DROP TABLE IF EXISTS {}",
        PgTable::table_name(std::marker::PhantomData::<T>),
    )
}

pub struct Partial<T> {
    data: T,
    columns: Vec<&'static str>,
}

impl<T: PgTable> Partial<T> {
    pub fn full(data: T) -> Partial<T> {
        Partial {
            data,
            columns: column_names::<T>(),
        }
    }
}

pub struct QueryResult<T> {
    pub data: T,
    pub rows_affected: i64,
}

#[async_trait]
pub trait Executor {
    async fn execute_sql(&self, query: &str) -> Result<QueryResult<()>, sqlx::Error>;

    async fn create<T: PgTable + IntoValues + Send>(
        &self,
        item: T,
    ) -> Result<QueryResult<()>, sqlx::Error>;

    async fn update<T: PgTable + IntoValues + Send>(
        &self,
        item: Partial<T>,
    ) -> Result<QueryResult<()>, sqlx::Error>;

    async fn save<T: PgTable + IntoValues + Send + Sync + Clone>(
        &self,
        item: T,
    ) -> Result<QueryResult<()>, sqlx::Error> {
        let mut result = self.update(Partial::full(item.clone())).await?;
        if result.rows_affected == 0 {
            result = self.create(item).await?;
        }

        Ok(result)
    }
}
