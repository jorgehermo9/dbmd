# MySQL backend

This crate owns MySQL connection configuration, catalog introspection,
presentation mapping, and embedded templates. Its catalog deliberately does not
double as the MariaDB catalog.

The executable compatibility target for this coverage matrix is MySQL 9.7.1,
tested with the official Community `mysql:9.7.1` image. Enterprise-only
catalog families are documented separately until they can be proved against an
Enterprise fixture.

## Public interface

- `Config` resolves a URL, optional schema scope, display name, and opt-in
  global-object coverage through `include_global_objects`.
- `MysqlSource` and `introspect` expose concrete MySQL catalog access without a
  driver trait.
- `render_source` and `template_files` expose MySQL-owned presentation to the
  composition root.

## Coverage

The executable Community schema surface includes schemas, base tables, SQL and
JSON relational duality views, columns (including generated, invisible,
spatial, vector, and masking-policy-marked columns), defaults, character sets,
collations, comments, primary/unique/foreign/check constraints, ordinary,
functional, full-text, spatial, and hash indexes, partition/subpartition
metadata, table options, triggers, stored procedures/functions and parameters,
and scheduled events. `SHOW CREATE` definitions are retained as the fidelity
backstop. The 9.7.1 fixture also proves generated invisible primary keys,
enforced inline foreign keys with an implicit parent primary key, trigger
ordering, one-time and recurring events, and JSON relational duality metadata.
Volatile storage statistics and object-history timestamps are intentionally
excluded.

Global objects are opt-in because they require broader privileges and produce a
server-wide catalog. That surface includes credential-safe server definitions,
application-defined spatial reference systems, InnoDB tablespaces, resource
groups, loadable functions, every built-in and dynamically loaded plugin,
installed components, accounts, authentication-factor metadata, role/default
role graphs, and global/schema/table/column/routine/proxy privileges. Passwords,
authentication strings, raw account JSON, and tablespace file locations are
excluded at acquisition time.

| Schema surface | Represented facts |
| --- | --- |
| Schemas | Name, default character set/collation, default encryption, and read-only state. |
| Tables | Engine, row format, collation, create options, comment, engine attributes, partition/subpartition properties, and normalized `SHOW CREATE`. |
| Columns | Position, full declared type including `VECTOR(N)`, nullability, default, generated expression, invisibility, SRS ID, masking-policy presence, engine attributes, character set/collation, enum values, and comment. |
| Constraints | Primary, unique, foreign, and check constraints, including semantic match/referential actions, enforcement, and engine attributes. |
| Indexes | Ordered columns/expressions, semantic sort order, prefix lengths, uniqueness, type, visibility, comments, and disabled reason. |
| Views | SQL versus JSON relational duality kind, check/updatability/security/definer metadata, query, and normalized `SHOW CREATE` definition. |
| Stored objects | Complete normalized routine/trigger/event definitions plus semantic kind, data-access, security, event/timing/status/completion, schedule, parameter-direction, SQL-mode, character-set, collation, and imported-library metadata. |
| Global objects | Servers, custom SRS definitions, tablespaces, resource groups, loadable functions, plugins, components, credential-safe accounts/authentication factors, role graphs, and grants. |

Closed native values are normalized during introspection into backend or shared
semantic enums. For example, index `A`/`D` codes become ascending/descending,
account `Y`/`N` flags become booleans, and event, trigger, security, resource,
plugin kind/state/load-policy/license, loadable-function kind/return family,
JSON duality-view validity, TLS, privilege-scope, and referential-action values
become typed facts.
Loadable functions come from Performance Schema so component- and
plugin-registered functions are represented in addition to `CREATE FUNCTION`
entries. Rendering supplies human-readable labels. An unknown value in a
closed 9.7.1 set is an introspection error rather than an `unknown` catalog
variant. Engine names, SQL types, component URNs, plugin names, and privilege
names remain strings because those are identities or extensible vocabularies.

MySQL 9.7.1 Enterprise additionally documents JavaScript/WebAssembly libraries
and routines, OpenID Connect authentication, and dynamic masking policy
definitions. The adapter consumes the edition-neutral `LIBRARIES`,
`ROUTINE_LIBRARIES`, account-plugin, and column `MASKING POLICY` surfaces when
present. Policy definitions themselves require the Enterprise
`component_object_policy`, its private registry, and
`MANAGE_DATA_MASKING_POLICY`; the Community contract therefore does not claim
that proprietary object family. NDB logfile groups and NDB-specific tablespace
properties likewise belong to the separate NDB Cluster product rather than the
Community Server contract.

## Template context

`source.data` contains the heading fields plus every catalog collection named
above. Tables use the shared presentation fields plus a `MySQL` backend block
for engine, partition, and creation facts. Other object values expose qualified
name, file name, nullable comment, facts, and an optional fenced definition.
Directory output gives every represented object a collision-safe object file.

All values are Markdown-ready and deterministically ordered. See the
[common template envelope](../../../docs/product/features/templates.md).

## Contract test

```sh
just test-contract mysql
```
