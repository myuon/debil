#[macro_export]
macro_rules! accessor {
    ($t: ident :: $v: ident) => {
        format!("{}.{}", debil::table_name::<$t>(), <$t>::$v())
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
