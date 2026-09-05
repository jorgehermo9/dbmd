# SQLite quickstart

This is the shortest complete dbmd project: one SQLite source, the embedded
agent profile, and one committed `DATABASE.md`.

Requirements: `dbmd`, `just`, and the `sqlite3` command-line client.

```sh
just render
just verify
```

The recipe recreates `runtime/app.db` from `schema/app/01-schema.sql`; no setup
step is required. Inspect `dbmd.toml` for the project contract and
`DATABASE.md` for the generated agent-readable artifact. Run `just down` to
remove the disposable database.
