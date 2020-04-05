# Changelog

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

