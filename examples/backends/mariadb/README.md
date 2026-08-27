# MariaDB backend showcase

This project demonstrates MariaDB sequences, system-versioned and bitemporal
tables, generated and invisible columns, vector indexes, constraints,
partitions, views, multi-event triggers, routines, packages, roles, grants,
events, and credential-safe server metadata.

Requirements: `dbmd`, `just`, Docker, and Docker Compose.

```sh
just render
just verify
```

The recipes start MariaDB 12.3.2 with the required authentication plugin and
initialize it automatically from the read-only `schema/commerce/` mount.
`just down` removes all example state.
