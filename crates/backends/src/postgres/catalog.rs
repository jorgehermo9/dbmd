//! PostgreSQL-owned normalized catalog.

use dbmd_core::SourceSnapshot;
use serde::Serialize;

use crate::relational::{ConstraintKind, ForeignKeyReference, IndexTerm, Namespace};

/// A normalized PostgreSQL source snapshot.
pub type Snapshot = SourceSnapshot<Catalog>;

/// PostgreSQL schema content in deterministic catalog order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Catalog {
    /// User namespaces in deterministic name order.
    pub namespaces: Vec<Namespace>,
    /// Enum types in schema/name order.
    pub enums: Vec<EnumType>,
    /// Relations represented as tables in schema/name order.
    pub tables: Vec<Table>,
    /// Ordinary and materialized views in schema/name order.
    pub views: Vec<View>,
    /// Triggers in target schema/relation/name order.
    pub triggers: Vec<Trigger>,
    /// Functions in schema/name/signature order.
    pub functions: Vec<Function>,
}

/// One PostgreSQL enum type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct EnumType {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified type name.
    pub name: String,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Labels in enum sort order.
    pub values: Vec<String>,
}

/// One PostgreSQL table-like relation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Table {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified relation name.
    pub name: String,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Columns in ordinal order.
    pub columns: Vec<Column>,
    /// Constraints in name order.
    pub constraints: Vec<Constraint>,
    /// Indexes in name order.
    pub indexes: Vec<Index>,
    /// Relation family.
    pub kind: TableKind,
    /// Explicit tablespace name.
    pub tablespace: Option<String>,
    /// Direct inheritance parents as qualified names.
    pub inherits: Vec<String>,
    /// Server-formatted partition key.
    pub partition_key: Option<String>,
    /// Qualified partition parent.
    pub partition_parent: Option<String>,
    /// Server-formatted partition bound.
    pub partition_bound: Option<String>,
    /// Whether row-level security is enabled.
    pub row_level_security: bool,
    /// Whether RLS is forced for the table owner.
    pub force_row_level_security: bool,
    /// Row-level security policies in name order.
    pub policies: Vec<Policy>,
}

impl Table {
    /// Returns the schema-qualified relation name.
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.namespace, self.name)
    }
}

/// One PostgreSQL relation or view column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Column {
    /// Column name.
    pub name: String,
    /// Server-formatted data type.
    pub data_type: String,
    /// Effective nullability when known.
    pub nullable: Option<bool>,
    /// Server-formatted default expression.
    pub default: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Enum labels when the column type is an enum.
    pub enum_values: Vec<String>,
    /// Identity generation mode.
    pub identity: Option<String>,
    /// Generated-column expression.
    pub generated: Option<String>,
}

/// One PostgreSQL table constraint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Constraint {
    /// Constraint name.
    pub name: Option<String>,
    /// Relational constraint category.
    pub kind: ConstraintKind,
    /// Participating columns in declared order.
    pub columns: Vec<String>,
    /// Preserved expression when applicable.
    pub expression: Option<String>,
    /// Foreign-key target and actions.
    pub references: Option<ForeignKeyReference>,
    /// Server-normalized definition.
    pub definition: String,
    /// Whether enforcement may be deferred.
    pub deferrable: bool,
    /// Whether enforcement starts deferred.
    pub initially_deferred: bool,
    /// Whether the constraint has been validated.
    pub validated: bool,
    /// Whether the constraint is locally defined.
    pub locally_defined: bool,
    /// Whether the constraint excludes inheritance.
    pub no_inherit: bool,
}

/// One PostgreSQL index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Index {
    /// Index name.
    pub name: String,
    /// Whether index keys must be unique.
    pub unique: bool,
    /// Ordered key terms.
    pub terms: Vec<IndexTerm>,
    /// Partial-index predicate.
    pub predicate: Option<String>,
    /// Complete server-normalized definition.
    pub definition: String,
    /// Index access method.
    pub method: String,
    /// Non-key included columns.
    pub included_columns: Vec<String>,
    /// Whether nulls compare as not distinct.
    pub nulls_not_distinct: bool,
    /// Whether the index is valid for queries.
    pub valid: bool,
    /// Whether the index is ready for writes.
    pub ready: bool,
    /// Whether the table is physically clustered on this index.
    pub clustered: bool,
    /// Whether the index supplies replica identity.
    pub replica_identity: bool,
}

