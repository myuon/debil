use crate::types::SQLValue;
use std::convert::TryFrom;

// Vec<u8> serializer

impl SQLValue<Vec<u8>> for i32 {
    fn column_type(_: std::marker::PhantomData<Self>, _: i32) -> String {
        "int".to_string()
    }

    fn serialize(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
    fn deserialize(val: Vec<u8>) -> i32 {
        i32::from_be_bytes(TryFrom::try_from(val.as_slice()).unwrap())
    }
}

impl SQLValue<Vec<u8>> for String {
    fn column_type(_: std::marker::PhantomData<Self>, size: i32) -> String {
        format!("varchar({})", size)
    }

    fn serialize(self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
    fn deserialize(val: Vec<u8>) -> String {
        String::from_utf8(val).unwrap()
    }
}
