# dbmd PRD

## Product Thesis

dbmd is a Rust CLI that generates an agent-readable Markdown snapshot of a live database schema. The artifact should be committed next to the codebase so agents and humans can inspect database structure without replaying migrations or querying the database during a coding task.

The core product is not "database documentation" in the generic sense. The core product is database context optimized for code agents, with enough precision for humans to review changes in pull requests.

See `docs/architecture.md` for technical design decisions.

## Problem

Code agents usually understand databases through weak context sources:

- Migration files, which require reconstructing the final state mentally and can diverge from reality.
- Live database queries, which consume turns, require credentials, and make agent behavior less repeatable.
- Raw schema dumps, which are accurate but noisy and poorly shaped for context windows.
- Human-oriented docs, which are often stale or omit backend-specific behavior that affects query correctness.

The missing artifact is a deterministic schema snapshot that is accurate, reviewable, searchable, and compact enough for agents.

## Existing Alternatives

`tbls` is strong prior art and overlaps heavily with the basic idea. dbmd should not compete by rebuilding every `tbls` feature. dbmd should compete by being agent-first, explicit about backend semantics, and careful about token-efficient output.

Raw `pg_dump --schema-only`, `sqlite .schema`, and ClickHouse `SHOW CREATE TABLE` are useful inputs, but they are not enough as the final product because they do not provide a navigable, normalized, cross-table context artifact.

MCP-based approaches can be useful later, but the initial product should work as plain files read by normal agent file tools.

## Target Users

- Developers using code agents in applications with real databases.
- Teams that want `DATABASE.md` or `database/` committed next to `README.md`, `DESIGN.md`, and `AGENTS.md`.
- Reviewers who want schema changes to appear as clean pull request diffs.
- Agent authors who need a deterministic database context artifact.

## Goals

- Generate Markdown from the actual current database schema.
- Preserve backend-specific semantics that affect correctness or performance.
- Produce deterministic output suitable for git diffs and CI verification.
- Keep secrets out of committed config.
- Support templates so teams can tune output for agents, humans, or CI.
- Start with a small set of backends and go deep enough to be useful.
- Provide batteries-included tooling around the generated artifact so teams can adopt and maintain it without inventing their own lifecycle.

## Non-Goals

- Replace migration tools.
- Become a live MCP server in the initial product.
- Reimplement all `tbls` features before proving the agent-first format.
- Store volatile production statistics in the primary schema artifact by default.
- Pretend all databases fit one universal SQL dialect.

## Product Workflow

The intended top-level flow is:

```sh
dbmd init
dbmd render
dbmd verify
```

`dbmd init` creates safe-to-commit config and optionally writes default templates for customization.

`dbmd render` connects to the configured database, introspects schema metadata, and writes Markdown output.

`dbmd verify` regenerates into a temporary location and fails if committed output is stale.

## Functional Requirements

- `render` must work from config without interactive prompts.
- `render` must support writing to a single Markdown file or a directory layout once that product decision is made.
- `verify` must be deterministic enough for CI.
- Official CI integration must make drift checks easy to install.
- Config must allow credentials to come from environment variables.
- Generated output must include generator version and enough source identity to diagnose stale docs.
- Generated output must avoid requiring agents to infer hidden backend defaults when those defaults affect query shape.
- Source rendering order must be deterministic and controllable: explicit `output.sources` order is preserved, while omitted source selection renders all sources sorted by source key.

## Output Requirements

The default agent profile should prefer compact Markdown with explicit semantics:

- Tables, columns, constraints, indexes, views, materialized views, and functions where supported.
- Multiple configured sources separated by clear source sections in single-file output.
- Column type, nullability, default expression, comments, and backend-specific annotations.
- Foreign keys written with referenced schema, table, and columns.
- Index details that inform query shape.
- ClickHouse engine, `ORDER BY`, effective `PRIMARY KEY`, `PARTITION BY`, `SAMPLE BY`, TTL, codecs, and settings.
- PostgreSQL schemas, enums, materialized views, partitions, row-level security, functions, and extensions where practical.
- SQLite tables, indexes, foreign keys, generated columns, strict tables, and `WITHOUT ROWID`.

