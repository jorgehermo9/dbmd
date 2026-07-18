# dbmd-introspect

`dbmd-introspect` reads backend catalogs and converts their structural metadata
into normalized `dbmd-core` source snapshots.

The crate owns database I/O, backend-specific catalog interpretation, capability
handling, and deterministic conversion. It does not own configuration loading,
source selection, rendering, output files, or CLI presentation.

## Interface

Each backend exposes a concrete source and introspection function. SQLite
provides `sqlite::SqliteSource` with ordered configured attachments; PostgreSQL
provides a connection-backed `postgres::PostgresSource`. The second adapter is
now available to prove a small shared dispatch seam without requiring a driver
trait.

## Backend coverage

- [SQLite](src/sqlite/README.md)
- [PostgreSQL](src/postgres/README.md)

Backend documentation beside its implementation is the live coverage contract:
it records what the adapter can currently observe, what it preserves in the core
model, and which fixtures prove that behavior. Product promises remain in the
[product documentation](../../docs/product/overview.md), while cross-crate
boundaries remain in the [architecture documentation](../../docs/architecture/overview.md).

## Tests

Integration tests create real temporary databases from SQL fixtures and snapshot
the public `SourceSnapshot` result:

```text
tests/
  fixtures/sqlite/<case>/schema.sql
  snapshots/
  sqlite.rs
```

Run this crate's suite with:

```sh
cargo test -p dbmd-introspect
```

The opt-in PostgreSQL suite starts one shared container and gives each fixture
an isolated logical database:

```sh
cargo test -p dbmd-introspect --features postgres-tests --test postgres
```
