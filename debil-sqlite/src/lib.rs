use debil::*;
use sqlite::*;

pub struct SqliteValue(Value);

impl SQLValue<String> for SqliteValue {
    fn column_type(_: std::marker::PhantomData<String>, size: i32) -> String {
        "TEXT".to_string()
    }

    fn serialize(t: String) -> Self {
        SqliteValue(Value::String(t))
    }

    fn deserialize(self) -> String {
        self.0.as_string().unwrap().to_string()
    }
}
