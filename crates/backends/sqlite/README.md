# dbmd-backend-sqlite

This vertical backend crate reads a SQLite database in read-only mode and
produces one normalized
`dbmd_core::SourceSnapshot<dbmd_backend_sqlite::Catalog>`. It describes the
resulting schema surface, not the migration history or the sequence of DDL
statements that created it.

The executable compatibility target for this coverage matrix is SQLite 3.53.3.

## Public interface

- `SqliteSource` carries stable source identity, an optional display name, a
  resolved main database path, and ordered configured attachments.
- `Config` owns the committed SQLite field shape and resolves its paths and
  attachment namespaces through application-supplied value expansion.
- `introspect` opens that path read-only and returns a deterministically ordered
  snapshot or a source-scoped error.
- `render_source` maps the SQLite catalog to SQLite-owned presentation data and
  a generic object manifest; `template_files` exposes the SQLite source
  templates for both output layouts.

The adapter reads `main` followed by configured attached namespaces. Within a
namespace, objects use binary name order; columns use catalog ordinal order;
composite keys use declared key order; and index and foreign-key terms use their
stored sequence.

## Coverage

`Supported` means represented by the public snapshot and proved with a real
SQLite fixture. `Intentionally excluded` identifies state that is not part of a
reopenable persistent SQLite database.

| Schema surface | Status | Current behavior and fixture evidence |
| --- | --- | --- |
| Ordinary tables and columns | Supported | Names, quoted identifiers, declared types, effective nullability, defaults, collations, ordinal order, exact stored definitions, and rowid-primary-key behavior—including the documented `INTEGER PRIMARY KEY DESC` exception; [ordinary table fixture](tests/fixtures/ordinary_table/schema.sql), [table-definition fixture](tests/fixtures/table_definition/schema.sql), and [grammar-edge fixture](tests/fixtures/grammar_edges/schema.sql). |
| Primary keys | Supported | Named/unnamed, column/table, single/composite, conflict policy, `AUTOINCREMENT`, physical index origin, and effective nullability; [table-definition fixture](tests/fixtures/table_definition/schema.sql). |
| `NOT NULL`, `UNIQUE`, and `CHECK` | Supported | Names, column/table declaration, expressions, conflict algorithms, and backing index origins are preserved; [table-definition fixture](tests/fixtures/table_definition/schema.sql). |
| Generated columns | Supported | Virtual/stored kinds and generating expressions are preserved; [table-features fixture](tests/fixtures/table_features/schema.sql). |
| `STRICT` tables | Supported | Preserved from `PRAGMA table_list`; [table features fixture](tests/fixtures/table_features/schema.sql). |
| `WITHOUT ROWID` tables | Supported | Preserved from `PRAGMA table_list`; [table features fixture](tests/fixtures/table_features/schema.sql). |
| Foreign keys | Supported | Named/unnamed, column/table, explicit/implicit composite targets, all update/delete actions, `MATCH`, and deferrability are merged from PRAGMAs and stored SQL. Multiple named constraints may share source columns and a target table without their distinct target keys being conflated; [relationships fixture](tests/fixtures/relationships/schema.sql), [table-definition fixture](tests/fixtures/table_definition/schema.sql), and [grammar-edge fixture](tests/fixtures/grammar_edges/schema.sql). |
| Indexes | Supported | Explicit, unique, partial, expression, collated, ascending/descending, UNIQUE-backed, and primary-key-backed indexes preserve typed terms, origin, predicate, and raw definition; [indexes fixture](tests/fixtures/indexes/schema.sql). |
| Views | Supported | Qualified identity, derived/declared columns with unknown nullability, and exact definitions; [schema-objects fixture](tests/fixtures/schema_objects/schema.sql). |
| Triggers | Supported | BEFORE/AFTER/INSTEAD OF timing, INSERT/UPDATE/UPDATE OF/DELETE events, target, `WHEN`, and exact body definition; [schema-objects fixture](tests/fixtures/schema_objects/schema.sql). |
| Virtual and shadow tables | Supported | Module name, raw module arguments, visible/hidden columns, exact definition, and typed shadow-table ownership; [schema-objects fixture](tests/fixtures/schema_objects/schema.sql). |
| Attached databases | Supported | Configured persistent attachments are opened query-only and traversed after `main` in configuration order; [namespaces fixture](tests/fixtures/namespaces). |
| `CREATE TABLE AS` | Supported as resulting schema | SQLite retains the resulting table definition, not the original SELECT; [schema-evolution fixture](tests/fixtures/schema_evolution/schema.sql). |
| `ALTER TABLE` | Supported as resulting schema | Rename table/column, add/drop column, and SQLite 3.53 `SET/DROP NOT NULL` are proved through the rewritten stored definition; [schema-evolution fixture](tests/fixtures/schema_evolution/schema.sql). |
| `DROP TABLE/INDEX/VIEW/TRIGGER` | Supported as resulting absence | Dropped objects are absent from the snapshot; [schema-evolution fixture](tests/fixtures/schema_evolution/schema.sql). |
| TEMP schema objects | Intentionally excluded | TEMP objects belong to one connection and disappear before a persistent source can be reopened. The schema-evolution fixture creates TEMP table/view/trigger objects, including a SQLite 3.53 TEMP trigger targeting `main`, and proves they do not enter the persistent snapshot. |
| Internal `sqlite_` objects | Intentionally excluded | SQLite reserves the literal `sqlite_` prefix for internal schema objects. The filter uses literal glob semantics; the grammar-edge fixture proves valid `sqliteX*` user tables, views, and triggers remain visible. Constraint-backed `sqlite_autoindex_*` entries are retained as typed index origins because they describe user constraints. |

