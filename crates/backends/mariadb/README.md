# MariaDB backend

MariaDB-specific catalog introspection and rendering. The catalog owns MariaDB
features such as sequences and system-versioned tables instead of flattening
them into MySQL-shaped metadata.

The executable compatibility target for this coverage matrix is MariaDB 12.3.2,
the latest stable release, tested with the official `mariadb:12.3.2` image.

## Public interface

- `Config` resolves a URL, optional schema scope, display name, and opt-in
  global-object coverage through `include_global_objects`.
- `MariaDbSource` and `introspect` expose concrete MariaDB catalog access.
- `render_source` and `template_files` expose MariaDB-owned presentation to
  the composition root.

## Coverage

The supported surface includes schemas, base tables, views, columns and virtual
columns, defaults, character sets, collations, comments, primary/unique/foreign
key/check constraints, ordered/prefix/ignored indexes, partitions, system-time
periods and versioning, sequences, routines and parameters, packages and
package bodies, triggers, scheduled events, and global server definitions.
`SHOW CREATE` definitions are retained as a fidelity backstop. With
`include_global_objects = true`, the catalog also includes accounts, roles,
memberships, global/schema/table/column/routine/proxy privileges, installed
plugins, and the loadable-function registry. Volatile storage statistics are
intentionally excluded.

Native catalog encodings are normalized at acquisition. Closed values such as
foreign-key actions, index collation codes, view/routine/trigger/event kinds,
event interval units, generated-column storage, partition methods, plugin
type/state/load policy/maturity/license, account TLS requirements, and the
numeric `mysql.func.ret` ABI enter the catalog as semantic enums. Privilege
targets retain their semantic object family (global, schema, table, column,
function, procedure, package, package body, or proxy account) rather than a
generic scope string. MariaDB 12
multi-event triggers enter as an ordered event collection. Render mapping owns
readable labels; unknown closed values fail introspection instead of leaking as
strings or being flattened to an `unknown` variant. Open engine, plugin name,
type name, privilege, option, identifier, and SQL values remain strings.

| Schema surface | Represented facts |
| --- | --- |
| Schemas | Name, default character set, default collation, and comment. |
| Tables | Engine, row format, collation, create options, partitions, system-versioning state/period, comment, and `SHOW CREATE`. |
| Columns | Position, type, nullability, default, virtual/generated expression, character set/collation, comment, `VECTOR(N)`, and 12.3 `XMLTYPE` fidelity. |
| Constraints and indexes | Primary, unique, foreign and check constraints; semantic foreign-key match/actions; table-scoped constraint identity; application-period `WITHOUT OVERLAPS`; ordered/prefix/ignored indexes; VECTOR `M` and `DISTANCE`. |
| MariaDB objects | Sequences retain type, numeric bounds, start, increment, cache, cycle, engine, comment, and normalized definition while excluding the volatile current value. Application-time periods, system-time periods, and bitemporal table state retain MariaDB-specific facts. |
| Other objects | Views, routines and parameters (including parameter defaults), package specifications/bodies, multi-event triggers (including `UPDATE OF` columns), scheduled events, server definitions, installed plugins, and registered loadable functions. |
| Access objects | Account/role identity, authentication-plugin names, password expiry/lifetime, lock/default-role/TLS/resource metadata, role edges, and scoped privileges. Credential material is excluded. |

The exact-version fixture proves the MariaDB 12.0-12.3 catalog deltas that are
available in the standard server image: stored-function parameter defaults,
table-scoped foreign-key names, `TRIGGERED_UPDATE_COLUMNS`, and `XMLTYPE`. It
also proves application-time and system-time periods on one bitemporal table,
temporal unique keys, configured VECTOR indexes, descending indexes, and
MariaDB 12 multi-event triggers.

Server definitions are acquired without selecting `mysql.servers.Password` or
the raw `Options` JSON. Arbitrary option names are retained, but values whose
names indicate credentials, secrets, tokens, private material, or keys are
replaced with a typed redacted value by the SQL query before they can enter
dbmd memory. `SHOW CREATE SERVER` is never used because it embeds passwords.

Account acquisition selects only non-secret columns from `mysql.user` and
server-side JSON projections of `mysql.global_priv`. Authentication strings,
password hashes, and raw privilege JSON never cross the database connection.
Routine grants use MariaDB's `mysql.procs_priv` rather than the MySQL-only
`information_schema.ROUTINE_PRIVILEGES` assumption. The database-scoped `SHOW
CREATE ROUTINE` privilege is read from its dedicated `mysql.db` flag because
MariaDB's `SCHEMA_PRIVILEGES` view does not expose it. `SHOW GRANTS` is never
used: it can include authentication verifiers in the returned statement.

This is the complete persistent-schema contract exposed by the standard
MariaDB 12.3.2 server image. Open-ended engine options and the internal members
of package specifications and bodies remain losslessly available in normalized
`SHOW CREATE` definitions instead of being forced into a false common model.
The stock image exposes an empty `mysql.func` registry, so the fixture proves
the credential-safe query and typed ABI mapping but cannot manufacture a real
loadable UDF without an external shared library. MariaDB 12.3.2 does not support
`CREATE TABLESPACE` or `CREATE LOGFILE GROUP`; those documentation placeholders
are not backend gaps. The fixture loads the optional `caching_sha2_password`
plugin at server startup and proves the 12.1 authentication-plugin delta without
retaining its synthetic verifier.

## Template context

`source.data` contains `section_heading`, `object_heading`, `detail_heading`,
`namespaces`, `tables`, `views`, `triggers`, and `functions`. The final
collection contains routines, sequences, events, packages, servers, plugins,
loadable functions, accounts, roles, memberships, and privileges with a
distinguishing `Kind` fact. Tables use the shared presentation fields plus a
`MariaDB` backend block for engine, partitioning, system-versioning, and
creation facts. Directory objects are emitted in table, view, trigger, then
function/object order.

All values are Markdown-ready and deterministically ordered. See the
[common template envelope](../../../docs/product/features/templates.md).

## Contract test

```sh
just test-contract mariadb
```
