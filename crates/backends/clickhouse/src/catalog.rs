//! ClickHouse-owned normalized catalog.

use dbmd_core::SourceSnapshot;
use serde::Serialize;

/// A normalized ClickHouse source snapshot.
pub type Snapshot = SourceSnapshot<Catalog>;

/// ClickHouse schema content in deterministic catalog order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Catalog {
    /// Selected databases in name order.
    pub databases: Vec<Database>,
    /// Tables and view families in database/name order.
    pub tables: Vec<Table>,
    /// SQL user-defined functions in name order.
    pub functions: Vec<UserDefinedFunction>,
}

/// One selected ClickHouse database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Database {
    /// Database name.
    pub name: String,
    /// Database engine.
    pub engine: String,
    /// Optional database comment.
    pub comment: Option<String>,
}

/// A ClickHouse table, view, or materialized view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Table {
    /// Owning database.
    pub database: String,
    /// Unqualified object name.
    pub name: String,
    /// Object family derived from its engine.
    pub kind: TableKind,
    /// Storage or view engine name.
    pub engine: String,
    /// Complete engine expression including parameters and table clauses.
    pub engine_full: String,
    /// Partition-key expression, or an empty string when absent.
    pub partition_key: String,
    /// Sorting-key expression, or an empty string when absent.
    pub sorting_key: String,
    /// Primary-key expression, or an empty string when absent.
    pub primary_key: String,
    /// Sampling-key expression, or an empty string when absent.
    pub sampling_key: String,
    /// Storage policy selected by the table.
    pub storage_policy: String,
    /// Materialized-view target when the view uses `TO`.
    pub target: Option<String>,
    /// Optional table or view comment.
    pub comment: Option<String>,
    /// Columns in catalog ordinal order.
    pub columns: Vec<Column>,
    /// Data-skipping indexes in name order.
    pub data_skipping_indexes: Vec<DataSkippingIndex>,
    /// Projections in name order.
    pub projections: Vec<Projection>,
    /// Check and assume constraints in name order.
    pub constraints: Vec<Constraint>,
    /// Exact normalized creation statement retained as a fidelity backstop.
    pub definition: String,
}

impl Table {
    /// Returns the database-qualified object name.
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.database, self.name)
    }
}

/// ClickHouse table-engine object family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TableKind {
    /// A data-bearing or external table engine.
    Table,
    /// An ordinary logical view.
    View,
    /// A materialized view.
    MaterializedView,
    /// A live view.
    LiveView,
    /// A window view.
    WindowView,
    /// A dictionary exposed through the table catalog.
    Dictionary,
}

/// One ClickHouse column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Column {
    /// Column name.
    pub name: String,
    /// Complete ClickHouse type expression.
    pub data_type: String,
    /// One-based catalog ordinal.
    pub position: u64,
    /// Default, materialization, alias, or ephemeral behavior.
    pub default_kind: ColumnDefaultKind,
    /// Associated expression when a default kind is present.
    pub default_expression: Option<String>,
    /// Optional column comment.
    pub comment: Option<String>,
    /// Effective compression codec expression.
    pub compression_codec: Option<String>,
    /// Whether the column participates in the partition key.
    pub in_partition_key: bool,
    /// Whether the column participates in the sorting key.
    pub in_sorting_key: bool,
    /// Whether the column participates in the primary key.
    pub in_primary_key: bool,
    /// Whether the column participates in the sampling key.
    pub in_sampling_key: bool,
}

/// ClickHouse column expression kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "raw", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ColumnDefaultKind {
    /// No default expression.
    None,
    /// A `DEFAULT` expression.
    Default,
    /// A `MATERIALIZED` expression.
    Materialized,
    /// An `ALIAS` expression.
    Alias,
    /// An `EPHEMERAL` expression.
    Ephemeral,
    /// A newer server value retained without flattening it.
    Unknown(String),
}

impl ColumnDefaultKind {
    /// Returns the stable normalized kind name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Default => "default",
            Self::Materialized => "materialized",
            Self::Alias => "alias",
            Self::Ephemeral => "ephemeral",
            Self::Unknown(value) => value,
        }
    }
}

/// One ClickHouse data-skipping index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct DataSkippingIndex {
    /// Index name.
    pub name: String,
    /// Index expression.
    pub expression: String,
    /// Short index type.
    pub index_type: String,
    /// Complete type expression including parameters.
    pub type_full: String,
    /// Whether ClickHouse created the index implicitly, or unknown on older servers.
    pub implicit: Option<bool>,
    /// Index granularity.
    pub granularity: u64,
}

/// One ClickHouse table projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Projection {
    /// Projection name.
    pub name: String,
    /// Projection category reported by ClickHouse.
    pub projection_type: String,
    /// Projection sorting key.
    pub sorting_key: String,
    /// Normalized projection query.
    pub query: String,
}

/// One ClickHouse table constraint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Constraint {
    /// Constraint name.
    pub name: String,
    /// ClickHouse constraint category.
    pub constraint_type: String,
    /// Constraint expression.
    pub expression: String,
}

/// One SQL user-defined function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct UserDefinedFunction {
    /// Global function name.
    pub name: String,
    /// Exact normalized creation statement.
    pub definition: String,
}
