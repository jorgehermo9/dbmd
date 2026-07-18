# dbmd Architecture

## Status

This document is a design workspace, not a frozen implementation contract. The goal is to make decisions explicit, record tradeoffs, and keep open questions visible while the product shape evolves.

Code examples in this document are sketches. They should guide discussion, not force a 1:1 implementation.

## Current Decision State

Accepted for now:

- Build dbmd as a Rust CLI.
- Generate plain Markdown artifacts first, not a server or MCP tool.
- Treat backend-specific semantics as first-class when they affect correctness.
- Prefer explicit generated output over agent inference.
- Keep crate directories concise: `crates/core`, `crates/render`, and `crates/cli`.
- Default to a single `DATABASE.md` artifact, with directory output as explicit opt-in.
- Support custom template directories as full template sets, not overlays coupled to embedded defaults.
- Support multiple named sources in MVP.

Leaning:

- Use `minijinja` for runtime templates.
- Use a common schema model with typed backend extension enums.
- Start with SQLite before PostgreSQL and ClickHouse.
- Keep package names prefixed, such as `dbmd-core`, while directories stay concise.

Open:

- Whether the final core model should expose internal structs directly to templates or build a dedicated render context.
- Whether drivers live in `crates/cli` initially or move to `crates/drivers` early.
- How much SQL expression parsing is worth doing versus preserving raw database expressions.
- How to represent observed facts, effective facts, and backend defaults in the core model.

## Design Principles

### Explicit Over Inferred

Generated docs should not make agents infer backend rules when those rules affect query correctness or performance.

ClickHouse is the motivating example. If ClickHouse derives an effective primary key from `ORDER BY`, the output should still show the effective primary key explicitly. If we care about where the value came from, that provenance should be modeled directly instead of hidden behind renderer conditions.

### Typed Semantics Where They Matter

Backend-specific metadata should be typed when it affects behavior. Examples include ClickHouse table engines, sorting keys, partition keys, TTLs, PostgreSQL partitioning, and SQLite `WITHOUT ROWID`.

Raw strings are acceptable for complex SQL expressions that dbmd does not need to understand structurally yet.

### Deterministic Output Is A Feature

The generated artifact is meant to be committed. Stable ordering, stable formatting, and stable headers matter as much as introspection coverage.

### Templates Are A Boundary

Templates are product surface area. The renderer should eventually expose a stable, documented context rather than leaking whatever shape the internal Rust structs happen to have.

### Start Narrow, Go Deep

Supporting many databases shallowly is less valuable than supporting a few databases well enough that agents stop making wrong assumptions.

## Workspace Shape

Current workspace:

- `crates/core`: domain model for schema snapshots.
- `crates/render`: template-based Markdown rendering.
- `crates/cli`: command parsing and orchestration.

Possible future crates:

- `crates/config`: config loading, env expansion, validation.
- `crates/drivers`: shared driver trait and backend implementations.
- `crates/testing`: fixtures or integration test helpers, if needed.

Package names can remain `dbmd-core` and `dbmd-render` to avoid ambiguity with Rust's `core` crate while keeping folder names concise.

## Data Flow

The intended pipeline is:

```text
config -> driver -> raw catalog rows -> normalized schema -> render context -> templates -> markdown files
```

`config` resolves backend, DSN, output path, template profile, and feature flags.

`driver` queries backend catalog tables or PRAGMA APIs.

`raw catalog rows` are backend-specific structs close to the database metadata source.

`normalized schema` is the core model used by dbmd internally.

`render context` is the template-facing shape, with computed fields and stable names.

`templates` render Markdown profiles such as `agent`, `human`, or `agent-compact`.

`markdown files` are the committed artifact.

## Core Model Direction

The core model should have common concepts plus backend-specific extensions:

