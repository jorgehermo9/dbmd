# Schema Model

## Responsibilities

The normalized model represents database structure after backend introspection and before presentation. It should:

- Preserve common relationships without erasing backend differences.
- Carry stable source and namespace identity.
- Distinguish observed, effective, and unknown facts when the distinction matters.
- Support deterministic traversal.
- Avoid dependencies on database drivers, CLI parsing, and templates.
- Serialize into a dedicated render context without itself becoming the template API.

## Aggregate shape

`SourceSnapshot` represents one identified introspection result. `DatabaseContext` is the explicit aggregate for the ordered sources selected by an application operation.

Directional sketch:

```rust
pub struct DatabaseContext {
    sources: Vec<SourceSnapshot>,
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

`DatabaseContext` is the domain aggregate consumed by application operations. Its constructor preserves resolved selection order and rejects an empty collection or duplicate source IDs. It contains no project configuration, credentials, template choices, or output paths.

Exact object nesting should be proven by SQLite and multi-source fixtures. The invariant is that source identity is explicit and survives to output paths and headings.

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

The backend extension field is named `backend` and uses `TableBackend`. Reserve `engine` for backend concepts that actually use that term, such as a ClickHouse table engine.

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

Do not wrap every scalar reflexively. Introduce `Fact<T>` where absence,
derivation, and unknown lead an agent to different conclusions. ClickHouse
effective keys are a motivating case.

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

The adapter-local [SQLite coverage matrix](../../crates/introspect/src/sqlite/README.md)
is the source of truth for fixture evidence. This architecture document defines
the model rather than duplicating adapter coverage.

The extension model must represent:

- Normal columns, virtual-table hidden columns, and virtual/stored generated columns as distinct `SqliteColumnKind` variants.
- Strict tables.
- `WITHOUT ROWID`.
- Index origin and partial indexes.
- Backend-version gaps in available PRAGMAs.

Generated-column kind comes from `PRAGMA table_xinfo`; virtual and stored
expressions are preserved by parsing SQLite's stored schema SQL. Fixtures cover
both forms.

Views, virtual tables, FTS5 objects, shadow tables, and triggers are first-class
model values with raw SQL definitions retained as a fidelity backstop.

### PostgreSQL

Use `pg_catalog` where `information_schema` loses semantics. PostgreSQL
extensions include:

- Relation kinds, tablespaces, inheritance, and partitioning.
- Row-level security and policies.
- Identity and generated columns.
- Enum values.
- Index methods, predicates, expressions, and included columns.
- Function volatility and signatures.

PostgreSQL adapter scope is defined by fixtures rather than an attempt to expose
every catalog feature.

The implementation-level [PostgreSQL coverage matrix](../../crates/introspect/src/postgres/README.md)
records the fixture-proven catalog and DDL surface.

### ClickHouse

ClickHouse extensions include:

- Engine name and parameters.
- Sorting, explicit/effective primary, partition, and sampling keys.
- TTL expressions.
- Column codecs.
- Data-skipping indexes.
- Table settings.

Engine names and parameters remain structurally modest; a typed engine enum
requires representative `engine_full` fixture evidence.

## Deterministic normalization

Drivers must not expose catalog row order as product behavior. Before rendering, normalize:

- Selected sources according to resolved selection order.
- Namespaces and schema objects using stable backend-appropriate keys.
- Columns using ordinal position.
- Composite constraint and index columns using stored ordinal position.
- Unordered settings/maps using ordered map types or explicit sorting.

Ordering policy belongs in normalization or render-context construction, with one testable owner per collection.
