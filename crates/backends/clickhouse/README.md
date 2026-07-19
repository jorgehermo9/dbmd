# dbmd ClickHouse backend

This crate owns ClickHouse source configuration, catalog semantics,
introspection, presentation mapping, templates, fixtures, and tests.

## Public interface

- `Config` resolves HTTP URL, optional database scope, credentials, and display
  name without exposing expanded values to rendering.
- `ClickHouseSource` is a concrete HTTP source; `introspect` returns a
  deterministic `SourceSnapshot<Catalog>`.
- `render_source` and `template_files` expose the ClickHouse-owned
  presentation and embedded source entrypoints to the composition root.

## Coverage

The fixture-backed schema surface includes databases, tables, views,
materialized views and their targets, columns and default kinds, MergeTree
engine parameters, partition/sorting/primary/sampling keys, storage policies,
column codecs and TTL expressions, table TTL definitions, projections,
data-skipping indexes, constraints, comments, and user-defined functions.

Raw creation SQL remains the fidelity backstop for engine settings and
expressions that ClickHouse exposes textually. Volatile rows, bytes, parts,
replication health, mutations, and query statistics are not canonical schema
context.

| Schema surface | Represented facts |
| --- | --- |
| Databases | Name, engine, and comment. Server filesystem paths are intentionally excluded. |
| Tables and dictionaries | Database/name, engine family and full engine expression, comment, raw creation SQL, settings, storage policy, and keys. Table and column TTL clauses remain available in raw creation SQL/engine text. |
| Views and materialized views | Raw definition, view kind, and materialized-view target where present. |
| Columns | Position, type, default kind/expression, comment, codec, and key roles. |
| Constraints and indexes | Check/assume expressions, data-skipping index expression/type/granularity, and projections. |
| SQL UDFs | Name and creation definition. |

## Template context

`source.data` contains `section_heading`, `object_heading`, `detail_heading`,
`namespaces`, `tables`, `views`, and `functions`. Tables use the shared
presentation fields plus a `ClickHouse` backend detail block containing engine,
key, TTL, projection, skip-index, and settings facts and fenced creation SQL.
Views retain their kind, materialized target, and definition. Functions expose
qualified name, file name, facts, and definition. Directory objects are emitted
in table, view, then function order.

All values are Markdown-ready and all collections retain catalog order. See the
[common template envelope](../../../docs/product/features/templates.md).

## Contract test

Run the real-server contract with:

```sh
cargo test -p dbmd-backend-clickhouse --features clickhouse-tests --test clickhouse
```
