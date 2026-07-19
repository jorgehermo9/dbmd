# DuckDB backend

This crate owns embedded DuckDB configuration, catalog introspection,
presentation mapping, and templates.

## Public interface

- `Config` resolves a main file, display name, and named attached files relative
  to the project configuration.
- `DuckDbSource` opens the main database read-only and can attach additional
  databases read-only by default before `introspect` produces a deterministic
  snapshot.
- `render_source` and `template_files` expose DuckDB-owned presentation to the
  composition root.

## Coverage

The supported surface includes attached databases and schemas, tables and
views, columns and defaults, primary/unique/foreign/check constraints, indexes,
sequences, enum and alias types, comments, SQL macros and table macros, and
extension state. Native catalog SQL is retained wherever DuckDB exposes it.
Volatile row-count estimates and secrets are intentionally excluded.

| Schema surface | Represented facts |
| --- | --- |
| Databases and schemas | Catalog name/type/read-only state and qualified schemas, including configured attachments. Resolved filesystem paths stay out of rendered artifacts. |
| Tables and columns | Qualified identity, temporary state, comments, defaults, generated expressions, types, nullability, and raw creation SQL. |
| Constraints and indexes | Primary, unique, foreign and check constraint text/expressions; index uniqueness, expressions, comments, and raw SQL. |
| Views and sequences | Qualified identity, comments, temporary state, sequence bounds/increment/cycle, and definitions. |
| Types | Logical/category metadata and enum labels. |
| Functions | Scalar and table macros, parameters, return/vararg metadata, stability, side effects, comments, and definitions. |
| Extensions | Installed/loaded state, version, and description. |

## Template context

`source.data` contains `section_heading`, `object_heading`, `detail_heading`,
`namespaces`, `tables`, `views`, and `objects`. Namespaces are qualified as
`catalog.schema`. The final collection contains types, sequences, macros, and
extensions with a distinguishing `Kind` fact. Tables and views use the shared
presentation fields with DuckDB-specific temporary state and raw definitions.
Directory objects are emitted in table, view, then object order.

All values are Markdown-ready and deterministically ordered. See the
[common template envelope](../../../docs/product/features/templates.md).

## Contract test

```sh
cargo test -p dbmd-backend-duckdb --test duckdb
```