```rust
pub struct Table {
    pub schema: String,
    pub name: String,
    pub columns: Vec<Column>,
    pub constraints: Vec<Constraint>,
    pub indexes: Vec<Index>,
    pub backend: TableBackend,
}

pub enum TableBackend {
    Postgres(PostgresTable),
    ClickHouse(ClickHouseTable),
    Sqlite(SqliteTable),
}
```

This remains a leaning, not an irreversible decision. It is preferred over `dyn Trait` for now because the first-party backend set is small, backend identity matters during rendering, serde is straightforward, and consumers can pattern-match without downcasting.

## Facts And Provenance

The model should distinguish at least three categories:

- Observed facts: values read directly from catalog metadata.
- Effective facts: values dbmd can state after applying documented backend rules.
- Unknown facts: values dbmd cannot determine confidently.

This matters for ClickHouse keys. A primary key can be explicit, defaulted from `ORDER BY`, or unavailable depending on what catalog metadata exposes and how dbmd interprets it.

Possible sketch:

```rust
pub struct Sourced<T> {
    pub value: T,
    pub source: ValueSource,
}

pub enum ValueSource {
    Catalog,
    BackendDefault { rule: String },
    Derived { explanation: String },
}
```

The renderer can then show the value explicitly while optionally annotating provenance when useful:

```text
Primary key: `user_id, occurred_at` (defaulted from ORDER BY)
```

We should avoid helper methods that hide semantic differences by comparing fields, such as rendering primary key only if it differs from order key.

## Backend Modeling

### ClickHouse

ClickHouse needs explicit modeling because engine metadata changes how queries should be written and interpreted.

Current sketch:

```rust
pub struct ClickHouseTable {
    pub engine: String,
    pub engine_params: Vec<String>,
    pub order_by: Vec<String>,
    pub primary_key: Vec<String>,
    pub partition_by: Option<String>,
    pub sample_by: Option<String>,
    pub ttl: Option<String>,
    pub settings: BTreeMap<String, String>,
}
```

Likely refinement:

```rust
pub struct ClickHouseTable {
    pub engine: ClickHouseEngine,
    pub order_by: Sourced<Vec<String>>,
    pub primary_key: Sourced<Vec<String>>,
    pub partition_by: Option<Sourced<String>>,
    pub sample_by: Option<Sourced<String>>,
    pub ttl: Option<String>,
    pub settings: BTreeMap<String, String>,
}
```

Open question: whether `ClickHouseEngine` should be a typed enum early or stay a string until real introspection examples force the shape.

### PostgreSQL

PostgreSQL should use `pg_catalog` where `information_schema` loses important detail.

Likely metadata areas:

- Schemas and relations.
- Columns, defaults, generated columns, identities, comments.
- Constraints and foreign keys.
- Index methods, predicates, expressions, included columns.
- Enums and enum values.
- Views and materialized views.
- Functions and signatures.
- Partitioning and inheritance.
- Row-level security policies.

Open question: how much of this belongs in the MVP versus later phases.

### SQLite

SQLite is likely first because it is easy to test locally and forces useful model decisions.

Likely metadata sources:

- `sqlite_master` or `sqlite_schema`.
- `PRAGMA table_xinfo`.
- `PRAGMA index_list` and `PRAGMA index_xinfo`.
- `PRAGMA foreign_key_list`.
- `PRAGMA table_list` for strict tables and `WITHOUT ROWID` when available.

Open question: how to handle SQLite versions that do not expose newer PRAGMA fields.

## Driver Interface

The driver boundary should isolate database I/O from the core schema model.

Sketch:

```rust
pub trait Driver {
    async fn introspect(&self, options: IntrospectionOptions) -> Result<DatabaseSchema>;
}
```

This interface is not final. Before committing to `dyn Driver`, we should decide whether the CLI can dispatch through a backend enum instead. A concrete enum may be simpler while the backend set is small.

Possible dispatch shape:

```rust
pub enum BackendDriver {
    Sqlite(SqliteDriver),
    Postgres(PostgresDriver),
    ClickHouse(ClickHouseDriver),
}
```

