# Changelog

## v0.4.2 - 2021-01-20

* Fixes params name problem

## v0.4.1 - 2021-01-20

* Fixes binds problem for QueryBuilder when using wheres method

## v0.4.0 - 2021-01-18

* Add record_expr! macro, which can be used to construct a query with some filtering conditions type-safely
* Support record_expr! macro into QueryBuilder::wheres method
* **BREAKING CHANGE**: Rename load_with/first_with to load/first and remove legacy load/first methods.

## v0.3.3 - 2020-04-21

* Bump the version of `debil-derive`

## v0.3.2 - 2020-04-21

* Make accessors public

## v0.3.1 - 2020-04-06

* Fix `accessor!` macro

## v0.3.0 - 2020-04-06

* Now `accessor!` macro generates prefixed name like `table_name.column`. A new macro `accessor_name!` is provided, which is compatible with old accessor macro.
* Add `ORDER BY` support for QueryBuilder

## v0.2.1 - 2020-03-28

* Export `accessor!` macro

## v0.2.0 - 2020-03-22

* `debil-derive` now supports `Accessor` proc-macro.
* `debil` supports `accessor!` macro, which helps using `Accessor` derives.

## v0.1.2 - 2020-03-22

* Fix #5

## v0.1.1 - 2020-01-19

* Fix `save` query
  * Now `UPDATE` first and `INSERT` if needed

## v0.1.0 - 2020-01-19

* First Release

