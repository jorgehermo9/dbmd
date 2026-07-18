# Architecture Overview

## Principles

### Preserve backend semantics

SQLite and PostgreSQL may use the same noun for concepts with different
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
  core/       source identity envelopes and aggregate invariants
  backends/   compiled backend composition and vertical backend modules
    src/sqlite/    catalog, introspection, render mapping, templates
    src/postgres/  catalog, introspection, render mapping, templates
  render/     presentation types, common templates, and artifact assembly
  app/        config resolution, operations, verification, and safe output
  cli/        argument parsing and report/error presentation
```

The dependency direction is:

```text
cli → app → backends → core
          │      └────→ render
          └───────────→ render
```

`render` does not depend on `core` or backend catalog types. `core` has no
database, configuration, filesystem, CLI, or template dependencies.

## Backend composition

`dbmd-backends` is both the registry of compiled-in database families and the
owner of shared relational leaf values. Its root contains only closed dispatch
types and functions. Concrete semantics stay in submodules:

```text
backends/src/
  lib.rs             Source, Catalog, Backend, dispatch
  relational.rs      equivalent relational leaf values
  sqlite/
    catalog.rs
    introspect.rs
    render.rs
    templates/
    README.md
  postgres/
    catalog.rs
    introspect.rs
    render.rs
    templates/
    README.md
```

Adding a compiled-in backend adds one sibling module and wires it into the
closed root. It does not add vendor variants to `dbmd-core` or imports to
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
  → backend-owned RenderSource mapping
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

### Backends

Each backend owns source connection inputs, catalog queries, normalization,
catalog types, render mapping, backend template fragments, fixture tests, and a
coverage matrix beside the code. The root converts concrete snapshots into a
closed composition catalog for application use.

### Render

`dbmd-render` owns the versioned presentation structs, Markdown escaping and
code fencing, strict MiniJinja execution, safe artifact-relative paths, and the
single-file/directory in-memory artifact. It receives already-prepared
`RenderSource` values and template manifests.

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
- [SQLite backend coverage](../../crates/backends/src/sqlite/README.md)
- [PostgreSQL backend coverage](../../crates/backends/src/postgres/README.md)
- [ADR-0005](../adr/0005-backend-owned-catalogs-and-templates.md)
