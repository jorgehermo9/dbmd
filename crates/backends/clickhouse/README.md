# dbmd ClickHouse backend

This crate owns ClickHouse source configuration, catalog semantics,
introspection, presentation mapping, templates, fixtures, and tests.

The executable compatibility target for this coverage matrix is ClickHouse
26.6.1.1193 (the latest vendor-documented stable build), tested with the exact
official `clickhouse/clickhouse-server:26.6.1.1193` image.

## Public interface

- `Config` resolves HTTP URL, optional database scope, credentials, and display
  name without exposing expanded values to rendering.
- `ClickHouseSource` is a concrete HTTP source; `introspect` returns a
  deterministic `SourceSnapshot<Catalog>`.
- `render_source` and `template_files` expose the ClickHouse-owned
  presentation and embedded source entrypoints to the composition root.

## Coverage

The fixture-backed schema surface includes databases, tables, dictionaries,
views, materialized views and their targets, columns and default kinds,
MergeTree engine arguments and named parameters,
partition/sorting/primary/sampling/unique keys, storage policies, column codecs
and statistics, typed table and column TTL contracts, projections including
projection indexes, data-skipping indexes, constraints, comments,
dependencies, parameterized-view metadata, refreshable materialized views,
and SQL user-defined functions. It also covers SQL-managed users, roles,
grants, row policies, quotas, settings profiles, named collections, resources,
and workloads. The latest storage fixture proves CoalescingMergeTree, ALP,
sharded Map serialization settings, QBit, and S3 `storage_class_name`.

Closed server vocabularies are normalized at acquisition. Column default kinds
and check/assume constraint semantics are typed, and an unrecognized value
fails with source, operation, field, and native-value context. Engine names,
index implementations, codecs, settings, access targets, and SQL remain open
strings because ClickHouse and its integrations can extend those vocabularies.

Raw creation SQL remains the fidelity backstop for engine settings and
expressions that ClickHouse exposes textually. Volatile rows, bytes, parts,
replication health, mutations, and query statistics are not canonical schema
context.

| Schema surface | Represented facts |
| --- | --- |
| Databases | Name, UUID, engine/full engine expression, external-catalog status, and comment. Server filesystem paths are intentionally excluded. |
| Tables | Database/name, UUID, engine family/full engine expression and ordered arguments, temporary status, comment, raw creation SQL, storage policy, key expressions, dependencies, and load ordering. Effective engine settings are structured in deterministic name order. Every table TTL action is typed (delete/predicate, disk or volume movement, recompression, and group-by assignments), and column TTL expressions are attached to their columns. External-engine credentials are server-redacted as `[HIDDEN]`; the exact-version fixture proves their absence from catalog and Markdown. Reserved `.inner.*` and `.inner_id.*` tables owned by views are excluded in favor of the owning view contract. |
| Dictionaries | Table identity plus typed key/attribute names and types, default and source expressions, `HIERARCHICAL`/`INJECTIVE`/`IS_OBJECT_ID` modifiers, layout, range bounds, refresh lifetime, dictionary settings, and the server-provided non-secret source description. Declared metadata is recovered from the server-normalized definition when an unloaded dictionary reports empty runtime fields. Credential-bearing source clauses remain in the normalized definition only after ClickHouse replaces the value with `[HIDDEN]`; the fixture proves the original value never appears. Runtime status, load failures, bytes, hit rates, and update timestamps are excluded as operational state. |
| Views and materialized views | Raw definition, view kind, parameters, definer, SQL-security mode, dependencies, `AS SELECT`, and materialized-view target where present. Refreshable materialized views expose `EVERY`/`AFTER`/dependency-only schedules, offset, randomization, dependencies, append/replace mode, and refresh settings. Runtime refresh status and timestamps are excluded. `POPULATE` is an initialization action and is removed from the normalized definition after creation, so it is not claimed as current schema state. |
| Window views | Target, inner and result storage engines, watermark strategy, allowed-lateness expression, query, and output columns. The exact fixture enables the documented experimental window-view setting and the required legacy analyzer. `LIVE VIEW` creation is rejected by the 26.6.1.1193 parser and is not a target capability. |
| Columns | Position, type, character/numeric/datetime precision, default kind/expression, comment, codec, serialization hint, statistics declaration, TTL expression, and key roles. |
| Constraints and indexes | Check/assume expressions, explicit/implicit data-skipping index origin, expression/type/granularity, aggregate projections, projection indexes, and projection-local settings. |
| SQL and WebAssembly UDFs | Name, catalog origin, creation definition, and server-provided syntax/arguments/return type. SQL lambda functions do not expose inferred signatures. The official arm64 26.6.1.1193 image reports `SUPPORT_IS_DISABLED` for WebAssembly and has no `system.webassembly_modules`, so executable WASM fixture evidence requires a WASM-enabled official build. |
| Users and roles | SQL-managed identity, storage, authentication method names, credential expiry, host rules, default roles/database, grantee bounds, role grants, and admin-option semantics. Authentication parameters are never acquired. `users.xml` bootstrap identities are excluded because they are configuration, not SQL DDL. |
| Privileges and row policies | Grants/partial revokes, object scope, grant option, permissive/restrictive mode, SELECT filter, and application target. |
| Quotas | Keying and address-prefix semantics, application target, interval randomization, and every limit exposed by ClickHouse 26.6, including `queries_per_normalized_hash`. |
| Settings profiles | Application target plus ordered values, bounds, writability, and inherited profiles. |
| Named collections | Name, source, sorted key names, and each key's overridability. Values are never selected. ClickHouse 26.6 server-redacts every value in `create_query` as `[HIDDEN]`; only that redacted definition crosses acquisition, and the exact fixture guards it with a secret sentinel. |
| Workload scheduling | Resources with typed master-thread, worker-thread, query, memory-reservation, and read/write-disk operations plus their unit and normalized definition; workloads with parent hierarchy and ordered, optionally resource-scoped settings. The exact 26.6.1.1193 runtime supports the CPU/query/IO surface. Memory reservations were backported upstream for 26.6.2 and are not claimed for this target build. |

Masking policies are documented for ClickHouse 26.6 but are ClickHouse Cloud
only. The 26.6 open-source server has no `system.masking_policies` catalog, so
the executable contract does not claim masking-policy coverage. Cloud catalog
acquisition needs a separate contract environment before this surface can be
represented safely.

## Template context

`source.data` contains `section_heading`, `object_heading`, `detail_heading`,
`namespaces`, `tables`, `views`, `functions`, and `access_objects`. Tables use the shared
presentation fields plus a `ClickHouse` backend detail block containing engine,
key, TTL, projection, skip-index, and settings facts and fenced creation SQL.
Views retain their kind, materialized target, and definition. Functions and
access/workload objects expose qualified name, file name, facts, and optional
definition. Directory objects are emitted in table, view, function, then
access/workload order.

All values are Markdown-ready and all collections retain catalog order. See the
[common template envelope](../../../docs/product/features/templates.md).

## Contract test

Run the real-server contract with:

```sh
just test-integration-backend clickhouse
```
