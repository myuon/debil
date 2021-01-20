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
            $params.push((accessor_name!($name::$field).to_string(), SQLValue::serialize(expr)));

            ($result, $params)
        }
    };

    (@record_expr $result:ident, $params:ident, $var:ident, $name:ident, $field:ident : $e:expr, $($tails:tt)*) => {
        {
            let expr = $e;
            $var.$field = expr.clone();

            let params_name = format!(":{}", accessor_name!($name::$field));
            $result.push(format!("{} = {}", accessor!($name::$field), &params_name));
            $params.push((accessor_name!($name::$field).to_string(), SQLValue::serialize(expr)));

            record_expr!(@record_expr $result, $params, $var, $name, $($tails)*)
        }
    };
}

#[cfg(test)]
#[allow(dead_code)]
mod test_record_expr {
    use crate::{SQLMapper, SQLTable};

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

    impl SQLMapper for H {
        type ValueType = Vec<u8>;

        fn map_from_sql(_: std::collections::HashMap<String, Self::ValueType>) -> Self {
            todo!()
        }
    }

    impl SQLTable for H {
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
        use crate::types::SQLValue;

        // single equation
        assert_eq!(
            record_expr!(H, { f: 2000 }),
            (
                vec!["table_H.foo = :foo".to_string()],
                vec![("foo".to_string(), SQLValue::serialize(2000))] as Vec<(String, Vec<u8>)>
            )
        );

        // trailing comma
        assert_eq!(
            record_expr!(H, { f: 2000, }),
            (
                vec!["table_H.foo = :foo".to_string()],
                vec![("foo".to_string(), SQLValue::serialize(2000))] as Vec<(String, Vec<u8>)>
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
                    ("foo".to_string(), SQLValue::serialize(2000)),
                    ("g".to_string(), SQLValue::serialize("fooo".to_string()))
                ] as Vec<(String, Vec<u8>)>
            )
        );
    }
}
