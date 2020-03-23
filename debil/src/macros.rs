macro_rules! accessor {
    ($t: ident :: $v: ident) => {
        <$t>::$v()
    };
}

#[cfg(test)]
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
        accessor!(H::f);
    }
}
