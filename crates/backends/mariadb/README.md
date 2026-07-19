# MariaDB backend

MariaDB-specific catalog introspection and rendering. The catalog owns MariaDB
features such as sequences and system-versioned tables instead of flattening
them into MySQL-shaped metadata.

## Public interface

- `Config` resolves a URL, optional schema scope, and display name.
- `MariaDbSource` and `introspect` expose concrete MariaDB catalog access.
- `render_source` and `template_files` expose MariaDB-owned presentation to
  the composition root.

## Coverage

The supported surface includes schemas, base tables, views, columns and virtual
columns, defaults, character sets, collations, comments, primary/unique/foreign
key/check constraints, ordered/prefix/ignored indexes, partitions, system-time
periods and versioning, sequences, routines and parameters, triggers, and
scheduled events. `SHOW CREATE` definitions are retained as a fidelity
backstop. Volatile storage statistics and grants are intentionally excluded.

| Schema surface | Represented facts |
| --- | --- |
| Schemas | Name, default character set, and default collation. |
| Tables | Engine, row format, collation, create options, partitions, system-versioning state/period, comment, and `SHOW CREATE`. |
| Columns | Position, type, nullability, default, virtual/generated expression, character set/collation, and comment. |
| Constraints and indexes | Primary, unique, foreign and check constraints; ordered/prefix indexes including ignored state. |
| MariaDB objects | Sequences and system-time periods retain MariaDB-specific facts. |
| Other objects | Views, routines and parameters, triggers, and scheduled events with raw definitions. |

## Template context

`source.data` contains `section_heading`, `object_heading`, `detail_heading`,
`namespaces`, `tables`, `views`, `triggers`, and `functions`. The final
collection contains routines, sequences, and events with a distinguishing
`Kind` fact. Tables use the shared presentation fields plus a `MariaDB` backend
block for engine, partitioning, system-versioning, and creation facts.
Directory objects are emitted in table, view, trigger, then function/object
order.

All values are Markdown-ready and deterministically ordered. See the
[common template envelope](../../../docs/product/features/templates.md).

## Contract test

```sh
cargo test -p dbmd-backend-mariadb --features mariadb-tests --test mariadb
```