Generated Markdown should contain schema context, not tool metadata noise. By default, dbmd should not prepend timestamps, fingerprints, generator versions, or generated-by headers to committed Markdown. Agents should not have to spend context budget reading metadata that does not help them understand the database.

Statistics should be optional and separate from the primary schema artifact, for example `DATABASE.stats.md`, because row counts and cardinalities change often and create noisy diffs.

## Batteries-Included Tooling

dbmd should ship a complete adoption path, not just a schema renderer. The product should make it easy to create, update, verify, and teach agents how to use the generated database artifact.

Planned tooling:

- Official GitHub Action for installing dbmd and running `dbmd verify` in CI.
- Generated CI workflow from `dbmd init ci`, starting with GitHub Actions.
- Generated `AGENTS.md` and `CLAUDE.md` snippets that instruct agents where the database artifact lives and when to read it.
- Open agent skill package that teaches compatible agents how to navigate dbmd output, prefer the generated artifact over migrations, and request regeneration when stale.
- Pre-commit hook recipe for local drift checks before pushing.
- `dbmd doctor` command to validate config, templates, environment variables, output paths, and selected sources without needing to complete a render.
- `dbmd lint` command to audit schema quality and agent-friendliness, separate from setup health checks.
- `dbmd init-templates` command to copy builtin template sets into a project for customization.
- `dbmd explain` command or subcommand that prints how dbmd resolved config, selected sources, layout, templates, and output files.
- Machine-readable manifest, for example `DATABASE.manifest.json`, when useful for tooling that needs file lists, source metadata, and hashes without parsing Markdown.

The GitHub Action and generated agent instructions are part of the product surface because they close the lifecycle loop: generate the artifact, keep it fresh, and make agents reliably consume it.

## Configuration Direction

Configuration should be declarative and safe to commit. The canonical config file is `dbmd.toml`:

