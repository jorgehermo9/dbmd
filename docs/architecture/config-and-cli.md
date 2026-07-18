# Configuration and CLI Architecture

Status: accepted product shape; not implemented.

## Canonical config

The default file is `dbmd.toml`. MVP supports one named multi-source shape and one configured output:

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

[output]
path = "DATABASE.md"
profile = "agent"
sources = ["analytics", "app"]

[output.layout]
kind = "single_file"
source_layout = "auto"

[templates]
dir = "templates/dbmd"
```

There is no `[source]` shorthand, YAML config, or `[outputs.<name>]` map in MVP.

## Parsing stages

Keep representations distinct:

1. Parsed config preserves optional fields and source table keys.
2. CLI overrides are parsed without mutating config structures.
3. Environment references are identified and expanded into connection-only values.
4. Resolution applies precedence and built-in defaults.
5. Validation produces command-specific plans.

Directional types:

```rust
struct ProjectConfig { /* committed values */ }
struct CliOverrides { /* one-off values */ }
struct ResolvedProject { /* defaults and expanded connections */ }
struct RenderPlan { /* selected sources, templates, destination */ }
```

Avoid passing partially resolved config deep into drivers or renderers.

## Environment expansion

Support exact `${NAME}` references in committed connection strings and backend fields that may contain secrets. Missing variables fail before connection.

Rules must be simple and documented:

- No shell execution.
- No command substitution.
- No implicit `.env` loading in MVP unless explicitly adopted later.
- Errors name variables but never expanded values.
- Redacted diagnostics preserve enough non-secret structure to identify a source.

## Selection and precedence

General precedence is CLI, then config, then built-in defaults. Source selection has explicit deterministic semantics documented in [multiple sources](../product/features/multi-source.md).

Canonical commands differ from exploratory commands:

| Command | Output-shaping overrides | Writes canonical artifact |
|---|---:|---:|
| `render` | yes | unless `--stdout` or alternate output is selected |
| `explain` | yes | no |
| `verify` | no | no |
| `doctor` | operational scope only | no |
| `lint` | policy/scope only | no |

`--config` is operational and may apply across commands.

## Shared preflight

Factor common validation while preserving command-specific reporting:

```text
local preflight
  config parses and validates
  source IDs are valid
  selected sources exist and are non-empty
  environment references resolve
  layout and flags are compatible
  template set resolves and required entrypoints exist
  output path is safe for the requested operation

source preflight
  selected source can connect
  backend version is supported
  required introspection queries are permitted
```

Render and verify fail fast enough to avoid partial work. Doctor may continue independent checks to provide a fuller diagnosis.

## Driver dispatch

The first backend can use direct SQLite dispatch without committing to an async trait. Directional options are:

```rust
enum BackendDriver {
    Sqlite(SqliteDriver),
    Postgres(PostgresDriver),
    ClickHouse(ClickHouseDriver),
}
```

or a trait once common async behavior is proven. The stable boundary is a driver producing a normalized `SourceSnapshot`; dynamic dispatch is not itself a product requirement.

Avoid async runtime adoption until PostgreSQL or ClickHouse clients require it. A sync SQLite vertical slice should not pay that cost preemptively.

## Output-path validation

Resolve relative output paths against a documented project base, normally the config file's directory. Normalize lexically and inspect filesystem state without following an output-root symlink into an unsafe destination.

Directory replacement rejects repository root, home, `.git`, and other nonsensical broad targets. Use explicit path types and validated resolved paths rather than string comparisons scattered across commands.

The writer treats directory output as fully owned. It must avoid deleting arbitrary paths when config values, symlinks, or relative traversal are involved.

## Command modules

CLI parsing should produce command-specific input types. Orchestration modules may share:

- Config loader.
- Resolver and validators.
- Source planner.
- Driver dispatch.
- Snapshot normalization.
- Renderer.
- Artifact writer/comparator.

Commands decide which capabilities to invoke and how to report outcomes. This keeps `doctor`, `verify`, and `lint` from becoming modes of one oversized command function.

## Error taxonomy

Use structured internal errors with user-facing context:

- Config read/parse/schema.
- Missing environment.
- Invalid source or selection.
- Template resolution/compilation/execution.
- Unsafe output path.
- Connection and authentication.
- Unsupported backend version.
- Introspection permission/query.
- Normalization invariant.
- Output write/replace.
- Verification drift.

Credentials are redacted at construction, not through hopeful formatting discipline at the final display boundary.

## Open implementation decisions

- Config modules inside `cli` versus a crate after reuse appears.
- Relative path base for explicit `--config` and one-off CLI usage.
- Async boundary timing.
- Exact atomic directory replacement behavior across operating systems.