## Template context

SQLite source entrypoints receive the common `source` envelope documented by
the [template product contract](../../../docs/product/features/templates.md).
`source.data` has this SQLite-owned shape:

| Field | Type | Meaning and order |
| --- | --- | --- |
| `section_heading` | string | `##` without source nesting, `###` with nesting. |
| `object_heading` | string | Heading used by single-file object templates. |
| `detail_heading` | string | Heading used for table subsections in a single file. |
| `namespaces` | namespace[] | `main`, then configured attachments in resolved attachment order. |
| `tables` | table[] | Tables in namespace/name order, including represented virtual and shadow tables. |
| `views` | view[] | Views in namespace/name order. |
| `triggers` | trigger[] | Triggers in namespace and trigger-name order. |

SQLite directory objects are declared in tables, views, then triggers order.
Their paths are `tables/<file_name>`, `views/<file_name>`, and
`triggers/<file_name>`. Trigger identity and filenames include the target so
equal trigger names on different targets cannot collide.

Presentation object fields are:

- Namespace: `name`, nullable `comment`.
- Table: `qualified_name`, `file_name`, nullable `comment`, `columns`,
  `constraints`, `indexes`, and `backend`. `backend.title` is `SQLite`;
  `backend.facts` contains the table kind; `backend.notices` identifies
  `STRICT` and `WITHOUT ROWID`; nullable `backend.definition` is fenced SQL.
- Column: `name`, `data_type`, `nullable` (`yes`, `no`, or `unknown`),
  `default` (`-` when absent), and `notes` for comments, generated/hidden kind,
  collation, and generated expression.
- Constraint: `name`, `kind`, `columns`, and `details`; absent display values
  use `-`. Details preserve references, actions, match/deferral, conflict
  policy, and autoincrement when present.
- Index: `name`, `terms`, `unique`, `origin`, and `predicate`; absent predicates
  use `-`.
- View: `qualified_name`, `file_name`, nullable `comment` (currently null),
  `facts` (currently empty), `columns`, and fenced `definition`.
- Trigger: `qualified_name`, `file_name`, nullable `comment` (currently null),
  `event`, `target`, `facts` (currently empty), nullable `when_expression`, and
  fenced `definition`.

In a directory object template the selected table/view/trigger is `object`;
`heading`, `detail_heading`, and `source` are also available. In a single-file
source template the same objects are under `source.data` and the backend
entrypoint chooses how to iterate or include them. Values are Markdown-ready;
templates must not re-derive SQLite catalog semantics.

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

Add a focused directory under `tests/fixtures`, execute its `schema.sql`
against a real temporary database in `tests/sqlite.rs`, and snapshot the public
source result. Update this matrix in the same change so implementation status and
fixture evidence stay synchronized.

## Contract test

```sh
just test-integration-backend sqlite
```
