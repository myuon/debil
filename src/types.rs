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

pub trait SQLTable {
    type ValueType;
    fn table_name(_: std::marker::PhantomData<Self>) -> String;
    fn schema_of(_: std::marker::PhantomData<Self>) -> Vec<(String, String, FieldAttribute)>;

    fn map_to_sql(self) -> Vec<(String, Self::ValueType)>;
    fn map_from_sql(_: std::collections::HashMap<String, Self::ValueType>) -> Self;
}

pub trait SQLValue<Type> {
    // Varchar type requires the size in the type representation, so we need size argument here
    fn column_type(_: std::marker::PhantomData<Self>, size: i32) -> String;

    fn serialize(self) -> Type;
    fn deserialize(_: Type) -> Self;
}
