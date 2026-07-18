# PostgreSQL introspection

This module reads PostgreSQL catalogs through a connection URL and produces one
normalized `dbmd_core::SourceSnapshot`. Catalog queries use `pg_catalog`
directly where `information_schema` would discard backend semantics.

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

## Fixture evidence

Container-backed fixtures live under `tests/fixtures/postgres/`:

- `ordinary_table` — columns, identities, generated values, comments, checks,
  and expression/partial indexes.
- `relationships` — composite keys and fully specified foreign keys.
- `schema_objects` — schemas, enums, views, materialized views, and functions.
- `table_semantics` — inheritance, partitioning, RLS, and policies.
- `indexes_and_constraints` — covering indexes, opclasses, null semantics,
  clustering, replica identity, and constraint enforcement state.

Triggers, procedures, domains, composite/range types, sequences, extensions,
foreign-server metadata, publications, and privileges are not yet modeled.
Raw server definitions remain a fidelity backstop for represented objects; this
matrix does not claim every PostgreSQL DDL family.
