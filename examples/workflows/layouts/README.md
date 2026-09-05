# Output layouts

This project renders one SQLite source twice: `dbmd.toml` owns the conventional
single-file `DATABASE.md`, while `dbmd.directory.toml` owns a nested
`database/` tree with stable object paths.

Requirements: `dbmd`, `just`, and `sqlite3`.

```sh
just render
just verify
```

The recipes execute both configurations after recreating the database. Compare
the compact file with the navigable directory tree, then run `just down` to
remove the disposable database.
