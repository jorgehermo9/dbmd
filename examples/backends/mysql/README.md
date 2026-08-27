# MySQL backend showcase

This project demonstrates MySQL tables and storage engines, generated and
invisible columns, functional and invisible indexes, spatial and vector types,
constraints, partitions, views, triggers, functions, procedures, and events.

Requirements: `dbmd`, `just`, Docker, and Docker Compose.

```sh
just render
just verify
```

The recipes start MySQL 9.7.1 and initialize the `test` schema automatically
from the read-only `schema/commerce/` mount. `just down` removes all example
state.
