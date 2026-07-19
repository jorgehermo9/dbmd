# MySQL backend

This crate owns MySQL connection configuration, catalog introspection,
presentation mapping, and embedded templates. Its catalog deliberately does not
double as the MariaDB catalog.

## Public interface

- `Config` resolves a URL, optional schema scope, and display name.
- `MysqlSource` and `introspect` expose concrete MySQL catalog access without a
  driver trait.
- `render_source` and `template_files` expose MySQL-owned presentation to the
  composition root.

## Coverage

The supported schema surface includes schemas, base tables, views, columns
(including generated and invisible columns), defaults, character sets,
collations, comments, primary/unique/foreign/check constraints, multi-part and
functional indexes, index prefix/order/visibility metadata, partitioning and
table options, triggers, stored procedures and functions with parameters, and
scheduled events. `SHOW CREATE` definitions are retained as a fidelity
backstop. Volatile storage statistics and grants are intentionally excluded.

| Schema surface | Represented facts |
| --- | --- |
| Schemas | Name, default character set, and default collation. |
| Tables | Engine, row format, collation, create options, comment, partitions, and `SHOW CREATE`. |
| Columns | Position, type, nullability, default, generated expression, invisibility, character set/collation, enum values, and comment. |
| Constraints | Primary, unique, foreign, and check constraints, including referential actions and check enforcement. |
| Indexes | Ordered columns/expressions, prefix lengths, direction, uniqueness, type, visibility, and comment. |
| Other objects | Views, routines and parameters, triggers, and scheduled events with raw definitions. |

## Template context

`source.data` contains `section_heading`, `object_heading`, `detail_heading`,
`namespaces`, `tables`, `views`, `triggers`, `routines`, and `events`. Tables use
the shared presentation fields plus a `MySQL` backend block for engine,
partition, and creation facts. Other object values expose qualified name, file
name, nullable comment, facts, and fenced definition. Directory objects are
emitted in table, view, trigger, routine, then event order.

All values are Markdown-ready and deterministically ordered. See the
[common template envelope](../../../docs/product/features/templates.md).

## Contract test

```sh
cargo test -p dbmd-backend-mysql --features mysql-tests --test mysql
```
