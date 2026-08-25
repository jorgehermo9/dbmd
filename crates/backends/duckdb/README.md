# DuckDB backend

This crate owns embedded DuckDB 1.5.4 configuration, catalog introspection,
presentation mapping, and templates. DuckDB 1.5.4 is the exact supported and
contract-tested release.

## Public interface

- `Config` resolves a main file, display name, optional persistent-secret and
  extension directories, and named attached files relative to the project
  configuration.
- `DuckDbSource` opens the main database read-only and can attach additional
  databases read-only by default before `introspect` produces a deterministic
  snapshot.
- `render_source` and `template_files` expose DuckDB-owned presentation to the
  composition root.

## Coverage

The supported persistent surface includes attached databases and schemas,
tables and views, columns and defaults, primary/unique/foreign/check
constraints, indexes, sequences, enum/alias/struct/union types, comments, SQL
macros and table macros, extension state, and persistent-secret metadata.
Native catalog SQL is retained wherever DuckDB exposes it. Volatile row-count
estimates are intentionally excluded. Secret values are excluded at
acquisition: dbmd never queries `secret_string`.

Closed metadata vocabularies are normalized at acquisition: constraint and
function families, function stability, and extension installation mode enter
the catalog as semantic enums with canonical display names. Unknown values in
those documented closed sets fail introspection. Extensible values such as
database/storage type, index/access method, secret provider/type/storage, and
repository identity remain strings because extensions may introduce them.

| Schema surface | Represented facts |
| --- | --- |
| Databases and schemas | Catalog name/type/read-only state, safe allowlisted attach options, tags, and qualified schemas, including configured attachments. Resolved filesystem paths stay out of rendered artifacts. Encryption keys are excluded in the metadata query itself. |
| Tables and columns | Qualified identity, temporary state, comments, tags, defaults, generated expressions, types, numeric precision/radix/scale, nullability, and raw creation SQL. |
| Constraints and indexes | Constraint identity, primary/unique/foreign/check text and expressions, foreign-key targets; index type, uniqueness, expressions, comments, tags, and raw SQL. |
| Views and sequences | Qualified identity, comments, tags, temporary state, sequence bounds/increment/cycle, and definitions. |
| Types | Logical/category metadata, storage size, definitions, enum labels, comments, and tags. |
| Functions | Scalar and table macros, parameters, return/vararg metadata, stability, side effects, comments, definitions, and tags. |
| Extensions | Installed/loaded state, version, description, aliases, install mode, and source repository. Install paths are excluded. |
| Persistent secrets | Name, type, provider, persistence, storage, and scope. Credential fields and the configured secret-directory path never enter the catalog or rendered output. |

DuckDB 1.5.4 exposes `duckdb_dependencies()`, but its dependency rows do not
survive reopening a persisted database. dbmd therefore does not present that
function as a durable dependency graph. Persistent dependency-bearing SQL,
such as sequence defaults and view definitions, remains available in the
owning objects' definitions.

## Template context

`source.data` contains `section_heading`, `object_heading`, `detail_heading`,
`namespaces`, `tables`, `views`, and `objects`. Namespaces are qualified as
`catalog.schema`. The final collection contains types, sequences, macros, and
extensions with a distinguishing `Kind` fact. Tables and views use the shared
presentation fields with DuckDB-specific temporary state and raw definitions.
Secret objects contain metadata only and never contain credential material.
Directory objects are emitted in table, view, then object order.

All values are Markdown-ready and deterministically ordered. See the
[common template envelope](../../../docs/product/features/templates.md).

## Contract test

```sh
cargo test -p dbmd-backend-duckdb --test duckdb
```

The persistent-secret contract downloads DuckDB's official 1.5.4 `httpfs`
extension into a temporary test-only extension directory. It never reads or
writes the user's DuckDB extension or secret directories.
