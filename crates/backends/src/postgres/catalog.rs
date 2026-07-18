//! PostgreSQL-owned normalized catalog.

use dbmd_core::SourceSnapshot;
use serde::Serialize;

use crate::relational::{ConstraintKind, ForeignKeyReference, IndexTerm, Namespace};

/// A normalized PostgreSQL source snapshot.
pub type Snapshot = SourceSnapshot<Catalog>;

/// PostgreSQL schema content in deterministic catalog order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Catalog {
    pub namespaces: Vec<Namespace>,
    pub enums: Vec<EnumType>,
    pub tables: Vec<Table>,
    pub views: Vec<View>,
    pub triggers: Vec<Trigger>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnumType {
    pub namespace: String,
    pub name: String,
    pub comment: Option<String>,
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
    pub kind: TableKind,
    pub tablespace: Option<String>,
    pub inherits: Vec<String>,
    pub partition_key: Option<String>,
    pub partition_parent: Option<String>,
    pub partition_bound: Option<String>,
    pub row_level_security: bool,
    pub force_row_level_security: bool,
    pub policies: Vec<Policy>,
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
    pub enum_values: Vec<String>,
    pub identity: Option<String>,
    pub generated: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Constraint {
    pub name: Option<String>,
    pub kind: ConstraintKind,
    pub columns: Vec<String>,
    pub expression: Option<String>,
    pub references: Option<ForeignKeyReference>,
    pub definition: String,
    pub deferrable: bool,
    pub initially_deferred: bool,
    pub validated: bool,
    pub locally_defined: bool,
    pub no_inherit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Index {
    pub name: String,
    pub unique: bool,
    pub terms: Vec<IndexTerm>,
    pub predicate: Option<String>,
    pub definition: String,
    pub method: String,
    pub included_columns: Vec<String>,
    pub nulls_not_distinct: bool,
    pub valid: bool,
    pub ready: bool,
    pub clustered: bool,
    pub replica_identity: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TableKind {
    Table,
    PartitionedTable,
    Partition,
    ForeignTable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Policy {
    pub name: String,
    pub permissive: bool,
    pub command: PolicyCommand,
    pub roles: Vec<String>,
    pub using_expression: Option<String>,
    pub check_expression: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyCommand {
    All,
    Select,
    Insert,
    Update,
    Delete,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Trigger {
    pub namespace: String,
    pub name: String,
    pub target_namespace: String,
    pub target: String,
    pub timing: TriggerTiming,
    pub events: Vec<TriggerEvent>,
    pub orientation: TriggerOrientation,
    pub comment: Option<String>,
    pub when_expression: Option<String>,
    pub definition: String,
    pub function: String,
    pub arguments: Vec<String>,
    pub enabled: TriggerEnabled,
    pub constraint: Option<ConstraintTrigger>,
    pub old_transition_table: Option<String>,
    pub new_transition_table: Option<String>,
    pub parent_trigger: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerTiming {
    Before,
    After,
    InsteadOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerOrientation {
    Row,
    Statement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum TriggerEvent {
    Delete,
    Insert,
    Update { columns: Vec<String> },
    Truncate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConstraintTrigger {
    pub referenced_table: Option<String>,
    pub deferrable: bool,
    pub initially_deferred: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerEnabled {
    Origin,
    Disabled,
    Replica,
    Always,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Function {
    pub namespace: String,
    pub name: String,
    pub signature: String,
    pub definition: Option<String>,
    pub comment: Option<String>,
    pub return_type: String,
    pub language: String,
    pub volatility: FunctionVolatility,
    pub parallel: FunctionParallel,
    pub security_definer: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionVolatility {
    Immutable,
    Stable,
    Volatile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionParallel {
    Safe,
    Restricted,
    Unsafe,
}
