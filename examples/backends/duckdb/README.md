# DuckDB backend showcase

This project demonstrates schemas, enums, structs, unions, alias types,
sequences, generated columns, nested values, constraints, indexes, views, and
scalar and table macros in DuckDB.

Requirements: `dbmd`, `just`, and the DuckDB 1.5.4 command-line client.

```sh
just render
just verify
```

The recipes recreate `runtime/warehouse.duckdb` from the committed SQL before
each operation. The generated artifact is `DATABASE.md`; `just down` removes
the disposable database.