The important boundary is not the exact trait shape. The important boundary is that catalog query code should not leak into rendering.

## Rendering Architecture

Use embedded default templates so the binary works without files on disk.

Allow user templates from a path so teams can customize output. Custom template directories should be fully uncoupled from the builtin template tree. If a custom template directory is selected, dbmd should load the required entrypoint templates from that directory and fail clearly when they are missing.

Use strict undefined behavior so template mistakes fail loudly.

Template set selection precedence:

- CLI flag, for example `--template-dir templates/dbmd`.
- Config, for example `templates.dir = "templates/dbmd"`.
- Embedded defaults.

Template selection dimensions:

- `profile`: arbitrary profile name selected by `output.profile`; builtin profiles provide defaults.
- `layout`: selected by `output.layout.kind`.
- `artifact`: examples include `database`, `index`, `table`, `view`, and `function`.
- `backend`: available to templates for conditional rendering or internal includes, but not part of the required top-level entrypoint key.

Profiles are discovered by directory names under the selected template root. dbmd should use an opinionated template directory convention instead of requiring config to map profile names to arbitrary paths.

Example custom template root:

```text
templates/dbmd/
  agent/
    single_file/
      database.md.j2
    directory/
      index.md.j2
      table.md.j2
  internal-review/
    single_file/
      database.md.j2
```

With config:

```toml
[output]
profile = "internal-review"

[output.layout]
kind = "single_file"

[templates]
dir = "templates/dbmd"
```

dbmd resolves the main template at:

```text
templates/dbmd/internal-review/single_file/database.md.j2
```

Do not support per-profile arbitrary template paths initially, such as `[profiles.internal-review].template_dir = "..."`. That can be added later if real users need it.

Main templates should define document structure. Partials are smaller templates included by main templates for reusable or backend-specific sections. Builtin templates may use partials heavily, but partial paths are not a public contract for custom templates. A custom template set can use no partials, copy the builtin partial structure, or invent its own internal structure.

Required custom template entrypoints should be minimal and based on selected layout:

- `profile/single_file/database.md.j2` for `single_file` output.
- `profile/directory/index.md.j2` and `profile/directory/table.md.j2` for `directory` output.
- Additional directory entrypoints such as `view.md.j2` and `function.md.j2` become required only when dbmd emits those artifact types.

Example template tree:

```text
agent/
  single_file/
    database.md.j2
  directory/
    index.md.j2
    table.md.j2
    view.md.j2
    function.md.j2
  partials/
    table.md.j2
    columns.md.j2
    clickhouse/
      table_engine.md.j2
      column.md.j2
    postgres/
      table_backend.md.j2
      column.md.j2
    sqlite/
      table_backend.md.j2
      column.md.j2
```

For default single-file ClickHouse output, the renderer would use `agent/single_file/database.md.j2` as the main template and include backend-specific partials such as `agent/partials/clickhouse/table_engine.md.j2` from inside table rendering.

For a custom template directory, dbmd should only require the selected entrypoint templates. Any partials referenced by those templates are owned by the custom template set.

The selected template set should be validated before dbmd connects to the database. Missing templates are local configuration errors and should fail before opening remote or production connections.

Target architecture:

```text
DatabaseSchema -> RenderContext -> minijinja -> Markdown
```

The current bootstrap may render serialized core structs directly. That is acceptable temporarily, but the target is a stable render context with documented fields.

Examples of render-context-only fields:

- `qualified_name`
- `engine_clause`
- `primary_key.note`
- `column.display_type`
- `constraint.human_summary`

Business semantics should generally be computed before rendering. Templates should mostly choose layout.

## Output Layout

Two layouts are under consideration:

```text
DATABASE.md
```

```text
database/
  index.md
  tables/public.users.md
  tables/analytics.events.md
```

Single-file output is simple and easy to commit.

