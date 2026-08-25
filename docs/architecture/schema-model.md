# Schema Model

## Identity envelope

A source snapshot has one backend-neutral envelope and one backend-owned
catalog:

```rust
pub struct SourceSnapshot<C> {
    id: SourceId,
    display_name: Option<String>,
    catalog: C,
}

pub struct DatabaseContext<C> {
    sources: Vec<SourceSnapshot<C>>,
}
```

`SourceId` is stable selection and path identity. `display_name` is
presentation-only. `DatabaseContext` preserves resolved source order and
rejects empty selections and duplicate IDs. Neither type contains credentials,
templates, output paths, or backend registration.

## Backend-owned catalogs

There is no universal `TableBackend`, `ColumnBackend`, or equivalent extension
enum in core. Each database family owns an independent aggregate type:

```rust
sqlite::Catalog { tables: Vec<sqlite::Table>, /* ... */ }
postgres::Catalog { tables: Vec<postgres::Table>, /* ... */ }
clickhouse::Catalog { tables: Vec<clickhouse::Table>, /* ... */ }
mysql::Catalog { tables: Vec<mysql::Table>, /* ... */ }
mariadb::Catalog { tables: Vec<mariadb::Table>, /* ... */ }
duckdb::Catalog { tables: Vec<duckdb::Table>, /* ... */ }
```

This permits SQLite `Trigger` to contain its single grammar event while
PostgreSQL `Trigger` represents multiple events, row/statement orientation,
transition tables, function arguments, constraint metadata, parentage, and
enablement. Neither shape is forced to pretend that the other database has the
same semantics.

Shared values in `dbmd-relational` are deliberately small and require
equivalent meaning across represented backends. Current examples include
namespaces, foreign-key actions and references, and ascending/descending index
ordering. Constraint categories and complete index-term shapes remain
backend-owned because PostgreSQL exclusion/operator-class/null-placement facts
and SQLite row identifiers are not shared semantics. Shared leaf types do not
imply a shared aggregate model.

## Composition catalog

Application operations need a heterogeneous ordered source collection. The
`dbmd-backends` root provides a closed compile-time catalog enum:

```rust
pub enum Catalog {
    Clickhouse(clickhouse::Catalog),
    Duckdb(duckdb::Catalog),
    Mariadb(mariadb::Catalog),
    Mysql(mysql::Catalog),
    Sqlite(sqlite::Catalog),
    Postgres(postgres::Catalog),
}

pub type DatabaseContext = dbmd_core::DatabaseContext<Catalog>;
```

This is composition wiring, not the domain owner of concrete catalogs. Adding
a backend changes this root by design but does not change core or render.

## Facts and fidelity

Catalog adapters distinguish absence from unknown wherever it affects agent
reasoning. Typed provenance may be introduced for facts whose observed,
effective, and unknown states lead to different conclusions; it should not wrap
every scalar reflexively.

Raw strings are appropriate for defaults, generated expressions, checks,
predicates, view/function/trigger definitions, partition expressions, and other
SQL until structured parsing enables correctness, linking, or lint behavior.
Preserve raw definitions when adding parsed fields.

### Native-code normalization

Engine catalog codes do not belong in backend catalog interfaces. Each backend
adapter translates native discriminator values at the introspection seam:

- Closed native sets become backend-owned enums or validated value types. For
  example, PostgreSQL ACL letters, relation-kind codes, and trigger enablement
  codes must not escape as strings.
- Unknown closed-set values fail introspection with the source and catalog field
  identified. They are not silently flattened into an `Other(String)` variant.
- Names, qualified identities, SQL expressions, extension-defined option names,
  and other genuinely open values remain strings.
- Server-normalized object identities may be retained as strings when they
  preserve quoting, overload signatures, or backend grammar; their object family
  is still represented semantically.
- Backend semantic types may expose a canonical human-facing name when that name
  is part of the backend vocabulary. Render mapping owns placement and Markdown
  formatting. Templates must not decode native database identifiers or invent
  catalog meaning.

This translation is backend-local because equal-looking native codes do not
establish equivalent semantics across database families. A leaf enum moves to
`dbmd-relational` only when at least two backends demonstrably share the same
meaning, not merely the same spelling.

## Deterministic normalization

Each backend owns stable ordering for its catalog:

- Namespaces and objects use backend-appropriate stable keys.
- Columns use catalog ordinal position.
- Composite constraints and index terms use stored ordinal position.
- Unordered maps are converted to ordered maps or sorted vectors.
- Selected sources preserve the application-resolved order.

Drivers never expose unspecified catalog row order as product behavior.

## Coverage contracts

The durable implementation coverage matrices live beside the owning backend:

- [SQLite](../../crates/backends/sqlite/README.md)
- [PostgreSQL](../../crates/backends/postgres/README.md)
- [ClickHouse](../../crates/backends/clickhouse/README.md)
- [MySQL](../../crates/backends/mysql/README.md)
- [MariaDB](../../crates/backends/mariadb/README.md)
- [DuckDB](../../crates/backends/duckdb/README.md)

Product documentation states user-visible contracts; these module documents
state which backend schema facts are implemented and fixture-proven.