```toml
[sources.analytics]
display_name = "Analytics"
backend = "clickhouse"
url = "${CLICKHOUSE_URL}"
database = "default"

[sources.app]
backend = "postgres"
url = "${DATABASE_URL}"

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

Secrets should be referenced through environment variables. dbmd should not encourage storing passwords directly in config.

The config file defines one canonical output artifact. Power users can generate alternate outputs by overriding output settings with CLI flags rather than by defining multiple persistent outputs in config.

General product rule: config defines the canonical project contract. CLI flags are for one-off execution, exploration, debugging, and local overrides. Lifecycle commands should operate on the canonical config by default.

## Product Roadmap

### Phase 1: Bootstrap

- Cargo workspace.
- Product and architecture docs.
- Core schema model sketch.
- Embedded template renderer sketch.
- Placeholder CLI command.

### Phase 2: First Useful Backend

- SQLite introspection.
- Default Markdown output that is useful on a real local database.
- Deterministic ordering.
- Basic renderer tests.
- Named source config.

SQLite is the likely first backend because it requires no external service and will force the model to handle tables, indexes, foreign keys, generated columns, and views.

### Phase 3: Drift Detection

- `verify` command.
- Stable generated headers.
- Clear CI failure messages.
- GitHub Action for running drift checks.
- Generated GitHub Actions workflow.

### Phase 4: PostgreSQL Depth

- Introspection through `pg_catalog` where needed.
- Schemas, constraints, indexes, enums, views, materialized views, functions, partitions, and row-level security.

### Phase 5: ClickHouse Depth

- Introspection through `system.tables`, `system.columns`, and index metadata.
- Explicit engine and key metadata.
- Careful handling of defaults such as primary key behavior.

### Phase 6: Agent Ergonomics

- Agent-compact output profile.
- Generated `AGENTS.md` and `CLAUDE.md` snippets.
- Open agent skill package for navigating dbmd output.
- Optional directory layout for large schemas.
- Local pre-commit hook recipe.
- `dbmd doctor` and `dbmd explain` for debugging project setup.
- `dbmd lint` for schema documentation and agent-readiness checks.

## Success Criteria

- A developer can run dbmd against a local SQLite database and commit useful Markdown in under five minutes.
- A code agent can answer basic schema questions by reading generated files without querying the database.
- Pull requests show clean diffs when schema changes.
- Backend-specific output prevents obvious wrong assumptions, especially for ClickHouse table engines and keys.
- A project can add CI drift checking and agent instructions with generated or copy-pasteable defaults.

## Open Product Questions

These questions are intentionally unresolved and should seed future `/grill-me` sessions.

### Command Behavior

- Which CLI overrides are allowed for lifecycle commands such as `verify`, `doctor`, and `lint`, versus only for one-off `render` and `explain` workflows?
- Should dbmd ever support archive or manifest stdout modes for directory layouts, or should `--stdout` remain single-file only?
- Should `dbmd verify` be strictly byte-for-byte against the configured output, or should it support a normalized comparison mode for whitespace/template-only churn?
- Should `dbmd explain` output human text, JSON, or both?

### Output Layout

- Should `directory` output support both `objects` and `sections` variants in MVP, or should `sections` be deferred?
- What exact files should `directory` + `objects` emit for tables, views, materialized views, functions, enums, extensions, and source indexes?
- What exact files should `directory` + `sections` emit, and how large can a section file become before it stops being useful for agents?
- Should `output.layout.source_layout = "nested"` affect only source wrappers/directories, or also table of contents structure and generated links?
- Should generated paths include schema names in filenames by default, for example `tables/public.users.md`, to avoid collisions?

### Templates

- Should custom templates be part of the public compatibility surface before `1.0`, or supported with an explicitly unstable context?
- What are the required template entrypoints for each layout and directory variant?
- Should builtin templates use partials freely while custom templates are only required to provide entrypoints?
- Should dbmd provide a command to print or validate the full render context for template authors?
- Should template errors show source spans and context paths, and how much effort is worth spending on template debugging UX?

### Sources And Config

- Should source `display_name` support template variables or remain a plain string?
- Should source names remain ASCII-only slugs, or eventually support broader Unicode with strict path-safe IDs?
- Should non-connection schema sources be supported later, such as SQL dumps, prior dbmd snapshots, command output, or generated catalog JSON?
- If non-connection sources are added, what is the config shape without compromising the current `[sources.<name>]` simplicity?
- Should source-specific include/exclude filters live under each source, under output, or both?

### Backend Coverage

- Which SQLite metadata is required for the first useful release: generated columns, strict tables, `WITHOUT ROWID`, triggers, virtual tables, FTS tables, or views?
- Which PostgreSQL metadata belongs in the first Postgres release versus later: extensions, policies, triggers, functions, enum values, partitions, publications, or privileges?
- How much ClickHouse `engine_full` parsing should dbmd do before falling back to raw expressions?
- Should ClickHouse engines become a typed enum early, or remain strings until real edge cases force a richer model?
- How should dbmd represent observed facts, effective facts, backend defaults, and unknown values in generated output?

### Agent Experience

- What should the generated `AGENTS.md` and `CLAUDE.md` snippets say exactly?
- Should dbmd ship an open agent skill, and if so which agent ecosystems should be targeted first?
- Should the agent skill teach agents to run `dbmd verify`, request regeneration, or only read the generated artifact?
- Should `agent`, `agent-compact`, and `human` builtin profiles all ship in the first public release?
- How prominent should human-oriented documentation be versus agent-compact documentation?

### Tooling And Lifecycle

- What should the official GitHub Action interface look like?
- Should `dbmd init ci` generate only GitHub Actions initially, or also support other CI systems later?
- Should `dbmd doctor` connect to databases by default, or should connection checks require an explicit flag for safety?
- Should `dbmd lint` ship as a later command with configurable rules, severities, and allowlists?
- Which lint rules should be warnings by default versus errors?
- Should pre-commit integration be generated by dbmd or documented as a copy-paste recipe?

### Statistics And Manifests

- What is the minimum useful stats artifact, if any?
- Should stats live in `DATABASE.stats.md`, a machine-readable file, or both?
- Should generated Markdown include a schema hash, and if so how do we avoid volatile metadata causing noisy diffs?