Directory output is better for large schemas and lets agents read one table at a time.

Decision: default to `single_file`, writing `DATABASE.md`. Directory output is available through explicit config. dbmd should not adaptively change layout based on schema size because that creates surprising diffs when schemas grow.

Canonical layout config uses an `[output.layout]` table. Do not support shorthand such as `layout = "directory"` in MVP.

Single file:

```toml
[output]
path = "DATABASE.md"
profile = "agent"

[output.layout]
kind = "single_file"
source_layout = "auto"
```

Directory objects:

```toml
[output]
path = "database"
profile = "agent"

[output.layout]
kind = "directory"
variant = "objects"
source_layout = "auto"
```

Directory sections:

```toml
[output]
path = "database"
profile = "agent"

[output.layout]
kind = "directory"
variant = "sections"
source_layout = "auto"
```

`source_layout = "auto" | "nested"` controls how the selected layout represents multiple sources. Default to `auto`.

When `kind = "directory"`, `variant` is optional and defaults to `objects`. The `objects` variant is the default because it is more agent-friendly: agents can read one table, view, or function without loading a large section file. The `sections` variant is available for teams that prefer fewer files.

For `single_file` output:

- `auto` omits a source heading when exactly one source is selected.
- `auto` includes source headings when multiple sources are selected.
- `nested` includes source headings even when one source is selected.

When multiple sources are selected with `single_file`, dbmd should still write one file. The generated document should contain clear source sections:

```md
# Database Schema

## Source: analytics

...

## Source: app

...
```

For `directory` output:

- `auto` omits source directories when exactly one source is selected.
- `auto` groups by direct source-name directories when multiple sources are selected.
- `nested` groups by direct source-name directories even when one source is selected.

Single source with `auto`:

```text
database/
  index.md
  tables/users.md
```

Multiple sources with `auto`:

```text
database/
  index.md
  app/
    index.md
    tables/users.md
  analytics/
    index.md
    tables/events.md
```

Single source with `nested`:

```text
database/
  index.md
  app/
    index.md
    tables/users.md
```

## Config Architecture

Config should separate committed settings from secrets.

The canonical config file is `dbmd.toml`. `dbmd init` should generate that filename. A `--config path` flag can point elsewhere, but dbmd should not add YAML or multiple default config names without a concrete need.

MVP config should model connection-backed schema sources. Use `sources`, not `databases`, because `database` collides with backend vocabulary such as ClickHouse database names.

Use only the named multi-source shape. Do not support `[source]` shorthand in MVP. One canonical config shape avoids migration and precedence questions.

Backend-specific connection fields live directly inside each `[sources.<name>]` table. A nested `[sources.<name>.connection]` table is unnecessary while all MVP sources are connection-backed.

Do not model multiple source kinds yet. Non-connection sources such as SQL dumps, generated snapshots, or command output are deferred until there is concrete need.

Named sources are first-class in MVP:

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
```

The source stable ID comes from the table key, such as `analytics`, `app`, or `local`. Sources may define an optional `display_name` for generated headings. If omitted, generated output should use the source key. `display_name` is presentation-only and must not affect file paths, CLI source selection, source identity, or verification identity.

Source names must be filesystem-safe slugs because they are used in config selection, CLI flags, generated headings, and directory paths. MVP validation should allow ASCII letters, numbers, `_`, and `-`, and reject names containing path separators, whitespace, or punctuation that would require slugification. Slugifying arbitrary names would create hidden mapping rules and collision risks.

`dbmd doctor` and `dbmd lint` are distinct commands:

- `doctor` validates dbmd setup health: config parsing, source names, env vars, template availability, output writability, connection ability, introspection permissions, and backend version compatibility.
- `lint` audits schema quality and agent-friendliness: missing table or column comments, tables without primary keys, foreign-key-like columns without constraints, missing FK indexes, suspicious ClickHouse ordering keys, undocumented views, and similar rules.

Keep lint separate so projects can run setup checks and drift checks without immediately enforcing schema-quality policy.

Shared command preflight should be factored internally so lifecycle commands fail consistently:

```text
shared preflight:
  config parses
  config schema is valid
  selected sources exist and are non-empty
  source names are valid slugs
  required environment variables are present
  selected template set resolves
  required template entrypoints exist
  resolved output layout and flags are compatible
  selected sources can connect
  selected sources can run required introspection queries

