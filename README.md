# debil [![debil at crates.io](https://img.shields.io/crates/v/debil.svg)](https://crates.io/crates/debil)

Lightweight ORM for Rust

To use a specific DB, see debil-xxx family.

- [debil-mysql](https://github.com/myuon/debil-mysql)
- [debil-dynamodb](https://github.com/myuon/debil-dynamodb)

## How to use

Basically `debil` provides `Table` macro and `Accessor` macro.

### Table macro

You need to specify `sql_type` to be something that each DB crate provides.

```rust
#[derive(Table)]
#[sql(table_name = "ex_1", sql_type = "...", primary_key = "pk")]
struct Ex1 {
    #[sql(size = 50, unique = true, not_null = true)]
    field1: String,
    aaaa: i32,
    pk: i32,
}
```

This example derives some useful mapper functions for this struct. See functions in [debil's docs](https://docs.rs/debil/).

### Accessor macro

Accessor macro provides safe way to access to each column. This is useful for constructing a query.

```rust
// Use Accessor derive here!
#[derive(Table, Accessor)]
#[sql(table_name = "ex_1", sql_type = "...", primary_key = "pk")]
struct Ex1 {
    field1: String,
    aaaa: i32,
    pk: i32,
}

// Use accessor! macro to access to a field with table_name prefixed
assert_eq!(accessor!(Ex1::field1), "ex_1.field1");

// If you only need field name, use accessor_name! macro
assert_eq!(accessor_name!(Ex1::aaaa), "aaaa");

// Or you can just call the field name function directly, which is derived by Accessor derive
assert_eq!(Ex1::field1(), "field1");

// accessor!(Ex1::foobar) <- compile error!
```
