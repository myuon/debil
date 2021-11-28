use crate::SQLValue;
use failure::_core::marker::PhantomData;

#[derive(Clone)]
pub struct MySQLValue(pub mysql_async::Value);

impl SQLValue<bool> for MySQLValue {
    fn column_type(_: PhantomData<bool>, size: i32) -> String {
        "bool".to_string()
    }

    fn serialize(val: bool) -> Self {
        MySQLValue(From::from(val))
    }

    fn deserialize(self) -> bool {
        mysql_async::from_value(self.0)
    }
}

impl SQLValue<String> for MySQLValue {
    fn column_type(_: std::marker::PhantomData<String>, size: i32) -> String {
        if size > 0 {
            format!("varchar({})", size)
        } else {
            "text".to_string()
        }
    }

    fn serialize(val: String) -> MySQLValue {
        MySQLValue(From::from(val))
    }

    fn deserialize(self) -> String {
        mysql_async::from_value(self.0)
    }
}

impl SQLValue<i32> for MySQLValue {
    fn column_type(_: std::marker::PhantomData<i32>, _: i32) -> String {
        "int".to_string()
    }

    fn serialize(val: i32) -> MySQLValue {
        MySQLValue(From::from(val))
    }

    fn deserialize(self) -> i32 {
        mysql_async::from_value(self.0)
    }
}

impl SQLValue<u32> for MySQLValue {
    fn column_type(_: PhantomData<u32>, size: i32) -> String {
        "int unsigned".to_string()
    }

    fn serialize(val: u32) -> Self {
        MySQLValue(From::from(val))
    }

    fn deserialize(self) -> u32 {
        mysql_async::from_value(self.0)
    }
}

impl SQLValue<i64> for MySQLValue {
    fn column_type(_: std::marker::PhantomData<i64>, _: i32) -> String {
        "bigint".to_string()
    }

    fn serialize(val: i64) -> MySQLValue {
        MySQLValue(From::from(val))
    }

    fn deserialize(self) -> i64 {
        mysql_async::from_value(self.0)
    }
}

impl SQLValue<u64> for MySQLValue {
    fn column_type(_: PhantomData<u64>, size: i32) -> String {
        "bigint unsigned".to_string()
    }

    fn serialize(val: u64) -> Self {
        MySQLValue(From::from(val))
    }

    fn deserialize(self) -> u64 {
        mysql_async::from_value(self.0)
    }
}

impl<V> SQLValue<Option<V>> for MySQLValue
where
    MySQLValue: SQLValue<V>,
{
    fn column_type(_: std::marker::PhantomData<Option<V>>, size: i32) -> String {
        <MySQLValue as SQLValue<V>>::column_type(std::marker::PhantomData::<V>, size)
    }

    fn serialize(val: Option<V>) -> MySQLValue {
        match val {
            None => MySQLValue(mysql_async::Value::NULL),
            Some(v) => SQLValue::serialize(v),
        }
    }

    fn deserialize(self) -> Option<V> {
        match self.0 {
            mysql_async::Value::NULL => None,
            _ => Some(SQLValue::deserialize(self)),
        }
    }
}
