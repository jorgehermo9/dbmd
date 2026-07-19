# PostgreSQL introspection

This module reads PostgreSQL catalogs through a connection URL and produces one
normalized `dbmd_core::SourceSnapshot<postgres::Catalog>`. Catalog queries use `pg_catalog`
directly where `information_schema` would discard backend semantics.

`Config` owns the committed PostgreSQL source fields and resolves the connection
URL through application-supplied environment expansion before constructing a
credential-redacting `PostgresSource`.

## Coverage

The adapter currently preserves:

- User schemas and schema comments.
- Ordinary, partitioned, partition, inherited, and foreign-table relation
  kinds; tablespaces when explicitly assigned.
- Columns in ordinal order with formatted types, nullability, defaults,
  identity mode, generated expressions, enum labels, and comments.
- Primary, unique, foreign-key, check, and exclusion constraint categories,
  including the server-normalized definition, validation, locality,
  inheritance, deferrability, match mode, and referential actions.
- Index keys and expressions, effective ordering/null placement, qualified
  collations and operator classes, access method, predicate, included columns,
  `NULLS NOT DISTINCT`, validity/readiness, clustering, replica identity, and
  the complete server-normalized definition.
- First-class enum types and values.
- Views and materialized views with columns, comments, and definitions.
- Functions with overload-safe signatures, return type, language, volatility,
  parallel safety, security mode, comments, and definitions.
- Traditional inheritance, partition keys/parents/bounds, row-level security,
  forced RLS, and policies.
- Triggers with multiple events and `UPDATE OF` columns, timing,
  row/statement orientation, `WHEN`, comments, called function and arguments,
  enablement state, constraint-trigger metadata, transition tables, parent
  trigger identity, and server-normalized definitions.
- Backend-owned render mapping and PostgreSQL source templates for both output
  layouts.

## Fixture evidence

Container-backed fixtures live under `tests/fixtures/postgres/`:

- `ordinary_table` — columns, identities, generated values, comments, checks,
  and expression/partial indexes.
- `relationships` — composite keys and fully specified foreign keys.
- `schema_objects` — schemas, enums, views, materialized views, and functions.
- `table_semantics` — inheritance, partitioning, RLS, and policies.
- `indexes_and_constraints` — covering indexes, opclasses, null semantics,
  clustering, replica identity, and constraint enforcement state.
- `triggers` — row and statement triggers, multi-event ordering, view and
  constraint triggers, transition tables, enablement modes, arguments, comments,
  predicates, and cloned partition-trigger parent identity.

## Template context

PostgreSQL source entrypoints receive the common `source` envelope documented
by the [template product contract](../../../../docs/product/features/templates.md).
`source.data` has this PostgreSQL-owned shape:

| Field | Type | Meaning and order |
| --- | --- | --- |
| `section_heading` | string | `##` without source nesting, `###` with nesting. |
| `object_heading` | string | Heading used by single-file object templates. |
| `detail_heading` | string | Heading used for table subsections in a single file. |
| `namespaces` | namespace[] | User schemas in binary name order. |
| `enums` | enum[] | Enum types in schema/name order. |
| `tables` | table[] | Relations in schema/name order, including partitions and foreign tables. |
| `views` | view[] | Ordinary and materialized views together in schema/name order. |
| `triggers` | trigger[] | Triggers in target schema, target relation, and trigger-name order. |
| `functions` | function[] | Functions in schema, name, and overload-safe signature order. |

PostgreSQL directory objects are declared in enums, tables, views, triggers,
then functions order. Their directories are `enums/`, `tables/`, `views/`,
`triggers/`, and `functions/`. Trigger identity and filenames include their
target relation; function filenames include the identity-argument signature.

Presentation object fields are:

- Namespace: `name`, nullable `comment`.
- Enum: `qualified_name`, `file_name`, nullable `comment`, and `values`.
- Table: `qualified_name`, `file_name`, nullable `comment`, `columns`,
  `constraints`, `indexes`, and `backend`. `backend.title` is `PostgreSQL`;
  `backend.facts` contains kind, storage/partition/inheritance details and RLS
  policies; `backend.notices` carries enabled/forced RLS state;
  `backend.definition` is currently null.
- Column: `name`, `data_type`, `nullable` (`yes`, `no`, or `unknown`),
  `default` (`-` when absent), and `notes` for comments, identity, generated
  expressions, and enum labels.
- Constraint: `name`, `kind`, `columns`, and `details`; details preserve the
  normalized definition plus validation/inheritance state.
- Index: `name`, `terms`, `unique`, `origin`, and `predicate`; `origin` includes
  PostgreSQL access method, included columns, null-distinctness, validity,
  readiness, clustering, and replica-identity facts.
- View: `qualified_name`, `file_name`, nullable `comment`, `facts` containing
  `view` or `materialized_view`, `columns`, and fenced `definition`.
- Trigger: `qualified_name`, `file_name`, nullable `comment`, combined `event`,
  `target`, `facts`, nullable `when_expression`, and fenced `definition`.
  Facts preserve orientation, function, enablement, arguments, constraint and
  transition-table metadata, and nullable parent-trigger identity.
- Function: `qualified_name`, `file_name`, nullable `comment`, `facts`, and
  nullable fenced `definition`. Facts preserve return type, language,
  volatility, parallel safety, and security mode.

In a directory object template the selected object is `object`; `heading`,
`detail_heading`, and `source` are also available. In a single-file source
template the same objects are under `source.data` and the backend entrypoint
chooses how to iterate or include them. Values are Markdown-ready; templates
must not re-derive PostgreSQL catalog semantics.

Procedures, domains, composite/range types, sequences, extensions,
foreign-server metadata, publications, and privileges are not yet modeled.
Raw server definitions remain a fidelity backstop for represented objects; this
matrix does not claim every PostgreSQL DDL family.
