use crate::types::SQLValue;
use std::convert::TryFrom;

// Vec<u8> serializer

impl SQLValue<i32> for Vec<u8> {
    fn column_type(_: std::marker::PhantomData<i32>, _: i32) -> String {
        "int".to_string()
    }

    fn serialize(v: i32) -> Vec<u8> {
        v.to_be_bytes().to_vec()
    }
    fn deserialize(self) -> i32 {
        i32::from_be_bytes(TryFrom::try_from(self.as_slice()).unwrap())
    }
}

impl SQLValue<String> for Vec<u8> {
    fn column_type(_: std::marker::PhantomData<String>, size: i32) -> String {
        format!("varchar({})", size)
    }

    fn serialize(v: String) -> Self {
        v.as_bytes().to_vec()
    }
    fn deserialize(self) -> String {
        String::from_utf8(self).unwrap()
    }
}

impl<V> SQLValue<Option<V>> for Vec<u8>
where
    Vec<u8>: SQLValue<V>,
{
    fn column_type(_: std::marker::PhantomData<Option<V>>, size: i32) -> String {
        Self::column_type(std::marker::PhantomData::<V>, size)
    }

    fn serialize(v: Option<V>) -> Self {
        match v {
            None => vec![],
            Some(v) => SQLValue::serialize(v),
        }
    }
    fn deserialize(self) -> Option<V> {
        if self.is_empty() {
            None
        } else {
            Some(Self::deserialize(self))
        }
    }
}
