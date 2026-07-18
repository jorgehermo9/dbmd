# Schema Model

Status: bootstrap model implemented; source aggregation, terminology cleanup, and provenance are required before the model becomes a stable internal contract.

## Responsibilities

The normalized model represents database structure after backend introspection and before presentation. It should:

- Preserve common relationships without erasing backend differences.
- Carry stable source and namespace identity.
- Distinguish observed, effective, and unknown facts when the distinction matters.
- Support deterministic traversal.
- Avoid dependencies on database drivers, CLI parsing, and templates.
- Serialize into a dedicated render context without itself becoming the template API.

## Aggregate shape

The current `DatabaseSchema` represents one unnamed source-like database. Multiple named sources require an explicit aggregate boundary.

Directional sketch:

```rust
pub struct ProjectSnapshot {
    pub sources: Vec<SourceSnapshot>,
}

pub struct SourceSnapshot {
    pub id: SourceId,
    pub display_name: Option<String>,
    pub backend: Backend,
    pub namespaces: Vec<Namespace>,
    pub views: Vec<View>,
    pub functions: Vec<Function>,
}
```

Exact nesting should be proven by SQLite and multi-source fixtures. The invariant is that source identity is explicit and survives to output paths and headings.

## Object model

Common concepts include:

- Namespace.
- Table.
- Column.
- Constraint.
- Foreign-key reference.
- Index.
- View and materialized view.
- Function and signature.
- Enum and enum values.
- Extension where supported.

Tables and columns carry common fields plus backend-specific extensions:

```rust
pub struct Table {
    pub namespace: String,
    pub name: String,
    pub comment: Option<String>,
    pub columns: Vec<Column>,
    pub constraints: Vec<Constraint>,
    pub indexes: Vec<Index>,
    pub backend: TableBackend,
}

pub enum TableBackend {
    Sqlite(SqliteTable),
    Postgres(PostgresTable),
    ClickHouse(ClickHouseTable),
}
```

The current code calls the backend extension field `engine` and its enum `TableEngine`. That terminology is misleading for PostgreSQL and SQLite and should be renamed before templates or consumers depend on it.

## Backend identifiers

Use one canonical product spelling per backend:

- `sqlite`
- `postgres`
- `clickhouse`

Serde tags, config values, diagnostics, and render contexts should not alternate between `clickhouse` and `click_house`. Rust variant naming remains idiomatic without defining external spelling.

## Facts and provenance

Some values need provenance:

```rust
pub enum Fact<T> {
    Observed(T),
    Effective {
        value: T,
        reason: EffectiveReason,
    },
    Unknown {
        reason: UnknownReason,
    },
}
```

Reasons should be typed or stable identifiers where dbmd needs programmatic behavior. Free-form explanation belongs in the render context.

Do not wrap every scalar reflexively. Introduce `Fact<T>` where absence, derivation, and unknown lead an agent to different conclusions. ClickHouse effective keys are the first motivating case.

## SQL expressions and types

Raw strings are acceptable for defaults, generated expressions, checks, predicates, view definitions, keys, partitions, and TTLs while dbmd only needs faithful rendering.

Structured parsing is justified when it enables:

- Correct cross-object links.
- Stable normalization unavailable from catalog fields.
- Lint rules that cannot operate on raw expressions.
- Backend-default computation.

Preserve the raw source expression even when optional parsed metadata is added.

## Backend coverage

### SQLite

Metadata sources include `sqlite_schema`, `PRAGMA table_xinfo`, `index_list`, `index_xinfo`, `foreign_key_list`, and `table_list` when supported.

The extension model must represent:

- Hidden and generated columns.
- Strict tables.
- `WITHOUT ROWID`.
- Index origin and partial indexes.
- Backend-version gaps in available PRAGMAs.

Views should preserve raw SQL definitions. Virtual tables, FTS objects, and triggers need fixtures before final inclusion decisions.

### PostgreSQL

Use `pg_catalog` where `information_schema` loses semantics. Candidate extensions include:

- Relation kinds, tablespaces, inheritance, and partitioning.
- Row-level security and policies.
- Identity and generated columns.
- Enum values.
- Index methods, predicates, expressions, and included columns.
- Function volatility and signatures.

The first PostgreSQL release should be scoped by fixtures rather than by exposing every catalog feature.

### ClickHouse

Candidate extensions include:

- Engine name and parameters.
- Sorting, explicit/effective primary, partition, and sampling keys.
- TTL expressions.
- Column codecs.
- Data-skipping indexes.
- Table settings.

Keep engine names and parameters structurally modest until real `engine_full` examples justify a typed engine enum.

## Deterministic normalization

Drivers must not expose catalog row order as product behavior. Before rendering, normalize:

- Selected sources according to resolved selection order.
- Namespaces and schema objects using stable backend-appropriate keys.
- Columns using ordinal position.
- Composite constraint and index columns using stored ordinal position.
- Unordered settings/maps using ordered map types or explicit sorting.

Ordering policy belongs in normalization or render-context construction, with one testable owner per collection.

## Model evolution checkpoints

Before the first useful SQLite release:

- Introduce project and source snapshot identity.
- Rename the misleading table engine/backend terminology.
- Canonicalize backend tags.
- Decide namespace representation using attached SQLite database fixtures.
- Separate internal model serialization from the template context.

Before ClickHouse:

- Introduce provenance for effective keys.
- Test explicit, defaulted, empty, and unavailable key scenarios.

Before promising a public machine-readable snapshot format:

- Define versioning independently from Markdown and template context.
- Specify compatibility and unknown-field behavior.
