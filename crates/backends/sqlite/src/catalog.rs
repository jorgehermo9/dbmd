//! SQLite-owned normalized catalog.

use dbmd_core::SourceSnapshot;
use serde::Serialize;

use dbmd_relational::{ForeignKeyReference, IndexSortOrder, Namespace};

/// A normalized SQLite source snapshot.
pub type Snapshot = SourceSnapshot<Catalog>;

/// SQLite schema content in deterministic catalog order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Catalog {
    /// Persisted database namespaces in resolved attachment order.
    pub namespaces: Vec<Namespace>,
    /// Tables in deterministic namespace/name order.
    pub tables: Vec<Table>,
    /// Views in deterministic namespace/name order.
    pub views: Vec<View>,
    /// Triggers in deterministic namespace/name order.
    pub triggers: Vec<Trigger>,
}

/// One persisted SQLite table or represented virtual-table shadow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Table {
    /// Owning database namespace.
    pub namespace: String,
    /// Unqualified table name.
    pub name: String,
    /// Optional catalog comment when a future SQLite source provides one.
    pub comment: Option<String>,
    /// Columns in declared ordinal order.
    pub columns: Vec<Column>,
    /// Declared and effective constraints.
    pub constraints: Vec<Constraint>,
    /// Explicit and constraint-backed indexes.
    pub indexes: Vec<Index>,
    /// Whether the table uses `WITHOUT ROWID` storage.
    pub without_rowid: bool,
    /// Whether the table uses SQLite strict typing.
    pub strict: bool,
    /// Exact persisted table definition when retained by SQLite.
    pub definition: Option<String>,
    /// Ordinary, virtual, or shadow-table semantics.
    pub kind: TableKind,
}

impl Table {
    /// Returns the namespace-qualified table name.
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.namespace, self.name)
    }
}

/// One SQLite table or view column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Column {
    /// Declared column name.
    pub name: String,
    /// Declared SQLite type spelling.
    pub data_type: String,
    /// Effective nullability, or unknown where SQLite cannot prove it.
    pub nullable: Option<bool>,
    /// Persisted default expression.
    pub default: Option<String>,
    /// Optional catalog comment when available.
    pub comment: Option<String>,
    /// Normal, generated, or virtual-table-hidden status.
    pub kind: ColumnKind,
    /// Effective collation name.
    pub collation: String,
    /// Generated-column expression when applicable.
    pub generated_expression: Option<String>,
}

/// SQLite column visibility and generation kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ColumnKind {
    /// An ordinary visible column.
    Normal,
    /// A hidden virtual-table implementation column.
    VirtualTableHidden,
    /// A virtual generated column.
    VirtualGenerated,
    /// A stored generated column.
    StoredGenerated,
}

/// SQLite constraint category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ConstraintKind {
    /// Primary key.
    PrimaryKey,
    /// Foreign key.
    ForeignKey,
    /// Unique key.
    Unique,
    /// Check expression.
    Check,
    /// Not-null requirement.
    NotNull,
}

/// One normalized SQLite constraint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Constraint {
    /// Declared constraint name, if any.
    pub name: Option<String>,
    /// Relational constraint category.
    pub kind: ConstraintKind,
    /// Participating columns in declared order.
    pub columns: Vec<String>,
    /// Check or other preserved expression.
    pub expression: Option<String>,
    /// Foreign-key target and actions.
    pub references: Option<ForeignKeyReference>,
    /// Declared conflict policy.
    pub conflict_resolution: Option<ConflictResolution>,
    /// Whether the primary key carries `AUTOINCREMENT`.
    pub auto_increment: bool,
    /// Whether the constraint was declared inline on a column.
    pub declared_on_column: bool,
}

/// SQLite conflict-resolution policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ConflictResolution {
    /// Roll back the transaction.
    Rollback,
    /// Abort the statement.
    Abort,
    /// Fail while preserving prior row changes.
    Fail,
    /// Ignore the conflicting row.
    Ignore,
    /// Replace the conflicting row.
    Replace,
}

/// One normalized SQLite index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Index {
    /// Index name.
    pub name: String,
    /// Whether keys must be unique.
    pub unique: bool,
    /// Ordered key terms.
    pub terms: Vec<IndexTerm>,
    /// Partial-index predicate.
    pub predicate: Option<String>,
    /// Exact persisted definition for explicit indexes.
    pub definition: Option<String>,
    /// Whether the index was explicit or constraint-backed.
    pub origin: IndexOrigin,
}

/// One ordered SQLite index key term.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct IndexTerm {
    /// Column, expression, or row identifier selected by the term.
    pub target: IndexTarget,
    /// Effective collation.
    pub collation: Option<String>,
    /// Effective sort direction.
    pub order: IndexSortOrder,
}

/// Value selected by a SQLite index key term.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IndexTarget {
    /// A named column.
    Column(String),
    /// A stored index expression.
    Expression(String),
    /// SQLite row identifier.
    RowId,
}

/// How SQLite created an index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IndexOrigin {
    /// Created by `CREATE INDEX`.
    CreateIndex,
    /// Backing a unique constraint.
    UniqueConstraint,
    /// Backing a primary key.
    PrimaryKey,
}

