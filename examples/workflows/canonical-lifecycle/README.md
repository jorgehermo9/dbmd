# Canonical artifact lifecycle

This project demonstrates the committed-artifact loop. `just render` replaces
`DATABASE.md` from current SQLite structure, and `just verify` proves exact
freshness without rewriting the file.

Requirements: `dbmd`, `just`, and `sqlite3`.

```sh
just render
just verify
printf '\nmanual drift\n' >> DATABASE.md
just verify   # exits unsuccessfully and preserves the edit
just render   # restores the canonical artifact
```

Database setup remains automatic. Run `just down` to remove the disposable
database.
