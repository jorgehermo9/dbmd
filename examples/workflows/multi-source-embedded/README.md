# Embedded multi-source project

This project combines a transactional SQLite database and a DuckDB analytics
warehouse in one ordered agent-readable artifact. It demonstrates stable source
IDs, presentation-only display names, explicit selection order, and source
sections in a single file.

Requirements: `dbmd`, `just`, `sqlite3`, and the DuckDB 1.5.4 CLI.

```sh
just render
just verify
```

Both databases are recreated automatically from their source-specific schema
directories. Inspect `dbmd.toml` to see that `analytics` intentionally renders
before `app`. `just down` removes both disposable files.
