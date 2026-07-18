# Product Roadmap

Status: Phases 1 and 2 complete; Phase 3 is the active milestone.

This roadmap orders product capability. It is not a promise that every later feature ships in the first public release.

## Phase 1 — Bootstrap

Status: complete.

Delivered:

- Cargo workspace with `core`, `render`, and `cli` crates.
- Product and architecture direction.
- Common schema-model sketch with backend extension types.
- Embedded MiniJinja renderer with strict undefined behavior.
- Placeholder `render` command.
- Small semantic and renderer tests.

The placeholder renderer does not introspect a database or write a canonical artifact.

## Phase 2 — First useful SQLite render

Status: complete.

Goal: turn a real local SQLite database into a deterministic, useful `DATABASE.md`.

First tracer bullet:

```text
dbmd.toml with one named SQLite source
  → config validation
  → SQLite introspection
  → normalized source snapshot
  → embedded agent profile
  → atomic single-file write
  → golden test
```

Required SQLite coverage:

- Tables and columns.
- Declared types, nullability, and default expressions.
- Primary keys, unique constraints, checks where discoverable, and foreign keys.
- Index columns, expressions where available, uniqueness, origin, and partial status.
- Views and definitions.
- Generated and hidden columns.
- Strict tables and `WITHOUT ROWID`.
- Deterministic object and field ordering.

Supporting work:

- Minimal `dbmd.toml` parsing and environment expansion.
- Stable source ID and display-name handling.
- Database-context/source-snapshot seam for future multi-source rendering.
- Dedicated render context before custom templates become a compatibility surface.
- Snapshot or golden tests using real temporary SQLite databases.

Delivered beyond the tracer bullet: configured attached SQLite namespaces,
complete persistent SQLite DDL/schema-surface coverage, first-class views and
triggers, virtual/shadow tables, and application-level golden Markdown tests.

Multi-source presentation and directory layout were completed in the active
canonical-lifecycle milestone.

Exit criteria:

- A developer can point dbmd at a local SQLite file and commit useful Markdown.
- Repeated renders with unchanged structure are byte-identical.
- Fixture changes produce focused Markdown diffs.
- Failures identify configuration, connection, introspection, rendering, and output errors distinctly.

## Phase 3 — Canonical lifecycle and drift detection

Status: current.

- `dbmd init` for safe-to-commit configuration. Base SQLite, complete template
  tree, and protected GitHub Actions initialization are implemented.
- `dbmd verify` with byte-for-byte and file-set comparison. Implemented.
- Compact changed/added/deleted summaries. Implemented.
- `dbmd verify --diff` with a complete unified diff. Implemented.
- Multi-source rendering and deterministic selection order. Configured and
  repeated CLI selection are implemented across SQLite and PostgreSQL.
- Directory object layout for large schemas. Implemented with atomic
  whole-tree replacement.
- Generated GitHub Actions workflow. Implemented; a reusable official action
  remains a separate release/integration feature.

The render module owns an explicit versioned presentation context and a
layout-neutral in-memory artifact. Complete custom template loading, custom
profile selection, configless SQLite, stdout/output overrides, and CI generation
are implemented. Agent snippets and release hardening remain before Phase 3 is
closed.

Generated Markdown remains free of timestamps, fingerprints, versions, and generated-by headers. Verification compares fresh output rather than trusting embedded metadata.

## Phase 4 — PostgreSQL depth

Status: active; the listed fixture-driven catalog surface is implemented in the
introspection, core, render, and application layers. Remaining PostgreSQL DDL
families and operational hardening continue in this phase.

- Catalog introspection through `pg_catalog` where `information_schema` loses detail.
- Schemas, relations, columns, defaults, identities, generated columns, and comments.
- Constraints and foreign keys.
- Index methods, predicates, expressions, and included columns.
- Enums and values.
- Views and materialized views.
- Functions and signatures.
- Partitions, inheritance, and row-level security.

Exact first-release coverage will be driven by golden fixtures and agent failure cases.

## Phase 5 — ClickHouse depth

Status: planned.

- Introspection through system tables and columns.
- Engine and engine parameters.
- `ORDER BY`, explicit and effective `PRIMARY KEY`, `PARTITION BY`, and `SAMPLE BY`.
- TTLs, codecs, data-skipping indexes, and settings.
- Provenance for effective facts derived from backend defaults.

Engine parsing should remain as raw expressions until real cases justify stable typed variants.

## Phase 6 — Agent ergonomics and operations

Status: planned.

- `agent-compact` and human-oriented profiles after the default agent profile stabilizes.
- Generated `AGENTS.md` and `CLAUDE.md` snippets.
- Open skill package for navigating dbmd artifacts.
- Pre-commit integration recipe.
- `dbmd doctor` for project health.
- `dbmd explain` for resolved configuration and output planning.
- `dbmd lint` for schema quality and agent-readiness policy.
- Optional statistics or machine-readable manifests only when concrete consumers justify them.

## Prioritization rules

- Prefer an end-to-end usable slice over new abstraction crates.
- Add backend model complexity in response to fixtures, not speculation.
- Stabilize output before declaring template-context compatibility.
- Keep setup health, drift, and schema quality as separate command responsibilities.
- Do not let later CI or agent integrations block proving the core artifact.
