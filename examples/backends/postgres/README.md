# PostgreSQL backend showcase

This project demonstrates PostgreSQL schemas, types, ranges, identity columns,
comments, constraints, indexes, sequences, routines, aggregates, views,
partitioning, inheritance, row-level security, triggers, storage properties,
foreign-data objects, access control, publications, disconnected subscriptions,
text search, event triggers, statistics, roles, and cluster metadata.

Extension-owned object reconstruction is covered by the backend compatibility
suite rather than this standard-image example because it requires a custom
extension image. The generated artifact still demonstrates built-in extension
metadata such as `btree_gist`.

Requirements: `dbmd`, `just`, Docker, and Docker Compose.

```sh
just render
just verify
```

The recipes start PostgreSQL 18.4, mount `schema/catalog/` read-only into the
image initialization directory, wait for readiness, and inject the local URL
without committing credentials. `just down` removes the example container and
volume state.