/// PostgreSQL table-like relation family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TableKind {
    /// An ordinary table.
    Table,
    /// A partitioned table.
    PartitionedTable,
    /// A child partition.
    Partition,
    /// A foreign table.
    ForeignTable,
}

/// One PostgreSQL row-level security policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Policy {
    /// Policy name.
    pub name: String,
    /// Whether the policy combines permissively.
    pub permissive: bool,
    /// Command governed by the policy.
    pub command: PolicyCommand,
    /// Role names in catalog order.
    pub roles: Vec<String>,
    /// Optional `USING` expression.
    pub using_expression: Option<String>,
    /// Optional `WITH CHECK` expression.
    pub check_expression: Option<String>,
}

/// Command governed by a row-level security policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PolicyCommand {
    /// All commands.
    All,
    /// Select operations.
    Select,
    /// Insert operations.
    Insert,
    /// Update operations.
    Update,
    /// Delete operations.
    Delete,
}

/// One ordinary or materialized PostgreSQL view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct View {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified view name.
    pub name: String,
    /// Server-normalized query definition.
    pub definition: String,
    /// Whether this is a materialized view.
    pub materialized: bool,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// View columns in ordinal order.
    pub columns: Vec<Column>,
}

/// One PostgreSQL trigger, including partition clones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Trigger {
    /// Target schema, which owns the trigger identity.
    pub namespace: String,
    /// Trigger name, unique per target relation.
    pub name: String,
    /// Target schema.
    pub target_namespace: String,
    /// Target relation or view.
    pub target: String,
    /// Trigger timing.
    pub timing: TriggerTiming,
    /// Events in canonical insert/update/delete/truncate order.
    pub events: Vec<TriggerEvent>,
    /// Row or statement orientation.
    pub orientation: TriggerOrientation,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Optional server-normalized `WHEN` predicate.
    pub when_expression: Option<String>,
    /// Complete server-normalized definition.
    pub definition: String,
    /// Qualified called-function identity.
    pub function: String,
    /// Literal trigger arguments in declared order.
    pub arguments: Vec<String>,
    /// Replication-role enablement state.
    pub enabled: TriggerEnabled,
    /// Constraint-trigger metadata.
    pub constraint: Option<ConstraintTrigger>,
    /// Old transition-table alias.
    pub old_transition_table: Option<String>,
    /// New transition-table alias.
    pub new_transition_table: Option<String>,
    /// Qualified parent-trigger identity for a partition clone.
    pub parent_trigger: Option<String>,
}

/// When a PostgreSQL trigger fires.
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

/// Whether a PostgreSQL trigger fires per row or statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TriggerOrientation {
    /// Once per affected row.
    Row,
    /// Once per statement.
    Statement,
}

/// Operation that fires a PostgreSQL trigger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
#[non_exhaustive]
pub enum TriggerEvent {
    /// Delete event.
    Delete,
    /// Insert event.
    Insert,
    /// Update event, optionally restricted to columns.
    Update { columns: Vec<String> },
    /// Truncate event.
    Truncate,
}

/// PostgreSQL constraint-trigger properties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ConstraintTrigger {
    /// Qualified referenced relation when present.
    pub referenced_table: Option<String>,
    /// Whether firing may be deferred.
    pub deferrable: bool,
    /// Whether firing starts deferred.
    pub initially_deferred: bool,
}

/// PostgreSQL trigger replication-role state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TriggerEnabled {
    /// Enabled for the origin/local role.
    Origin,
    /// Disabled.
    Disabled,
    /// Enabled for replica role.
    Replica,
    /// Enabled for every replication role.
    Always,
}

/// One PostgreSQL function overload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Function {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified function name.
    pub name: String,
    /// Identity-argument signature including parentheses.
    pub signature: String,
    /// Server-normalized definition when available.
    pub definition: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Server-formatted return type.
    pub return_type: String,
    /// Implementation language.
    pub language: String,
    /// Volatility contract.
    pub volatility: FunctionVolatility,
    /// Parallel-safety contract.
    pub parallel: FunctionParallel,
    /// Whether execution uses definer rather than invoker privileges.
    pub security_definer: bool,
}

/// PostgreSQL function volatility contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FunctionVolatility {
    /// Immutable for identical arguments.
    Immutable,
    /// Stable within a statement.
    Stable,
    /// May change on every call.
    Volatile,
}

/// PostgreSQL function parallel-safety contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FunctionParallel {
    /// Safe in parallel workers.
    Safe,
    /// Restricted to the parallel leader.
    Restricted,
    /// Unsafe in a parallel plan.
    Unsafe,
}
