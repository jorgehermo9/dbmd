# Configuration and CLI Architecture

## Canonical config

The default file is `dbmd.toml`. Configuration uses one named multi-source
shape and one configured output:

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

[sources.local.attachments.analytics]
path = "./analytics.db"

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

There is no `[source]` shorthand, YAML config, or `[outputs.<name>]` map.

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

Avoid passing partially resolved config deep into introspection or rendering modules.

## Environment expansion

Support exact `${NAME}` references in committed connection strings and backend fields that may contain secrets. Missing variables fail before connection.

Rules must be simple and documented:

- No shell execution.
- No command substitution.
- No implicit `.env` loading.
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

## Introspection dispatch

SQLite and PostgreSQL expose concrete adapters in `dbmd-introspect`. Their proven
shared behavior is represented by a closed source enum and one dispatch
function, without a driver trait:

```rust
pub enum Source { Sqlite(SqliteSource), Postgres(PostgresSource) }
pub fn introspect(source: &Source) -> Result<SourceSnapshot, IntrospectionError>;
```

A generic backend trait is not part of this boundary because application
orchestration needs closed dispatch, not runtime-extensible drivers. The stable behavior is
introspection producing a normalized `SourceSnapshot`; dynamic dispatch is not
itself a product requirement.

Adapters use synchronous clients. Runtime choice is internal and not part of
the public seam.

## Output-path validation

Resolve relative output paths against a documented project base, normally the config file's directory. Normalize lexically and inspect filesystem state without following an output-root symlink into an unsafe destination.

Directory replacement rejects repository root, home, `.git`, and other nonsensical broad targets. Use explicit path types and validated resolved paths rather than string comparisons scattered across commands.

The writer treats directory output as fully owned. It must avoid deleting arbitrary paths when config values, symlinks, or relative traversal are involved.

## Application and CLI modules

The CLI parses command-specific values, converts them to application requests, calls one operation, and presents its report or error. It does not read project configuration, connect to databases, render templates, or write artifacts itself.

Application interfaces are operation-oriented and intentionally small:

```rust
pub fn render(request: RenderRequest) -> Result<RenderReport, RenderError>;
pub fn verify(request: VerifyRequest) -> Result<VerifyReport, VerifyError>;
pub fn explain(request: ExplainRequest) -> Result<ExplainReport, ExplainError>;
pub fn doctor(request: DoctorRequest) -> DoctorReport;
```

`dbmd-app` may internally compose:

- Config loader.
- Resolver and validators.
- Source planner.
- Introspection dispatch.
- Snapshot normalization.
- Renderer.
- Artifact writer/comparator.

Application operations decide which capabilities to invoke and return structured
results. CLI commands decide only how to present those results. Explain reuses
resolved plans without database access. Doctor reuses local preflight and only
dispatches introspection when connections are explicitly enabled. This keeps
doctor, verify, and lint from becoming modes of one oversized command function.

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

Credentials are redacted at construction, not through hopeful formatting discipline at the final display seam.

## Path bases

Configured paths, including CLI output/template overrides, resolve relative to
the selected config file. Configless SQLite paths resolve relative to the
process working directory.