/// SQLite table storage family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum TableKind {
    /// An ordinary table.
    Ordinary,
    /// A virtual table owned by a module.
    Virtual {
        /// Virtual-table module name.
        module: String,
        /// Raw module arguments in declared order.
        arguments: Vec<String>,
    },
    /// A module-owned virtual-table shadow table.
    Shadow {
        /// Owning virtual table when recognized.
        virtual_table: Option<String>,
    },
}

/// One persisted SQLite view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct View {
    /// Owning database namespace.
    pub namespace: String,
    /// Unqualified view name.
    pub name: String,
    /// Exact persisted view definition.
    pub definition: String,
    /// Derived or declared columns in ordinal order.
    pub columns: Vec<Column>,
}

/// One persisted SQLite trigger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Trigger {
    /// Owning database namespace.
    pub namespace: String,
    /// Trigger name, unique per target.
    pub name: String,
    /// Target database namespace.
    pub target_namespace: String,
    /// Target table or view name.
    pub target: String,
    /// Trigger timing.
    pub timing: TriggerTiming,
    /// Trigger event and optional update-column list.
    pub event: TriggerEvent,
    /// Optional `WHEN` predicate.
    pub when_expression: Option<String>,
    /// Exact persisted trigger definition.
    pub definition: String,
}

/// When a SQLite trigger fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TriggerTiming {
    /// Before the triggering operation.
    Before,
    /// After the triggering operation.
    After,
    /// Instead of an operation on a view.
    InsteadOf,
}

/// Operation that fires a SQLite trigger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
#[non_exhaustive]
pub enum TriggerEvent {
    /// A delete event.
    Delete,
    /// An insert event.
    Insert,
    /// An update event, optionally restricted to columns.
    Update { columns: Vec<String> },
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    macro_rules! serialization_cases {
        ($($name:ident: $value:expr => $expected:expr;)+) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!(
                        serde_json::to_value(&$value).expect("semantic enum should serialize"),
                        $expected
                    );
                }
            )+
        };
    }

    serialization_cases! {
        serializes_normal_column_kind: ColumnKind::Normal => json!("normal");
        serializes_hidden_virtual_table_column_kind: ColumnKind::VirtualTableHidden => json!("virtual_table_hidden");
        serializes_virtual_generated_column_kind: ColumnKind::VirtualGenerated => json!("virtual_generated");
        serializes_stored_generated_column_kind: ColumnKind::StoredGenerated => json!("stored_generated");
        serializes_primary_key_constraint: ConstraintKind::PrimaryKey => json!("primary_key");
        serializes_foreign_key_constraint: ConstraintKind::ForeignKey => json!("foreign_key");
        serializes_unique_constraint: ConstraintKind::Unique => json!("unique");
        serializes_check_constraint: ConstraintKind::Check => json!("check");
        serializes_not_null_constraint: ConstraintKind::NotNull => json!("not_null");
        serializes_rollback_conflict_resolution: ConflictResolution::Rollback => json!("rollback");
        serializes_abort_conflict_resolution: ConflictResolution::Abort => json!("abort");
        serializes_fail_conflict_resolution: ConflictResolution::Fail => json!("fail");
        serializes_ignore_conflict_resolution: ConflictResolution::Ignore => json!("ignore");
        serializes_replace_conflict_resolution: ConflictResolution::Replace => json!("replace");
        serializes_column_index_target: IndexTarget::Column("account_id".to_string()) => json!({"column": "account_id"});
        serializes_expression_index_target: IndexTarget::Expression("lower(email)".to_string()) => json!({"expression": "lower(email)"});
        serializes_rowid_index_target: IndexTarget::RowId => json!("row_id");
        serializes_create_index_origin: IndexOrigin::CreateIndex => json!("create_index");
        serializes_unique_constraint_index_origin: IndexOrigin::UniqueConstraint => json!("unique_constraint");
        serializes_primary_key_index_origin: IndexOrigin::PrimaryKey => json!("primary_key");
        serializes_ordinary_table_kind: TableKind::Ordinary => json!({"kind": "ordinary"});
        serializes_virtual_table_kind: TableKind::Virtual { module: "fts5".to_string(), arguments: vec!["body".to_string()] } => json!({"kind": "virtual", "module": "fts5", "arguments": ["body"]});
        serializes_shadow_table_kind: TableKind::Shadow { virtual_table: Some("documents".to_string()) } => json!({"kind": "shadow", "virtual_table": "documents"});
        serializes_before_trigger_timing: TriggerTiming::Before => json!("before");
        serializes_after_trigger_timing: TriggerTiming::After => json!("after");
        serializes_instead_of_trigger_timing: TriggerTiming::InsteadOf => json!("instead_of");
        serializes_delete_trigger_event: TriggerEvent::Delete => json!({"event": "delete"});
        serializes_insert_trigger_event: TriggerEvent::Insert => json!({"event": "insert"});
        serializes_update_trigger_event: TriggerEvent::Update { columns: vec!["email".to_string()] } => json!({"event": "update", "columns": ["email"]});
    }
}
