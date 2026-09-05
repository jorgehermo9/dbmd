# Architecture Overview

## Principles

### Preserve backend semantics

Database families may use the same noun for concepts with different
catalog shape and behavior. Backend modules normalize those facts into their
own typed catalogs instead of flattening them into one universal relational
model. Raw SQL definitions remain a fidelity backstop where structured catalog
fields are incomplete.

### Keep the composition boundary explicit

dbmd supports compiled-in backends. One closed composition root knows which
backend modules are present and dispatches configuration, introspection,
presentation mapping, and template manifests. Core identity and the rendering
engine remain independent of that list.

### Determinism before presentation

Catalog adapters establish stable ordering and effective semantics before
templates run. Templates choose layout; they do not repair catalog order or
infer database rules.

### Deep operation interfaces

The CLI is a presentation layer over small application operations. The
application resolves configuration and coordinates backend and artifact
modules without exposing their internal steps to callers.

## Workspace

```text
crates/
  core/              source identity envelopes and aggregate invariants
  backends/
    composition/     closed built-in backend registry and dispatch
    relational/      equivalent relational leaf and presentation values
    sqlite/          SQLite vertical backend implementation
    postgres/        PostgreSQL vertical backend implementation
    clickhouse/       ClickHouse vertical backend implementation
    mysql/            MySQL vertical backend implementation
    mariadb/          MariaDB vertical backend implementation
    duckdb/           DuckDB vertical backend implementation
  render/            presentation engine, common templates, and artifact assembly
  app/               config resolution, operations, verification, and safe output
  cli/               argument parsing and report/error presentation
```

The dependency direction is:

```text
cli → app
app → backends/composition, core, render
backends/composition → concrete backend crates, core, render
concrete backend → core, relational where applicable, render, its database client
relational → render
```

`render` does not depend on `core` or backend catalog types. `core` has no
database, configuration, filesystem, CLI, or template dependencies.

## Backend composition

`dbmd-backends` is the registry of compiled-in database families. Its crate
contains only closed dispatch types and functions. Concrete semantics stay in
sibling backend crates, while equivalent relational support stays separate:

```text
backends/
  composition/       SourceConfig, Source, Catalog, Backend, dispatch
  relational/        equivalent relational leaf and presentation values
  sqlite/
    src/catalog.rs
    src/introspect.rs
    src/render.rs
    src/templates/
    README.md
  postgres/
    src/catalog.rs
    src/introspect.rs
    src/render.rs
    src/templates/
    README.md
  clickhouse/        same vertical ownership, using ClickHouse HTTP catalogs
  mysql/             same vertical ownership, using MySQL catalogs
  mariadb/           same vertical ownership, using MariaDB catalogs
  duckdb/            same vertical ownership, using embedded DuckDB catalogs
```

Adding a compiled-in backend adds one sibling crate and wires it into the closed
composition crate. It does not add vendor variants to `dbmd-core` or imports to
`dbmd-render`. A public driver trait and runtime plugin ABI are intentionally
absent until a concrete runtime-extension consumer exists.

## Data flow

```text
CLI + dbmd.toml
  → resolved application plan
  → selected backend Source values
  → backend introspection
  → SourceSnapshot<backend Catalog>
  → composition DatabaseContext<Catalog enum>
  → backend-owned presentation data + object manifest
  → RenderSource envelope
  → versioned RenderContext
  → common + selected backend template manifests
  → in-memory RenderedArtifact
  → stdout, atomic replacement, or verification comparison
```

Template completeness and output-path safety are validated before connection
where selected backend manifests are known from configuration. Expanded secrets
never enter catalogs, render contexts, diagnostics, or generated artifacts.

## Module responsibilities

### Core

`dbmd-core` owns `SourceId`, `SourceSnapshot<C>`, and `DatabaseContext<C>`.
These types preserve stable identity and source order and reject empty or
duplicate aggregates. The generic catalog parameter prevents core from
registering every database family.

### Backend crates

Each backend owns source connection inputs, catalog queries, normalization,
catalog types, render mapping, backend template fragments, fixture tests, and a
coverage matrix beside the code. `dbmd-relational` owns only leaf vocabulary and
presentation helpers whose meaning is equivalent across multiple backends.

### Backend composition

`dbmd-backends` converts concrete snapshots into a closed composition catalog
for application use. It owns the compiled backend list and heterogeneous
dispatch, but no database driver, catalog query, or backend template body.

### Render

`dbmd-render` owns the versioned presentation envelope, Markdown escaping and
code fencing, strict MiniJinja execution, safe artifact-relative paths, and the
single-file/directory in-memory artifact. It receives already-prepared
`RenderSource` values containing opaque backend data and generic object
manifests. It has no table/view/trigger/function branches.

### Application and CLI

`dbmd-app` owns parsing, environment expansion, defaults, source selection,
preflight, orchestration, atomic output, and verification. `dbmd-cli` maps Clap
values to operation requests and presents structured results. Neither command
code nor application orchestration reaches into concrete catalog fields.

## Related documents

- [Schema model](schema-model.md)
- [Rendering](rendering.md)
- [Configuration and CLI](config-and-cli.md)
- [Testing](testing.md)
- [SQLite backend coverage](../../crates/backends/sqlite/README.md)
- [PostgreSQL backend coverage](../../crates/backends/postgres/README.md)
- [ClickHouse backend coverage](../../crates/backends/clickhouse/README.md)
- [MySQL backend coverage](../../crates/backends/mysql/README.md)
- [MariaDB backend coverage](../../crates/backends/mariadb/README.md)
- [DuckDB backend coverage](../../crates/backends/duckdb/README.md)
- [ADR-0005](../adr/0005-backend-owned-catalogs-and-templates.md)
