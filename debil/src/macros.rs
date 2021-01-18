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
            let mut $var: $name = Default::default();

            record_expr!(@record_expr result, $var, $name, $($body)*)
        }
    };

    (@record_expr $result:ident, $var:ident, $name:ident, $field:ident : ? $(,)?) => {
        {
            $result.push(format!("{} = ?", accessor!($name::$field)));

            $result
        }
    };

    (@record_expr $result:ident, $var:ident, $name:ident, $field:ident : $e:expr $(,)?) => {
        {
            $var.$field = $e;

            $result.push(format!("{} = {}", accessor!($name::$field), $e));

            $result
        }
    };

    (@record_expr $result:ident, $var:ident, $name:ident, $field:ident : ?, $($tails:tt)*) => {
        {
            $result.push(format!("{} = ?", accessor!($name::$field)));

            record_expr!(@record_expr $result, $var, $name, $($tails)*)
        }
    };

    (@record_expr $result:ident, $var:ident, $name:ident, $field:ident : $e:tt, $($tails:tt)*) => {
        {
            $var.$field = $e;

            $result.push(format!("{} = {}", accessor!($name::$field), $e));

            record_expr!(@record_expr $result, $var, $name, $($tails)*)
        }
    };
}

#[cfg(test)]
#[allow(dead_code)]
mod test_record_expr {
    use crate::{SQLMapper, SQLTable};

    #[derive(Default)]
    struct H {
        f: usize,
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
        type ValueType = ();

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
        // single equation
        assert_eq!(record_expr!(H, { f: 2000 }), vec!["table_H.foo = 2000"]);

        // trailing comma
        assert_eq!(record_expr!(H, { f: 2000, }), vec!["table_H.foo = 2000"]);

        // This gives you a type error!
        // assert_eq!(record_expr!(H, { f: "fooo" }), vec!["table_H.foo = fooo"]);

        // multiple columns
        assert_eq!(
            record_expr!(H, { f: 2000, g: "'fooo'".to_string() }),
            vec!["table_H.foo = 2000", "table_H.g = 'fooo'"]
        );

        // with ? placeholders
        assert_eq!(
            record_expr!(H, { f: ?, g: ? }),
            vec!["table_H.foo = ?", "table_H.g = ?"]
        );
    }
}
