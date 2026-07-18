# SQLite introspection

This module reads a SQLite database in read-only mode and produces one normalized
`dbmd_core::SourceSnapshot`. It describes the resulting schema surface, not the
migration history or the sequence of DDL statements that created it.

## Public interface

- `SqliteSource` carries stable source identity, an optional display name, a
  resolved main database path, and ordered configured attachments.
- `introspect` opens that path read-only and returns a deterministically ordered
  snapshot or a source-scoped error.

The adapter reads `main` followed by configured attached namespaces. Within a
namespace, objects use binary name order; columns use catalog ordinal order;
composite keys use declared key order; and index and foreign-key terms use their
stored sequence.

## Coverage

The status values mean:

- **Supported**: represented by the public snapshot and proved with a real
  SQLite fixture.
- **Partial**: some semantics are represented, but known SQLite forms are not.
- **Planned**: not yet represented by this adapter.

| Schema surface | Status | Current behavior and fixture evidence |
| --- | --- | --- |
| Ordinary tables and columns | Supported | Names, declared types, effective nullability, defaults, collations, ordinal order, exact stored definitions, and rowid-primary-key behavior; [ordinary table fixture](../../tests/fixtures/sqlite/ordinary_table/schema.sql) and [table-definition fixture](../../tests/fixtures/sqlite/table_definition/schema.sql). |
| Primary keys | Supported | Named/unnamed, column/table, single/composite, conflict policy, `AUTOINCREMENT`, physical index origin, and effective nullability; [table-definition fixture](../../tests/fixtures/sqlite/table_definition/schema.sql). |
| `NOT NULL`, `UNIQUE`, and `CHECK` | Supported | Names, column/table declaration, expressions, conflict algorithms, and backing index origins are preserved; [table-definition fixture](../../tests/fixtures/sqlite/table_definition/schema.sql). |
| Generated columns | Supported | Virtual/stored kinds and generating expressions are preserved; [table-features fixture](../../tests/fixtures/sqlite/table_features/schema.sql). |
| `STRICT` tables | Supported | Preserved from `PRAGMA table_list`; [table features fixture](../../tests/fixtures/sqlite/table_features/schema.sql). |
| `WITHOUT ROWID` tables | Supported | Preserved from `PRAGMA table_list`; [table features fixture](../../tests/fixtures/sqlite/table_features/schema.sql). |
| Foreign keys | Supported | Named/unnamed, column/table, explicit/implicit composite targets, all update/delete actions, `MATCH`, and deferrability are merged from PRAGMAs and stored SQL; [relationships fixture](../../tests/fixtures/sqlite/relationships/schema.sql) and [table-definition fixture](../../tests/fixtures/sqlite/table_definition/schema.sql). |
| Indexes | Supported | Explicit, unique, partial, expression, collated, ascending/descending, UNIQUE-backed, and primary-key-backed indexes preserve typed terms, origin, predicate, and raw definition; [indexes fixture](../../tests/fixtures/sqlite/indexes/schema.sql). |
| Views | Supported | Qualified identity, derived/declared columns with unknown nullability, and exact definitions; [schema-objects fixture](../../tests/fixtures/sqlite/schema_objects/schema.sql). |
| Triggers | Supported | BEFORE/AFTER/INSTEAD OF timing, INSERT/UPDATE/UPDATE OF/DELETE events, target, `WHEN`, and exact body definition; [schema-objects fixture](../../tests/fixtures/sqlite/schema_objects/schema.sql). |
| Virtual and shadow tables | Supported | Module name, raw module arguments, visible/hidden columns, exact definition, and typed shadow-table ownership; [schema-objects fixture](../../tests/fixtures/sqlite/schema_objects/schema.sql). |
| Attached databases | Supported | Configured persistent attachments are opened query-only and traversed after `main` in configuration order; [namespaces fixture](../../tests/fixtures/sqlite/namespaces). |
| `CREATE TABLE AS` | Supported as resulting schema | SQLite retains the resulting table definition, not the original SELECT; [schema-evolution fixture](../../tests/fixtures/sqlite/schema_evolution/schema.sql). |
| `ALTER TABLE` | Supported as resulting schema | Rename table/column, add/drop column, and SQLite 3.53 `SET/DROP NOT NULL` are proved through the rewritten stored definition; [schema-evolution fixture](../../tests/fixtures/sqlite/schema_evolution/schema.sql). |
| `DROP TABLE/INDEX/VIEW/TRIGGER` | Supported as resulting absence | Dropped objects are absent from the snapshot; [schema-evolution fixture](../../tests/fixtures/sqlite/schema_evolution/schema.sql). |
| TEMP schema objects | Intentionally excluded | TEMP objects belong to one connection and disappear before a persistent source can be reopened. The schema-evolution fixture creates TEMP table/view/trigger objects and proves they do not enter the persistent snapshot. |

Other SQL statements such as `ALTER TABLE` and `DROP TABLE` change schema history
but do not create additional kinds of final schema objects. The target is complete
coverage of SQLite's introspectable schema surface and explicit reporting of facts
SQLite does not retain—not replay or documentation of every DDL statement.

## Catalog sources

The implementation combines `sqlite_schema`, `PRAGMA table_list`,
`table_xinfo`, `foreign_key_list`, `index_list`, and `index_xinfo`. It parses
SQLite's stored schema SQL for names, expressions, conflict policies, trigger
semantics, and virtual-table arguments that structured catalog interfaces omit.
The exact stored SQL remains in the model as the fidelity backstop.

## Adding coverage

Add a focused directory under `tests/fixtures/sqlite`, execute its `schema.sql`
against a real temporary database in `tests/sqlite.rs`, and snapshot the public
source result. Update this matrix in the same change so implementation status and
fixture evidence stay synchronized.
