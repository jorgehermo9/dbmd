# Multiple Sources

## Configuration

Sources use one canonical named shape:

```toml
[sources.analytics]
display_name = "Analytics"
backend = "clickhouse"
url = "${CLICKHOUSE_URL}"
database = "default"

[sources.app]
backend = "postgres"
url = "${DATABASE_URL}"

[sources.local]
backend = "sqlite"
path = "./dev.db"

[sources.embedded_analytics]
backend = "duckdb"
path = "./analytics.duckdb"
```

There is no singular `[source]` shorthand. Backend fields live directly inside
each source table while all supported sources are connection-backed.

Supported backend tags and committed connection fields are:

| Backend | Required fields | Optional fields |
| --- | --- | --- |
| `clickhouse` | `url` | `database`, `username`, `password`, `display_name` |
| `duckdb` | `path` | `display_name`, `secret_directory`, `extension_directory`, named `attachments` with `path` and `read_only` |
| `mariadb` | `url` | `schema`, `display_name`, `include_global_objects` |
| `mysql` | `url` | `schema`, `display_name`, `include_global_objects` |
| `postgres` | `url` | `display_name`, `include_cluster_objects` |
| `sqlite` | `path` | `display_name`, named attachments with `path` |

MySQL and MariaDB are distinct backends even though their connection URL
syntax overlaps. Their catalogs and rendered facts preserve their different
schema features.

MariaDB's `include_global_objects` defaults to `false`. When enabled, dbmd also
catalogs server definitions, accounts, roles, memberships, and privileges. The
connection must be allowed to read the corresponding `mysql` system catalogs.
Credential verifiers, authentication strings, raw server-option JSON, and
`SHOW CREATE SERVER` are never acquired.

MySQL's `include_global_objects` also defaults to `false`. When enabled, dbmd
catalogs credential-safe server definitions, spatial reference systems,
tablespaces, resource groups, loadable functions, plugins, components,
accounts, authentication-factor metadata, role graphs, and privileges.
Passwords, authentication strings, raw account JSON, and tablespace file
locations are excluded at acquisition time.

DuckDB resolves `path`, `secret_directory`, `extension_directory`, and attached
database paths relative to the configuration file after environment expansion.
`secret_directory` makes persistent secret identities visible to introspection;
dbmd never acquires `secret_string` or other credential fields. Directory paths
are connection details and never appear in catalog or rendered output.

## Identity

The table key is the stable source ID. `display_name` is optional presentation text and does not affect CLI selection, ordering references, output paths, or verification identity.

For PostgreSQL, `include_cluster_objects` defaults to `false`. When enabled it
includes cluster-wide databases, user-created roles and memberships, and
tablespaces. Password material and subscription connection strings are never
included; tablespace filesystem locations are redacted.

Source IDs accept ASCII letters, numbers, `_`, and `-`. dbmd rejects identifiers that would require slugification or could escape an output directory.

## Selection

```toml
[output]
sources = ["analytics", "app"]
```

- Omitted selection renders all configured sources sorted by source ID.
- Configured selection renders only the listed sources in list order.
- Repeated CLI `--source` flags replace config selection and preserve flag
  order.
- Empty selection and duplicate source IDs in an explicit order are invalid.

## Single-file layout

The renderer emits one document with explicit source sections when multiple
sources are selected. It preserves resolved source order.

With `source_layout = "auto"`:

- Exactly one source omits a redundant source wrapper.
- Multiple sources receive explicit source sections.

With `source_layout = "nested"`, even one source receives a source section. Multiple selected sources still produce one file.

## Directory layout

With `source_layout = "auto"`:

- One source writes objects directly beneath the output root.
- Multiple sources receive stable source-ID directories.

With `source_layout = "nested"`, source directories are always present.

```text
database/
  index.md
  app/
    index.md
    tables/public.users.md
  analytics/
    index.md
    tables/default.events.md
```

Direct source-ID directories are preferred over an extra `sources/` wrapper. Display names may appear in headings but never in paths.

## Failure isolation

The canonical render succeeds only if every selected source succeeds. Partial canonical artifacts are unsafe because they look complete. Errors identify the failing source without exposing its credentials.

Partial diagnostic modes must not write the canonical artifact.

## Source kinds outside the contract

SQL dumps, prior snapshots, command output, and catalog JSON are outside the
connection-backed source model. Supporting them requires an explicit
source-kind model rather than overloaded connection fields.
