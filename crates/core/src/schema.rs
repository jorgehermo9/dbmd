use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Table {
    pub namespace: String,
    pub name: String,
    pub comment: Option<String>,
    pub columns: Vec<Column>,
    pub constraints: Vec<Constraint>,
    pub indexes: Vec<Index>,
    pub backend: TableBackend,
}

impl Table {
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.namespace, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    /// Effective nullability, or `None` when the backend cannot determine it.
    pub nullable: Option<bool>,
    pub default: Option<String>,
    pub comment: Option<String>,
    pub backend: ColumnBackend,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "backend", rename_all = "lowercase")]
pub enum ColumnBackend {
    Common,
    Postgres(PostgresColumn),
    #[serde(rename = "clickhouse")]
    ClickHouse(ClickHouseColumn),
    Sqlite(SqliteColumn),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresColumn {
    pub enum_values: Vec<String>,
    pub identity: Option<String>,
    pub generated: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClickHouseColumn {
    pub codec: Option<String>,
    pub ttl: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SqliteColumn {
    pub kind: SqliteColumnKind,
    /// Effective collation used by the column.
    pub collation: String,
    /// Stored SQL expression for a generated column.
    pub generated_expression: Option<String>,
}

/// How SQLite stores or exposes a column according to `PRAGMA table_xinfo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SqliteColumnKind {
    /// An ordinary table column.
    Normal,
    /// A hidden input column exposed by a virtual table.
    VirtualTableHidden,
    /// A generated column computed when read.
    VirtualGenerated,
    /// A generated column computed and stored when written.
    StoredGenerated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Constraint {
    pub name: Option<String>,
    pub kind: ConstraintKind,
    pub columns: Vec<String>,
    pub expression: Option<String>,
    pub references: Option<ForeignKeyReference>,
    pub backend: ConstraintBackend,
}

/// Backend-specific constraint semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "backend", rename_all = "lowercase")]
pub enum ConstraintBackend {
    /// No backend-specific semantics are currently represented.
    Common,
    /// Semantics preserved from a SQLite table definition.
    Sqlite(SqliteConstraint),
}

/// SQLite-specific constraint semantics preserved from stored schema SQL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SqliteConstraint {
    /// Conflict algorithm explicitly declared for this constraint.
    pub conflict_resolution: Option<SqliteConflictResolution>,
    /// Whether an integer primary key declared `AUTOINCREMENT`.
    pub auto_increment: bool,
    /// Whether the constraint appeared in a column definition rather than the table constraint list.
    pub declared_on_column: bool,
}

/// SQLite's conflict resolution algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SqliteConflictResolution {
    Rollback,
    Abort,
    Fail,
    Ignore,
    Replace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintKind {
    PrimaryKey,
    ForeignKey,
    Unique,
    Check,
    NotNull,
    Exclusion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ForeignKeyReference {
    pub namespace: String,
    pub table: String,
    pub columns: Vec<String>,
    pub on_update: ForeignKeyAction,
    pub on_delete: ForeignKeyAction,
    /// `MATCH` name preserved from the stored table definition.
    pub match_name: Option<String>,
    /// Declared deferrability and initial timing.
    pub deferrability: ForeignKeyDeferrability,
}

/// Whether and when a foreign-key constraint may be deferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ForeignKeyDeferrability {
    pub deferrable: bool,
    pub initially: ForeignKeyInitialTiming,
}

impl Default for ForeignKeyDeferrability {
    fn default() -> Self {
        Self {
            deferrable: false,
            initially: ForeignKeyInitialTiming::Immediate,
        }
    }
}

/// Initial enforcement timing of a foreign-key constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForeignKeyInitialTiming {
    Immediate,
    Deferred,
}

/// The behavior applied to child rows when a referenced key changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForeignKeyAction {
    /// Perform no special action before normal constraint enforcement.
    NoAction,
    /// Reject the referenced change immediately while child rows exist.
    Restrict,
    /// Set child-key columns to `NULL`.
    SetNull,
    /// Set child-key columns to their declared defaults.
    SetDefault,
    /// Propagate the referenced change to child rows.
    Cascade,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Index {
    pub name: String,
    pub unique: bool,
    pub terms: Vec<IndexTerm>,
    pub predicate: Option<String>,
    pub definition: Option<String>,
    pub backend: IndexBackend,
}

/// One ordered key term of an index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndexTerm {
    /// The value indexed by this term.
    pub target: IndexTarget,
    /// Effective collation used to compare the term.
    pub collation: String,
    /// Effective ascending or descending order.
    pub order: IndexSortOrder,
}