verify:
  shared preflight
  render canonical output to a temporary location
  compare temporary output to committed artifact

doctor:
  shared preflight
  extra diagnostics
  richer explanations
  optional broader checks

lint:
  shared preflight or a lint-specific subset
  introspect selected schema
  evaluate configured schema-quality rules
```

Command responsibilities:

- `doctor`: operational correctness.
- `verify`: artifact freshness.
- `lint`: schema quality.

This boundary should stay clean even if the commands share validation, connection, introspection, and reporting internals. `doctor` should be a superset of verify's preflight validation, not a superset of `verify`. Drift comparison belongs to `verify`; setup diagnosis belongs to `doctor`; schema policy belongs to `lint`.

### Doctor

`dbmd doctor` answers: can dbmd successfully operate in this project?

It should validate setup and execution prerequisites without turning schema-quality opinions into blockers.

Checks include:

- Config file exists and parses.
- Source names are valid filesystem-safe slugs.
- Referenced sources exist.
- Required environment variables are present.
- Selected template set exists.
- Required template entrypoints exist.
- Templates compile under strict undefined behavior when possible.
- Output path is writable or can be created.
- Backend driver can connect when connection checks are enabled.
- Required database permissions exist for introspection.
- Backend version is compatible with required metadata queries, such as SQLite PRAGMAs used by dbmd.

`doctor` should be useful before `render` and before `verify`. CI may run `doctor` to fail fast on setup problems.

By default, `doctor` should check the sources selected by the canonical output, matching `render` and `verify`. `doctor --all-sources` should broaden checks to every configured source, including sources not currently selected by `output.sources`.

### Lint

`dbmd lint` answers: is this database schema well documented and agent-friendly?

Lint is product scope, but it should be separate from MVP render/verify. It needs rule configuration, severities, allowlists, and backend-specific nuance.

Example lint checks:

- Missing table comments.
- Missing column comments.
- Columns with ambiguous names like `status` without enum, check, or comment values.
- Foreign-key-looking columns without foreign key constraints.
- Foreign keys without useful indexes, where applicable.
- Tables without primary keys.
- ClickHouse tables missing explicit `ORDER BY` or using suspicious `ORDER BY tuple()`.
- ClickHouse `ReplacingMergeTree` tables without clear version or deleted semantics.
- PostgreSQL enum or check values not documented in output.
- Views without comments.
- Functions without comments or volatility metadata.

Why keep `doctor` and `lint` separate:

- `doctor` is about setup health.
- `lint` is about schema quality.
- CI may want `doctor` and `verify`, but not strict `lint` initially.
- Lint rules need configuration, severities, allowlists, and backend-specific nuance.
- Keeping them separate avoids turning “why won’t dbmd run?” into “your database is badly documented.”

dbmd should support both committed config and flag-driven one-off usage. Config is the normal path for repeatability and CI, but users should be able to try dbmd without creating a config file first.

Precedence:

- CLI flags override config.
- Config provides project defaults.
- Builtin defaults fill missing optional values.

MVP supports one configured output. Use canonical `[output]` and `[output.layout]` tables. Do not support `[outputs.<name>]` in MVP.

This is a flagship product decision: the config defines the project's canonical database artifact. Power users can generate alternate outputs by overriding output settings with CLI flags, such as output path, profile, selected sources, layout kind, or directory variant. Those one-off renders should not force every project to carry multiple configured outputs.

General command design principle:

- Config defines the canonical project contract.
- CLI flags are for one-off execution, exploration, debugging, and local overrides.
- Lifecycle and CI-oriented commands should operate on canonical config by default.
- Output-shaping CLI overrides are primarily for `render` and `explain`, not for canonical drift verification.
- Operational flags such as `--config` may still apply where useful.

`dbmd render` should use normal config resolution: CLI flags override config, then dbmd writes to the resolved output destination. A plain `dbmd render` writes the configured output path. If the user supplies output-shaping overrides such as profile, selected sources, layout kind, directory variant, source layout, output path, or template dir, those overrides apply to the render and the result is written to the resolved output path unless `--stdout` is explicitly requested.

`--stdout` is an output destination, not a layout. It is valid for single-file renders. It should be rejected for multi-file layouts such as directory output unless dbmd later introduces an explicit archive or manifest stdout mode. Incompatible flag combinations should fail during validation before connecting to any database.

`output.sources` is optional. If omitted, the output renders all configured sources. If present, the output renders only the listed source names. CLI `--source` flags override output source selection for one-off runs.

Source ordering is part of the product behavior:

- If `output.sources` is present, render sources in the listed order.
- If `output.sources` is omitted, render all configured sources sorted by source key.
- If CLI `--source` flags are provided, render sources in flag order.

This gives users control when they ask for it while preserving deterministic output by default.

An explicit empty source selection, such as `output.sources = []`, is invalid for `render` and `verify`. Empty output is almost certainly a configuration mistake.

Current leaning:

```toml
[sources.app]
backend = "postgres"
url = "${DATABASE_URL}"

