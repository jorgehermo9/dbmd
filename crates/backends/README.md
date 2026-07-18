# dbmd-backends

`dbmd-backends` composes the database families compiled into dbmd while keeping
each family's catalog semantics in a vertical module.

## Ownership

Each module under `src/<backend>/` owns its source input, normalized catalog,
introspection, render-context mapping, backend templates, fixture tests, and
coverage document. The crate root owns only the closed `Backend`, `Source`, and
`Catalog` enums plus dispatch and template-manifest composition.

Shared relational leaf values live in `relational` only when their semantics
are equivalent across represented backends. Concrete table, column, constraint,
index, view, trigger, and function aggregates do not live in `dbmd-core`.

## Interface

Concrete callers may use `sqlite::introspect` or `postgres::introspect` and
receive a `SourceSnapshot` containing that backend's catalog. Application
orchestration uses the closed root `introspect` function and the composition
`DatabaseContext` alias. `render_context` delegates catalog-to-presentation
mapping back to the owning module.

This is compile-time extensibility, not a runtime plugin ABI. Adding a backend
adds one sibling module and explicit composition wiring; it does not add vendor
types to core or render.

## Backend coverage

- [SQLite](src/sqlite/README.md)
- [PostgreSQL](src/postgres/README.md)

These colocated documents are the live fixture-backed implementation coverage
contracts. User-visible promises remain in
[product documentation](../../docs/product/overview.md); cross-crate boundaries
remain in [architecture documentation](../../docs/architecture/overview.md).

## Tests

SQLite uses real temporary database files in the default suite:

```sh
cargo test -p dbmd-backends --test sqlite
```

PostgreSQL uses isolated logical databases in a shared PostgreSQL 17 container:

```sh
cargo test -p dbmd-backends --features postgres-tests --test postgres
```
