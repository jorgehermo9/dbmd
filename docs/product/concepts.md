# Product Concepts

The canonical definitions live in [CONTEXT.md](../../CONTEXT.md). This document explains how those terms form the product model.

## Conceptual model

```text
Project configuration
  ├── Source "app"
  │     └── introspection → Source snapshot
  ├── Source "analytics"
  │     └── introspection → Source snapshot
  └── Canonical artifact contract
        ├── selected and ordered sources
        ├── output layout
        ├── profile
        └── template set

Ordered source snapshots
  → Database context
  → Render
  → Agent-readable artifact
  → Compare with canonical artifact
  → Fresh or drifted
```

## Source identity

A source is a configured database connection. Its table key is its stable source ID:

```toml
[sources.analytics]
display_name = "Analytics"
backend = "clickhouse"
url = "${CLICKHOUSE_URL}"
database = "default"
```

`analytics` controls CLI selection, ordering references, generated paths, verification identity, and fallback headings. `display_name` is presentation-only. Changing the display name may change rendered prose but never changes source identity.

Source IDs are filesystem-safe ASCII slugs. The exact grammar is owned by the
[multiple-sources specification](features/multi-source.md#identity).

## Snapshots

A source snapshot is the normalized structural state of one source at introspection time. A database context is the ordered collection of source snapshots selected for an operation.

The distinction matters because:

- Multiple sources may use different backends.
- The same backend may appear more than once.
- Output order is part of the canonical artifact.
- Source identity must survive normalization and rendering.
- A project may configure sources that are not selected for a particular operation.

A source snapshot represents database structure, not operational history. It is not a migration plan, live connection, or long-term event log. Database context does not imply the complete state of a project; configuration, template selection, and output policy remain separate.

## Namespace and qualification

“Schema” is overloaded across supported backends. Product-level language uses namespace for the backend-defined container that qualifies an object:

- PostgreSQL schema: `public.users`.
- SQLite attached database: `main.users`.
- ClickHouse database: `analytics.events`.
- MySQL schema: `app.users`.
- MariaDB database/schema: `app.users`.
- DuckDB catalog and schema: `warehouse.analytics.events`.

Rendered qualification should preserve backend meaning. The normalized model may share a namespace field, but dbmd must not invent equivalence between backend capabilities.

## Facts and provenance

Backend semantics fall into three product categories:

- Observed: read directly from catalog metadata.
- Effective: derived by applying a documented backend rule.
- Unknown: not determinable with sufficient confidence.

Absence is not unknown. For example, “no explicit primary key” differs from “primary-key metadata unavailable,” and an effective ClickHouse primary key derived from `ORDER BY` differs from a catalog-observed explicit key.

Provenance annotations should appear only when they help agents interpret the fact. The underlying distinction must remain available to rendering regardless of presentation.

## Canonical artifact contract

Every project configuration declares one canonical artifact. The contract includes:

- Output path.
- Selected sources and their order.
- Layout and directory variant.
- Source nesting behavior.
- Profile.
- Template source.

CLI overrides can produce alternate one-off artifacts. They do not silently redefine what `verify` checks.

## Layout, profile, and template set

These concepts are independent:

- Layout determines file organization.
- Profile determines presentation policy.
- Template set supplies the rendering implementation.

`--stdout` is an output destination, not a layout. Backend identity is data available to templates, not a required top-level template dimension.

## Artifact ownership

The configured output path is dbmd-owned:

- A single-file artifact may be overwritten atomically.
- A directory artifact is replaced as a complete generated tree.
- A directory output path must never be mixed with user-maintained files.
- Verification does not modify the configured output.

Ownership is established by configuration and workflow, not by generated headers inside Markdown.

## Drift

Drift is an exact difference between a fresh render and the committed canonical artifact. It includes:

- Changed file bytes.
- Missing generated files.
- Newly required files.
- Extra stale files in a dbmd-owned directory artifact.

Semantic or whitespace-normalized comparison is outside the verification
contract. Deterministic generation is the mechanism for avoiding meaningless
churn.