[output]
path = "DATABASE.md"
profile = "agent"
sources = ["app"]

[output.layout]
kind = "single_file"
source_layout = "auto"
```

Environment expansion should be simple and predictable. Missing variables should produce clear errors before any connection attempt.

One-off usage should be possible:

```sh
dbmd render --backend sqlite --path ./dev.db --output DATABASE.md
```

Equivalent config:

```toml
[sources.app]
backend = "sqlite"
path = "./dev.db"

[output]
path = "DATABASE.md"
profile = "agent"
sources = ["app"]

[output.layout]
kind = "single_file"
source_layout = "auto"
```

## Verification And Drift

`verify` should generate output to a temporary location, compare it to committed output, and fail with actionable information.

`verify` should run shared preflight validation first. It should validate enough setup state to make the drift result trustworthy, but it should not replace `doctor`'s broader diagnostics.

`verify` should not create, delete, or modify configured output paths. It may create temporary directories/files for regenerated output, then compare those temp artifacts against committed output. Missing committed output remains a drift failure.

If the configured output artifact is missing, `verify` should report a drift failure, not a setup failure. Missing output means the committed artifact is not up to date with the configured project contract. The fix is to run `dbmd render` and commit the generated output.

For directory layouts, extra stale files under the configured dbmd-owned output path should also be drift failures. If a table, view, function, or source disappears, its generated documentation must disappear too; stale docs are dangerous for agents. This cleanup responsibility applies only inside the configured output path, not arbitrary repository files.

`render` should treat a directory-layout output path as fully dbmd-owned. Before rendering directory output, dbmd should remove the configured output directory and recreate it from scratch. This is simpler and safer than trying to infer stale generated files one-by-one, and it guarantees that `dbmd render` followed by `dbmd verify` does not fail because of stale files.

Users should not point directory output at a mixed-content directory. dbmd should document clearly that directory output is destructive and fully owns the configured path. The user is responsible for choosing an appropriate output path.

`render` should not prompt before wiping directory output, even in interactive terminals. Prompts make automation worse and create inconsistent behavior. The config is the contract.

dbmd should refuse only truly dangerous or nonsensical directory output paths:

- Empty path.
- `.`.
- `..`.
- `/`.
- Repository root.
- User home directory.
- `.git` or any path inside `.git`.
- Symlink output paths.

Do not special-case common mixed-content directories such as `docs`, `src`, or `.github`. If the user configures those paths, they are responsible for that tool usage.

Single-file output follows the same ownership principle. The configured output file is dbmd-owned and should be overwritten unconditionally by `render`, except when the resolved path is dangerous or nonsensical. dbmd should not try to detect whether the existing file looks generated.

`render` should create missing parent directories automatically for both single-file and directory outputs, after output path validation succeeds.

`render` should use best-effort atomic writes. For single-file output, render to a temporary file and rename it over the target. For directory output, render to a temporary sibling directory, then replace the configured output directory. This avoids leaving half-written files or empty/partial directory output if introspection or template rendering fails.

Do not require a manifest in MVP. `verify` should compare actual generated files in a temporary directory against committed files. A manifest could drift separately from Markdown, add review noise, invite agents to read machine metadata instead of schema docs, and is not needed while directory output is fully dbmd-owned.

Do not emit a generated-by header comment in Markdown by default. The generated artifact should be optimized for agents reading schema context, and ownership is enforced by `verify` plus documented workflow rather than repeated boilerplate in every generated file.

Do not include timestamps in committed generated output. Timestamps create guaranteed diff churn and complicate byte-for-byte verification.

Artifact principle: generated Markdown should contain schema context only, not tool metadata noise. Do not include timestamps, fingerprints, dbmd versions, or generated-by headers in committed Markdown by default. Agents should not have to spend context budget reading metadata that does not help them understand the database.

`verify` should treat generated artifacts as opaque bytes/files. It should render fresh output to a temporary location and compare actual bytes against committed output. It should not parse Markdown headers, trust embedded fingerprints, or exclude metadata lines from comparison.

`verify` should use byte-for-byte comparison. If generated Markdown was manually edited, even in a semantically equivalent way, `verify` should fail. Deterministic rendering is the fix for formatting churn; semantic comparison is not part of MVP.

Default `verify` output should be compact and CI-friendly. When drift is found, print a concise summary with word statuses rather than single-letter codes:

```text
error: database docs are stale

