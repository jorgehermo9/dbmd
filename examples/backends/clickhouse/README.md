# ClickHouse backend showcase

This project demonstrates ClickHouse databases, MergeTree-family engines,
keys, partitions, TTLs, codecs, projections, skip indexes, dictionaries,
materialized and window views, SQL functions, access-control objects, named
collections, resources, and workloads.

Requirements: `dbmd`, `just`, Docker, and Docker Compose.

```sh
just render
just verify
```

The recipes start ClickHouse 26.6.1.1193 and initialize it from the read-only
`schema/analytics/` mount. The documented credentials are intentionally local
and never appear in generated Markdown. `just down` removes all example state.
