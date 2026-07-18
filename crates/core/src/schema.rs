use std::collections::BTreeMap;

use serde::Serialize;

/// A backend namespace that can qualify schema objects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Namespace {
    /// Backend-defined namespace name.
    pub name: String,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// A first-class enumerated database type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnumType {
    /// Namespace containing the enum type.
    pub namespace: String,
    /// Enum type name.
    pub name: String,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Labels in backend-defined sort order.
    pub values: Vec<String>,
}

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
    /// PostgreSQL catalog and enforcement semantics.
    Postgres(PostgresConstraint),
    /// Semantics preserved from a SQLite table definition.
    Sqlite(SqliteConstraint),
}

/// PostgreSQL-specific constraint semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresConstraint {
    /// Complete server-normalized constraint definition.
    pub definition: String,
    /// Whether enforcement may be deferred within a transaction.
    pub deferrable: bool,
    /// Whether a deferrable constraint starts each transaction deferred.
    pub initially_deferred: bool,
    /// Whether existing rows have been validated against the constraint.
    pub validated: bool,
    /// Whether the constraint is defined locally rather than inherited.
    pub locally_defined: bool,
    /// Whether a check constraint is excluded from inheritance.
    pub no_inherit: bool,
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
    pub collation: Option<String>,
    /// Backend-qualified operator class, when the backend exposes one.
    pub operator_class: Option<String>,
    /// Effective ascending or descending order.
    pub order: IndexSortOrder,
    /// Effective null placement, when the backend exposes one.
    pub nulls_order: Option<IndexNullsOrder>,
}

/// Effective placement of null values in an ordered index term.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexNullsOrder {
    /// Null values sort before non-null values.
    First,
    /// Null values sort after non-null values.
    Last,
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
    /// PostgreSQL index access method, such as `btree` or `gin`.
    pub method: String,
    /// Partial-index predicate, when present.
    pub predicate: Option<String>,
    /// Non-key columns stored with the index.
    pub included_columns: Vec<String>,
    /// Whether a unique index treats nulls as equal.
    pub nulls_not_distinct: bool,
    /// Whether the index is valid for query planning.
    pub valid: bool,
    /// Whether the index is ready to receive writes.
    pub ready: bool,
    /// Whether the table is physically clustered on this index.
    pub clustered: bool,
    /// Whether this index is the table's replica identity.
    pub replica_identity: bool,
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
    /// Catalog relation classification.
    pub table_kind: PostgresTableKind,
    /// Explicit tablespace, when not using the database default.
    pub tablespace: Option<String>,
    /// Qualified parent relations in catalog order.
    pub inherits: Vec<String>,
    /// Server-normalized partition key for a partitioned table.
    pub partition_key: Option<String>,
    /// Qualified parent of a partition.
    pub partition_parent: Option<String>,
    /// Server-normalized partition bound.
    pub partition_bound: Option<String>,
    /// Whether row-level security is enabled.
    pub row_level_security: bool,
    /// Whether row-level security is forced for table owners.
    pub force_row_level_security: bool,
    /// Policies ordered by stable policy name.
    pub policies: Vec<PostgresPolicy>,
}

/// PostgreSQL table relation classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PostgresTableKind {
    /// Ordinary table.
    Table,
    /// Partitioned parent relation.
    PartitionedTable,
    /// Child partition relation.
    Partition,
    /// Foreign table backed by a foreign-data wrapper.
    ForeignTable,
}

/// One PostgreSQL row-level security policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresPolicy {
    /// Stable policy name.
    pub name: String,
    /// Whether multiple policies combine permissively with `OR`.
    pub permissive: bool,
    /// SQL command governed by the policy.
    pub command: PostgresPolicyCommand,
    /// Roles to which the policy applies.
    pub roles: Vec<String>,
    /// Row-visibility predicate.
    pub using_expression: Option<String>,
    /// New-row validation predicate.
    pub check_expression: Option<String>,
}

/// SQL command governed by a PostgreSQL row-level security policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PostgresPolicyCommand {
    /// All supported commands.
    All,
    /// `SELECT` operations.
    Select,
    /// `INSERT` operations.
    Insert,
    /// `UPDATE` operations.
    Update,
    /// `DELETE` operations.
    Delete,
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
    /// Backend-specific execution semantics.
    pub backend: FunctionBackend,
}

/// Backend-specific function semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "backend", rename_all = "lowercase")]
pub enum FunctionBackend {
    /// No backend-specific semantics are currently represented.
    Common,
    /// PostgreSQL function catalog semantics.
    Postgres(PostgresFunction),
}

impl FunctionBackend {
    /// Returns PostgreSQL volatility when this is a PostgreSQL function.
    #[must_use]
    pub fn volatility(&self) -> Option<PostgresFunctionVolatility> {
        match self {
            Self::Postgres(function) => Some(function.volatility),
            Self::Common => None,
        }
    }

    /// Returns PostgreSQL parallel safety when this is a PostgreSQL function.
    #[must_use]
    pub fn parallel(&self) -> Option<PostgresFunctionParallel> {
        match self {
            Self::Postgres(function) => Some(function.parallel),
            Self::Common => None,
        }
    }
}

/// PostgreSQL-specific function semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresFunction {
    /// Formatted return type.
    pub return_type: String,
    /// Procedural language name.
    pub language: String,
    /// Planner-visible volatility contract.
    pub volatility: PostgresFunctionVolatility,
    /// Parallel-execution safety.
    pub parallel: PostgresFunctionParallel,
    /// Whether the function executes with its owner's privileges.
    pub security_definer: bool,
}

/// PostgreSQL function volatility category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PostgresFunctionVolatility {
    /// Result depends only on arguments for all database states.
    Immutable,
    /// Result is stable within one statement.
    Stable,
    /// Result may change within one statement or cause side effects.
    Volatile,
}

/// PostgreSQL function parallel-execution safety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PostgresFunctionParallel {
    /// Safe in parallel workers.
    Safe,
    /// Must execute in the parallel group leader.
    Restricted,
    /// Cannot participate in a parallel query.
    Unsafe,
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