Changed:
  modified  database/app/tables/users.md
  added     database/app/tables/posts.md
  deleted   database/app/tables/old_table.md

Run:
  dbmd render
```

`dbmd verify --diff` should print a full git-style unified diff. Because `--diff` is explicit, do not truncate diff output by default. Optional truncation controls can be added later for CI ergonomics.

No built-in pager is required in MVP. Users can pipe large diffs to their preferred pager, for example `dbmd verify --diff | less -R`, or redirect to a file. `--diff` changes verbosity only; drift should still exit non-zero.

Questions to settle:

- Should verify support checking only a subset of schemas or tables?

Current leaning: avoid timestamps in committed output and prefer byte-for-byte verification.

## Statistics

Statistics can help agents avoid bad queries, but they are volatile.

Stats should not live in the primary schema artifact by default. If implemented, they should be separate and clearly marked with collection source and time.

Possible stats:

- Approximate row counts.
- Table sizes.
- Column cardinality estimates.
- Last analyzed time where available.

Open question: whether stats belong in dbmd core or as a later optional command.

## Testing Strategy

Core model tests should validate small semantic helpers only when those helpers encode real domain concepts.

Renderer tests should use snapshot-style assertions once output stabilizes.

Driver tests should use real databases where possible:

- SQLite through temporary local files.
- PostgreSQL through containerized integration tests.
- ClickHouse through containerized integration tests.

Golden fixtures should include backend edge cases, not only happy paths.

## Architecture Questions To Revisit

- Should `Sourced<T>` or a similar provenance type be introduced before the first real driver?
- Should data types be strings, structured enums, or strings plus optional parsed metadata?
- Should ClickHouse table engines be typed immediately?
- Should render context be introduced now or after SQLite proves the internal model?
- Should drivers be async from day one?
- Should package names be shorter too, or only folder names?
