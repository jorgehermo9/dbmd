# Architecture Overview

Status: Phase 2 vertical slice implemented through the core, introspection, rendering, application, and thin CLI modules.

## Principles

### Explicit semantics

Normalization computes backend behavior that affects query correctness or performance. Renderers receive explicit values and provenance rather than reimplementing backend rules in template conditionals.

### Typed where behavior differs

Common schema concepts share a model. Backend extensions remain typed when their meaning affects behavior. Complex SQL expressions may remain raw strings until dbmd needs structural understanding.

### Determinism before presentation

Stable ordering and computed semantics are established before templates run. Templates choose layout; they do not repair unstable catalog order or infer hidden defaults.

### Templates are a compatibility seam

Internal Rust types can evolve in response to introspection adapters. A dedicated render context will isolate public template compatibility from those changes.

### Narrow vertical slices

SQLite proves the end-to-end architecture before generalized backend interfaces are introduced. PostgreSQL and ClickHouse then deepen the model with real edge cases.

## Current workspace

```text
crates/
  core/      normalized schema-model sketch
  introspect/ concrete SQLite introspection
  render/    embedded MiniJinja renderer
  app/       config, orchestration, and atomic single-file output
  cli/       thin command parsing and report presentation
```

Package names remain prefixed (`dbmd-core`, `dbmd-render`) while directories stay concise.

Current implementation:

- `core` contains validated source identity, `SourceSnapshot`, `DatabaseContext`, and common/backend-specific schema-object types.
- `introspect` covers SQLite's persistent DDL/schema surface through catalog metadata plus stored-definition parsing.
- `render` renders tables, constraints, indexes, views, triggers, virtual/shadow tables, and raw definitions with embedded templates.
- `app` resolves named SQLite sources and environment-backed paths, coordinates rendering, and atomically replaces one Markdown file.
- `cli` maps `render --config` to the application operation and presents its report.

Base `init`, exact `verify`, multi-source presentation, directory layouts, and a
versioned dedicated presentation context are implemented. Custom templates,
one-off overrides, CI initialization, and additional backends remain.

## Target workspace

```text
crates/
  core/        pure domain model, invariants, and deterministic normalization
  introspect/  database I/O and backend-specific metadata interpretation
  render/      render-context construction and in-memory artifact generation
  app/         config resolution, operation orchestration, and safe output policy
  cli/         argument parsing and user-facing presentation
```

The dependency direction is:

```text
cli → app → core
          ├→ introspect → core
          └→ render → core
```

`core` is pure but not an anemic struct collection: it owns domain constructors, invariants, semantic helpers, and deterministic collection ordering. It does not know TOML, environment variables, filesystem paths, database clients, CLI arguments, or templates.

## Target data flow

```text
CLI arguments ─┐
               ├→ resolved project contract
dbmd.toml ─────┘        │
                        ├→ selected source plans
                        │        │
                        │        ├→ backend connection
                        │        ├→ raw catalog rows
                        │        └→ normalized source snapshot
                        │
                        └→ database context
                                  │
                                  ├→ render context
                                  ├→ selected template set
                                  └→ in-memory artifact
                                            │
                                            ├→ stdout
                                            ├→ atomic output replacement
                                            └→ temporary verification comparison
```

The resolved contract carries no expanded secrets into render contexts or diagnostics.

## Primary modules and seams

### Application module

`dbmd-app` presents a small operation-oriented interface to the CLI. It owns parsing, environment expansion, defaults, CLI precedence, source selection, operation planning, failure semantics, and artifact output policy. Commands consume resolved plans rather than independently interpreting configuration.

The initial render interface is directional rather than a compatibility promise:

```rust
pub fn render(request: RenderRequest) -> Result<RenderReport, RenderError>;
```

### Introspection module

`dbmd-introspect` owns database I/O, catalog-specific row types, backend-version capability handling, and conversion into a normalized `SourceSnapshot`. Catalog queries and compatibility logic do not leak into application orchestration, core domain types, or rendering.

The first SQLite interface is concrete:

```rust
pub fn introspect(source: SqliteSource) -> Result<SourceSnapshot, IntrospectionError>;
```

Do not introduce a generic backend trait for one adapter. A shared backend seam should be designed only after a second concrete adapter proves what actually varies.

### Normalization seam

Normalization maps raw backend metadata to source snapshots, establishes deterministic order, and records observed/effective/unknown facts.

### Render module

The render-context builder computes presentation-ready facts and qualified names. MiniJinja renders an in-memory file set with strict undefined behavior.

### Artifact seam

Output writing and verification operate on a common in-memory artifact representation: one file or a relative-path-to-bytes map. Render writes it atomically; verify compares it without modifying the canonical destination.

## Crate responsibilities

- `core` owns `DatabaseContext`, `SourceSnapshot`, schema objects, domain invariants, semantic helpers, and deterministic normalization without I/O dependencies.
- `introspect` owns SQLite connection/catalog behavior and returns core types. Internal query and row-mapping modules remain implementation details.
- `render` accepts core domain values and rendering choices, builds a dedicated presentation context, and returns an in-memory artifact.
- `app` owns configuration and filesystem dependencies, coordinates the other modules, and exposes deep operation interfaces to the CLI.
- `cli` maps Clap values to application requests and presents reports or errors. It contains no database or rendering behavior.

Config and output do not become separate crates during Phase 2. They are cohesive internal modules of `app` until real consumers or dependency pressure prove another seam.

## Command responsibility

- `init` scaffolds project-owned files.
- `render` creates and replaces an artifact.
- `verify` compares a fresh temporary artifact with the canonical artifact.
- `doctor` diagnoses operational readiness.
- `explain` reports resolution and planning.
- `lint` evaluates schema policy.

Shared preflight and introspection internals do not blur these user-facing responsibilities.

## Security model

- Committed config references environment variables rather than containing credentials.
- Expanded secrets stay in connection construction and are redacted from errors.
- Template contexts never contain DSNs or environment values.
- Output paths are validated before destructive replacement.
- Verify writes only to temporary locations.

## Related documents

- [Schema model](schema-model.md)
- [Rendering](rendering.md)
- [Configuration and CLI](config-and-cli.md)
- [Testing](testing.md)
- [SQLite introspection coverage](../../crates/introspect/src/sqlite/README.md)
- [Product feature specifications](../product/features/README.md)
- [Architecture decisions](../adr/README.md)
