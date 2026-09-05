# Full multi-backend showcase

This project demonstrates dbmd's complete composition model with six ordered
sources: SQLite, DuckDB, PostgreSQL, ClickHouse, MySQL, and MariaDB. It renders
both a nested directory artifact and a single-file artifact from the same live
database context.

Requirements: `dbmd`, `just`, `sqlite3`, the DuckDB 1.5.4 CLI, Docker, and
Docker Compose. The containers are intentionally substantial because this
example shows the complete product rather than a minimal tutorial.

```sh
just render
just verify
```

The recipe recreates both embedded databases, starts all four exact-version
servers, applies every source's committed SQL automatically, and renders both
configured layouts. Inspect `dbmd.toml` for nested multi-source directory
output, `dbmd.single-file.toml` for the alternate file, and the committed
`database/` and `DATABASE.md` artifacts without running anything.

`just down` removes the six disposable database states. All credentials are
obviously fake local values and are excluded from generated artifacts.
