# SQLite backend showcase

This project demonstrates dbmd's SQLite schema surface: strict and
`WITHOUT ROWID` tables, generated columns, named constraints, foreign-key
actions, expression and partial indexes, views, triggers, virtual tables, and
schema evolution reflected in final catalog state.

Requirements: `dbmd`, `just`, and a SQLite 3.53.3 command-line client.

```sh
just render
just verify
```

The disposable database is recreated from `schema/app/` on every command. The
generated single-file artifact is committed as `DATABASE.md`; run `just down`
to remove local state.
