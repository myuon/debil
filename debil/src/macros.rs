#[macro_export]
macro_rules! accessor {
    ($t: ident :: $v: ident) => {
        format!("{}.{}", $crate::table_name::<$t>(), <$t>::$v())
    };
}

#[macro_export]
macro_rules! accessor_name {
    ($t: ident :: $v: ident) => {
        <$t>::$v()
    };
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
    struct H {
        f: usize,
    }

    impl H {
        fn f() -> &'static str {
            "foo"
        }
    }

    fn k() {
        accessor_name!(H::f);
    }
}

#[macro_export]
macro_rules! record_expr {
    ($name:ident, {$($body:tt)*}) => {
        record_expr!(@wrapper _internal_for_type_checking, $name, $($body)*)
    };

    (@wrapper $var:ident, $name:ident, $($body:tt)*) => {
        {
            let mut result = vec![];
            let mut params = vec![];
            let mut $var: $name = Default::default();

            record_expr!(@record_expr result, params, $var, $name, $($body)*)
        }
    };

    (@record_expr $result:ident, $params:ident, $var:ident, $name:ident, $field:ident : $e:expr $(,)?) => {
        {
            let expr = $e;
            $var.$field = expr.clone();

            let params_name = format!(":{}", accessor_name!($name::$field));
            $result.push(format!("{} = {}", accessor!($name::$field), &params_name));
            $params.push((accessor_name!($name::$field).to_string(), SqlValue::serialize(expr)));

            ($result, $params)
        }
    };

    (@record_expr $result:ident, $params:ident, $var:ident, $name:ident, $field:ident : $e:expr, $($tails:tt)*) => {
        {
            let expr = $e;
            $var.$field = expr.clone();

            let params_name = format!(":{}", accessor_name!($name::$field));
            $result.push(format!("{} = {}", accessor!($name::$field), &params_name));
            $params.push((accessor_name!($name::$field).to_string(), SqlValue::serialize(expr)));

            record_expr!(@record_expr $result, $params, $var, $name, $($tails)*)
        }
    };
}

#[cfg(test)]
#[allow(dead_code)]
mod test_record_expr {
    use std::convert::TryFrom;

    use crate::{SqlMapper, SqlTable, SqlValue};

    #[derive(Clone, Debug, PartialEq)]
    struct Binary(Vec<u8>);

    impl SqlValue<i32> for Binary {
        fn column_type(_: std::marker::PhantomData<i32>, _: i32) -> String {
            "int".to_string()
        }

        fn serialize(v: i32) -> Self {
            Binary(v.to_be_bytes().to_vec())
        }
        fn deserialize(self) -> i32 {
            i32::from_be_bytes(TryFrom::try_from(self.0.as_slice()).unwrap())
        }
    }

    impl SqlValue<String> for Binary {
        fn column_type(_: std::marker::PhantomData<String>, size: i32) -> String {
            format!("varchar({})", size)
        }

        fn serialize(v: String) -> Self {
            Binary(v.as_bytes().to_vec())
        }
        fn deserialize(self) -> String {
            String::from_utf8(self.0).unwrap()
        }
    }

    #[derive(Default)]
    struct H {
        f: i32,
        g: String,
    }

    impl H {
        fn f() -> &'static str {
            "foo"
        }

        fn g() -> &'static str {
            "g"
        }
    }

    impl SqlMapper for H {
        type ValueType = Vec<u8>;

        fn map_from_sql(_: std::collections::HashMap<String, Self::ValueType>) -> Self {
            todo!()
        }
    }

    impl SqlTable for H {
        fn table_name(_: std::marker::PhantomData<Self>) -> String {
            "table_H".to_string()
        }

        fn schema_of(
            _: std::marker::PhantomData<Self>,
        ) -> Vec<(String, String, crate::FieldAttribute)> {
            todo!()
        }

        fn primary_key_columns(_: std::marker::PhantomData<Self>) -> Vec<String> {
            todo!()
        }

        fn map_to_sql(self) -> Vec<(String, Self::ValueType)> {
            todo!()
        }
    }

    #[test]
    fn record_expr() {
        use crate::types::SqlValue;

        // single equation
        assert_eq!(
            record_expr!(H, { f: 2000 }),
            (
                vec!["table_H.foo = :foo".to_string()],
                vec![("foo".to_string(), SqlValue::serialize(2000))] as Vec<(String, Binary)>
            )
        );

        // trailing comma
        assert_eq!(
            record_expr!(H, { f: 2000, }),
            (
                vec!["table_H.foo = :foo".to_string()],
                vec![("foo".to_string(), SqlValue::serialize(2000))] as Vec<(String, Binary)>
            )
        );

        // This gives you a type error!
        // record_expr!(H, { f: "fooo" })

        // multiple columns
        assert_eq!(
            record_expr!(H, { f: 2000, g: "fooo".to_string() }),
            (
                vec![
                    "table_H.foo = :foo".to_string(),
                    "table_H.g = :g".to_string()
                ],
                vec![
                    ("foo".to_string(), SqlValue::serialize(2000)),
                    ("g".to_string(), SqlValue::serialize("fooo".to_string()))
                ] as Vec<(String, Binary)>
            )
        );
    }
}
