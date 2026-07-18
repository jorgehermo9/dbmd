# Architecture Overview

Status: Phase 1 structure implemented; driver, config, render-context, and artifact-writer boundaries remain target architecture.

## Principles

### Explicit semantics

Normalization computes backend behavior that affects query correctness or performance. Renderers receive explicit values and provenance rather than reimplementing backend rules in template conditionals.

### Typed where behavior differs

Common schema concepts share a model. Backend extensions remain typed when their meaning affects behavior. Complex SQL expressions may remain raw strings until dbmd needs structural understanding.

### Determinism before presentation

Stable ordering and computed semantics are established before templates run. Templates choose layout; they do not repair unstable catalog order or infer hidden defaults.

### Templates are an external boundary

Internal Rust types can evolve in response to drivers. A dedicated render context will isolate public template compatibility from those changes.

### Narrow vertical slices

SQLite proves the end-to-end architecture before new abstraction crates or generalized driver frameworks are introduced. PostgreSQL and ClickHouse then deepen the model with real edge cases.

## Current workspace

```text
crates/
  core/      normalized schema-model sketch
  render/    embedded MiniJinja renderer
  cli/       command parsing and orchestration bootstrap
```

Package names remain prefixed (`dbmd-core`, `dbmd-render`) while directories stay concise.

Current implementation:

- `core` contains one `DatabaseSchema` and common/backend-specific object structs.
- `render` serializes core structs, adds a few computed fields, and renders two embedded templates.
- `cli` creates a placeholder PostgreSQL table in memory and prints the result.

No current module parses config, connects to a database, normalizes catalog rows, writes an artifact, or verifies drift.

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
                        └→ ordered project snapshot
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

## Primary boundaries

### Configuration boundary

Parsing, environment expansion, defaults, CLI precedence, source selection, and compatibility validation produce a resolved contract. Commands consume this result rather than independently interpreting config.

### Driver boundary

A driver owns database I/O and catalog-specific row types. Catalog queries and compatibility logic do not leak into core normalization or rendering.

### Normalization boundary

Normalization maps raw backend metadata to source snapshots, establishes deterministic order, and records observed/effective/unknown facts.

### Render boundary

The render-context builder computes presentation-ready facts and qualified names. MiniJinja renders an in-memory file set with strict undefined behavior.

### Artifact boundary

Output writing and verification operate on a common in-memory artifact representation: one file or a relative-path-to-bytes map. Render writes it atomically; verify compares it without modifying the canonical destination.

## Crate evolution

Do not create a crate solely because architecture diagrams name a boundary. During the SQLite slice:

- Config and SQLite driver modules may begin inside `cli` if their APIs remain testable.
- `core` owns normalized snapshots and semantic helpers without database dependencies.
- `render` owns render-context construction, template loading, and in-memory artifact generation.

Extract `config`, `drivers`, or test-support crates when a second consumer, a second backend, or compile-time dependency pressure proves the seam.

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
- [Product feature specifications](../product/features/README.md)
- [Architecture decisions](../adr/README.md)
