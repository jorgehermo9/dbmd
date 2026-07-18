//! SQLite-owned normalized catalog.

use dbmd_core::SourceSnapshot;
use serde::Serialize;

use crate::relational::{ConstraintKind, ForeignKeyReference, IndexTerm, Namespace};

/// A normalized SQLite source snapshot.
pub type Snapshot = SourceSnapshot<Catalog>;

/// SQLite schema content in deterministic catalog order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Catalog {
    pub namespaces: Vec<Namespace>,
    pub tables: Vec<Table>,
    pub views: Vec<View>,
    pub triggers: Vec<Trigger>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Table {
    pub namespace: String,
    pub name: String,
    pub comment: Option<String>,
    pub columns: Vec<Column>,
    pub constraints: Vec<Constraint>,
    pub indexes: Vec<Index>,
    pub without_rowid: bool,
    pub strict: bool,
    pub definition: Option<String>,
    pub kind: TableKind,
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
    pub nullable: Option<bool>,
    pub default: Option<String>,
    pub comment: Option<String>,
    pub kind: ColumnKind,
    pub collation: String,
    pub generated_expression: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnKind {
    Normal,
    VirtualTableHidden,
    VirtualGenerated,
    StoredGenerated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Constraint {
    pub name: Option<String>,
    pub kind: ConstraintKind,
    pub columns: Vec<String>,
    pub expression: Option<String>,
    pub references: Option<ForeignKeyReference>,
    pub conflict_resolution: Option<ConflictResolution>,
    pub auto_increment: bool,
    pub declared_on_column: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    Rollback,
    Abort,
    Fail,
    Ignore,
    Replace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Index {
    pub name: String,
    pub unique: bool,
    pub terms: Vec<IndexTerm>,
    pub predicate: Option<String>,
    pub definition: Option<String>,
    pub origin: IndexOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexOrigin {
    CreateIndex,
    UniqueConstraint,
    PrimaryKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TableKind {
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
    pub columns: Vec<Column>,
}

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