/// The value selected by an index key term.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexTarget {
    /// A named table column.
    Column(String),
    /// A SQL expression.
    Expression(String),
    /// SQLite's implicit row identifier.
    RowId,
}

/// Effective ordering of an index key term.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexSortOrder {
    /// Ascending order.
    Ascending,
    /// Descending order.
    Descending,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "backend", rename_all = "lowercase")]
pub enum IndexBackend {
    Common,
    Postgres(PostgresIndex),
    #[serde(rename = "clickhouse")]
    ClickHouse(ClickHouseIndex),
    Sqlite(SqliteIndex),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresIndex {
    pub method: String,
    pub predicate: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClickHouseIndex {
    pub granularity: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SqliteIndex {
    /// How SQLite created the index.
    pub origin: SqliteIndexOrigin,
}

/// The catalog origin reported by `PRAGMA index_list`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SqliteIndexOrigin {
    /// An explicit `CREATE INDEX` statement.
    CreateIndex,
    /// An implicit index backing a `UNIQUE` constraint.
    UniqueConstraint,
    /// An implicit index backing a primary key.
    PrimaryKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "backend", rename_all = "lowercase")]
pub enum TableBackend {
    Postgres(PostgresTable),
    #[serde(rename = "clickhouse")]
    ClickHouse(ClickHouseTable),
    Sqlite(SqliteTable),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresTable {
    pub table_kind: PostgresTableKind,
    pub tablespace: Option<String>,
    pub inherits: Vec<String>,
    pub partition: Option<String>,
    pub row_level_security: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PostgresTableKind {
    Table,
    PartitionedTable,
    ForeignTable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClickHouseTable {
    pub engine: String,
    pub engine_params: Vec<String>,
    pub order_by: Vec<String>,
    pub partition_by: Option<String>,
    pub primary_key: Vec<String>,
    pub sample_by: Option<String>,
    pub ttl: Option<String>,
    pub settings: BTreeMap<String, String>,
}

impl ClickHouseTable {
    #[must_use]
    pub fn engine_clause(&self) -> String {
        if self.engine_params.is_empty() {
            self.engine.clone()
        } else {
            format!("{}({})", self.engine, self.engine_params.join(", "))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SqliteTable {
    pub without_rowid: bool,
    pub strict: bool,
    /// Exact `CREATE TABLE` SQL retained by SQLite when available.
    pub definition: Option<String>,
    /// SQLite's catalog classification for the table.
    pub kind: SqliteTableKind,
}

/// SQLite table classification reported by `PRAGMA table_list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SqliteTableKind {
    Ordinary,
    Virtual {
        module: String,
        arguments: Vec<String>,
    },
    Shadow {
        virtual_table: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct View {
    pub namespace: String,
    pub name: String,
    pub definition: String,
    pub materialized: bool,
    pub comment: Option<String>,
    pub columns: Vec<Column>,
}

/// A database trigger and its stored definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Trigger {
    pub namespace: String,
    pub name: String,
    pub target_namespace: String,
    pub target: String,
    pub timing: TriggerTiming,
    pub event: TriggerEvent,
    pub when_expression: Option<String>,
    pub definition: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerTiming {
    Before,
    After,
    InsteadOf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum TriggerEvent {
    Delete,
    Insert,
    Update { columns: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Function {
    pub namespace: String,
    pub name: String,
    pub signature: String,
    pub definition: Option<String>,
    pub comment: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_clickhouse_engine_clause() {
        let table = ClickHouseTable {
            engine: "ReplacingMergeTree".to_string(),
            engine_params: vec!["version".to_string(), "is_deleted".to_string()],
            order_by: vec!["user_id".to_string(), "occurred_at".to_string()],
            partition_by: Some("toYYYYMM(occurred_at)".to_string()),
            primary_key: vec!["user_id".to_string()],
            sample_by: None,
            ttl: None,
            settings: BTreeMap::new(),
        };

        assert_eq!(
            table.engine_clause(),
            "ReplacingMergeTree(version, is_deleted)"
        );
    }
}
