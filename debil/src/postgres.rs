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
