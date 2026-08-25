//! ClickHouse-owned normalized catalog.

use dbmd_core::SourceSnapshot;
use serde::{Deserialize, Serialize};

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
    /// Users visible through the server access-control catalog, in name order.
    pub users: Vec<User>,
    /// Roles visible through the server access-control catalog, in name order.
    pub roles: Vec<Role>,
    /// Privilege grants in deterministic subject and object order.
    pub grants: Vec<Grant>,
    /// Role-to-user and role-to-role grants in deterministic order.
    pub role_grants: Vec<RoleGrant>,
    /// Row policies in qualified-name order.
    pub row_policies: Vec<RowPolicy>,
    /// Quotas and their limits in name order.
    pub quotas: Vec<Quota>,
    /// Settings profiles and their ordered elements.
    pub settings_profiles: Vec<SettingsProfile>,
    /// Named-collection identities and key names; values are never acquired.
    pub named_collections: Vec<NamedCollection>,
    /// Workload-scheduler resources in name order.
    pub resources: Vec<Resource>,
    /// Workload-scheduler workload hierarchy in name order.
    pub workloads: Vec<Workload>,
}

/// One selected ClickHouse database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Database {
    /// Database name.
    pub name: String,
    /// Database engine.
    pub engine: String,
    /// Stable database UUID.
    pub uuid: String,
    /// Full database-engine expression.
    pub engine_full: String,
    /// Whether the database is backed by an external catalog integration.
    pub external: bool,
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
    /// Stable table UUID.
    pub uuid: String,
    /// Object family derived from its engine.
    pub kind: TableKind,
    /// Storage or view engine name.
    pub engine: String,
    /// Complete engine expression including parameters and table clauses.
    pub engine_full: String,
    /// Positional table-engine arguments in declared order.
    pub engine_arguments: Vec<String>,
    /// Named table-engine parameters in deterministic name order.
    pub engine_parameters: std::collections::BTreeMap<String, String>,
    /// Stable/effective table-engine settings in deterministic name order.
    pub settings: std::collections::BTreeMap<String, String>,
    /// Table TTL rules in declaration order.
    pub ttl_rules: Vec<TableTtl>,
    /// Whether the object is connection-local.
    pub temporary: bool,
    /// Partition-key expression, or an empty string when absent.
    pub partition_key: String,
    /// Sorting-key expression, or an empty string when absent.
    pub sorting_key: String,
    /// Primary-key expression, or an empty string when absent.
    pub primary_key: String,
    /// Sampling-key expression, or an empty string when absent.
    pub sampling_key: String,
    /// Unique-key expression, or an empty string when absent.
    pub unique_key: String,
    /// Storage policy selected by the table.
    pub storage_policy: String,
    /// Materialized-view target when the view uses `TO`.
    pub target: Option<String>,
    /// Refresh contract for a refreshable materialized view.
    pub refresh: Option<ViewRefresh>,
    /// Window-specific execution contract for a window view.
    pub window: Option<WindowView>,
    /// Normalized `AS SELECT` query, when the object was defined from a query.
    pub as_select: Option<String>,
    /// Parameter declarations for a parameterized view.
    pub parameters: Vec<ViewParameter>,
    /// Objects this table depends on for query semantics.
    pub dependencies: Vec<TableReference>,
    /// Objects that must load before this table.
    pub loading_dependencies: Vec<TableReference>,
    /// Objects that load after this table.
    pub loading_dependents: Vec<TableReference>,
    /// Definer identity for view-like objects.
    pub definer: Option<String>,
    /// Execution identity contract for view-like objects.
    pub sql_security: Option<ViewSqlSecurity>,
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
    /// Dictionary-owned source, layout, key, attribute, and lifetime metadata.
    pub dictionary: Option<DictionaryDetails>,
    /// Exact normalized creation statement retained as a fidelity backstop.
    pub definition: String,
}

/// Qualified ClickHouse table identity used in dependency edges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TableReference {
    pub database: String,
    pub table: String,
}

/// One parameter declared by a parameterized ClickHouse view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: String,
}

/// Execution identity used for a ClickHouse view query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
#[non_exhaustive]
pub enum ViewSqlSecurity {
    /// Execute using the view definer's privileges.
    Definer,
    /// Execute using the invoking user's privileges.
    Invoker,
    /// Execute without a privilege-checking identity.
    None,
    /// A server value unknown to this dbmd version.
    Unknown {
        /// Exact normalized value retained for forward compatibility.
        raw: String,
    },
}

/// Retained execution contract for a ClickHouse window view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct WindowView {
    /// Explicit target table selected with `TO`.
    pub target: Option<String>,
    /// Explicit inner engine expression, when declared.
    pub inner_engine: Option<String>,
    /// Explicit result storage engine expression, when declared.
    pub storage_engine: Option<String>,
    /// Watermark strategy or expression.
    pub watermark: Option<String>,
    /// Accepted lateness interval expression.
    pub allowed_lateness: Option<String>,
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
    pub character_octet_length: Option<u64>,
    pub numeric_precision: Option<u64>,
    pub numeric_precision_radix: Option<u64>,
    pub numeric_scale: Option<u64>,
    pub datetime_precision: Option<u64>,
    /// Default, materialization, alias, or ephemeral behavior.
    pub default_kind: ColumnDefaultKind,
    /// Associated expression when a default kind is present.
    pub default_expression: Option<String>,
    /// Optional column comment.
    pub comment: Option<String>,
    /// Effective compression codec expression.
    pub compression_codec: Option<String>,
    /// Effective serialization hint.
    pub serialization_hint: Option<String>,
    /// Normalized statistics declaration.
    pub statistics: Option<String>,
    /// Column TTL expression, when declared.
    pub ttl: Option<String>,
    /// Whether the column participates in the partition key.
    pub in_partition_key: bool,
    /// Whether the column participates in the sorting key.
    pub in_sorting_key: bool,
    /// Whether the column participates in the primary key.
    pub in_primary_key: bool,
    /// Whether the column participates in the sampling key.
    pub in_sampling_key: bool,
}

/// One table-level ClickHouse TTL rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TableTtl {
    pub expression: String,
    pub action: TtlAction,
    pub raw: String,
}

/// Stable action selected by a table TTL rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TtlAction {
    Delete {
        predicate: Option<String>,
    },
    Move {
        destination: TtlDestination,
        target: String,
    },
    Recompress {
        codec: String,
    },
    GroupBy {
        keys: String,
        assignments: Vec<String>,
    },
    Unknown {
        raw: String,
    },
}

/// Storage target category for a TTL move action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TtlDestination {
    Disk,
    Volume,
}

/// Stable refresh contract for a refreshable materialized view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ViewRefresh {
    /// Time/dependency scheduling mode.
    pub schedule: RefreshSchedule,
    /// Optional alignment offset for an `EVERY` schedule.
    pub offset: Option<String>,
    /// Optional schedule jitter interval.
    pub randomize_for: Option<String>,
    /// Refreshable views that must complete before this view refreshes.
    pub dependencies: Vec<TableReference>,
    /// Refresh-specific settings in deterministic name order.
    pub settings: std::collections::BTreeMap<String, String>,
    /// Whether refreshes append instead of atomically replacing prior results.
    pub append: bool,
}

/// Refreshable materialized-view scheduling mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RefreshSchedule {
    Every { interval: String },
    After { interval: String },
    DependenciesOnly,
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
        }
    }
}

/// Semantics of a ClickHouse table constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ConstraintKind {
    /// Enforced for newly inserted rows.
    Check,
    /// Assumed by query optimization but not enforced on insert.
    Assume,
}

impl ConstraintKind {
    /// Returns the stable human-facing kind name.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Check => "check",
            Self::Assume => "assumption",
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
    /// Projection-local settings in deterministic key order.
    pub settings: std::collections::BTreeMap<String, String>,
    /// Secondary-index role when created with `PROJECTION ... INDEX`.
    pub index: Option<ProjectionIndex>,
}

/// Typed projection-index declaration introduced before ClickHouse 26.6.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectionIndex {
    pub expression: String,
    pub index_type: String,
}

/// One ClickHouse table constraint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Constraint {
    /// Constraint name.
    pub name: String,
    /// ClickHouse constraint category.
    pub kind: ConstraintKind,
    /// Constraint expression.
    pub expression: String,
}

/// Stable schema metadata for a ClickHouse dictionary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct DictionaryDetails {
    /// Allocation/layout family such as `Hashed`.
    pub layout: String,
    /// Dictionary key columns in declared order.
    pub keys: Vec<DictionaryField>,
    /// Dictionary attributes in declared order.
    pub attributes: Vec<DictionaryField>,
    /// Server-provided non-secret source description.
    pub source: String,
    /// Minimum refresh lifetime in seconds.
    pub lifetime_min_seconds: u64,
    /// Maximum refresh lifetime in seconds.
    pub lifetime_max_seconds: u64,
    /// Range lower-bound field for range-hashed dictionaries.
    pub range_min: Option<String>,
    /// Range upper-bound field for range-hashed dictionaries.
    pub range_max: Option<String>,
    /// Dictionary-owned settings in deterministic name order.
    pub settings: std::collections::BTreeMap<String, String>,
}

/// One typed dictionary key or attribute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct DictionaryField {
    /// Field name.
    pub name: String,
    /// Field data type.
    pub data_type: String,
    /// Default expression used when the source value is absent.
    pub default_expression: Option<String>,
    /// Source-side expression used to derive the field.
    pub expression: Option<String>,
    /// Whether the attribute defines a dictionary hierarchy.
    pub hierarchical: bool,
    /// Whether the attribute is injective.
    pub injective: bool,
    /// Whether the key field is the object identifier.
    pub object_id: bool,
}

/// One SQL user-defined function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct UserDefinedFunction {
    /// Global function name.
    pub name: String,
    /// Catalog origin such as `SQLUserDefined` or `WasmUserDefined`.
    pub origin: String,
    /// Server-provided invocation syntax, when available.
    pub syntax: Option<String>,
    /// Server-provided argument signature, when available.
    pub arguments: Option<String>,
    /// Server-provided return type, when available.
    pub returned_value: Option<String>,
    /// Exact normalized creation statement.
    pub definition: String,
}

/// One ClickHouse user with authentication identities but no credential material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct User {
    pub name: String,
    pub storage: String,
    pub authentication_types: Vec<String>,
    /// Expiration aligned with each authentication method; `None` means no expiry.
    pub valid_until: Vec<Option<String>>,
    pub hosts: UserHosts,
    pub default_roles: AccessTarget,
    pub grantees: AccessTarget,
    pub default_database: Option<String>,
}

/// Host selectors accepted by a ClickHouse user.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct UserHosts {
    pub ip: Vec<String>,
    pub names: Vec<String>,
    pub name_regexps: Vec<String>,
    pub name_like_patterns: Vec<String>,
}

/// A target list represented by ClickHouse's `ALL`, include, and exception fields.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AccessTarget {
    pub all: bool,
    pub include: Vec<String>,
    pub except: Vec<String>,
}

/// One ClickHouse role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Role {
    pub name: String,
    pub storage: String,
}

/// One privilege grant or partial revoke.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Grant {
    pub user: Option<String>,
    pub role: Option<String>,
    pub access_type: String,
    pub access_object: Option<String>,
    pub database: Option<String>,
    pub table: Option<String>,
    pub column: Option<String>,
    pub partial_revoke: bool,
    pub grant_option: bool,
}

/// One granted role and its default/admin semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoleGrant {
    pub user: Option<String>,
    pub role: Option<String>,
    pub granted_role: String,
    pub default: bool,
    pub admin_option: bool,
}

/// One row-level SELECT policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RowPolicy {
    pub name: String,
    pub short_name: String,
    pub database: String,
    pub table: Option<String>,
    pub storage: String,
    pub select_filter: Option<String>,
    pub restrictive: bool,
    pub target: AccessTarget,
}

/// One quota with all normalized 26.6 limit families.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Quota {
    pub name: String,
    pub storage: String,
    pub keys: Vec<String>,
    pub target: AccessTarget,
    pub ipv4_prefix_bits: Option<u8>,
    pub ipv6_prefix_bits: Option<u8>,
    pub limits: Vec<QuotaLimit>,
}

/// Limits for one quota interval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuotaLimit {
    pub duration_seconds: u64,
    pub randomized: bool,
    pub max_queries: Option<u64>,
    pub max_query_selects: Option<u64>,
    pub max_query_inserts: Option<u64>,
    pub max_errors: Option<u64>,
    pub max_result_rows: Option<u64>,
    pub max_result_bytes: Option<u64>,
    pub max_read_rows: Option<u64>,
    pub max_read_bytes: Option<u64>,
    pub max_execution_time: Option<String>,
    pub max_written_bytes: Option<u64>,
    pub max_failed_sequential_authentications: Option<u64>,
    pub max_queries_per_normalized_hash: Option<u64>,
}

/// One ClickHouse settings profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SettingsProfile {
    pub name: String,
    pub storage: String,
    pub target: AccessTarget,
    pub elements: Vec<SettingsProfileElement>,
}

/// One ordered settings-profile value, constraint, or inherited profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SettingsProfileElement {
    pub index: u64,
    pub setting_name: Option<String>,
    pub value: Option<String>,
    pub minimum: Option<String>,
    pub maximum: Option<String>,
    pub writability: Option<String>,
    pub inherited_profile: Option<String>,
}

/// Safe metadata for a named collection.
///
/// Collection values are deliberately absent so secrets never cross the
/// acquisition boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct NamedCollection {
    /// Collection name.
    pub name: String,
    /// Value-free collection entries in key order.
    pub entries: Vec<NamedCollectionEntry>,
    /// Definition after server-side value redaction.
    pub definition: Option<String>,
    /// Catalog source such as `SQL` or configuration.
    pub source: String,
}

/// One named-collection key without its potentially secret value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct NamedCollectionEntry {
    /// Entry key.
    pub key: String,
    /// Whether callers may override the entry, when catalog-visible.
    pub overridable: Option<bool>,
}

/// One workload-scheduler resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Resource {
    pub name: String,
    pub operations: Vec<ResourceOperation>,
    pub read_disks: Vec<String>,
    pub write_disks: Vec<String>,
    pub unit: String,
    pub definition: String,
}

/// One scheduler operation assigned to a resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResourceOperation {
    MasterThread,
    WorkerThread,
    Query,
    MemoryReservation,
    ReadDisk { disk: Option<String> },
    WriteDisk { disk: Option<String> },
    Unknown { raw: String },
}

/// One workload-scheduler workload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Workload {
    pub name: String,
    pub parent: Option<String>,
    pub settings: Vec<WorkloadSetting>,
    pub definition: String,
}

/// One ordered workload setting, optionally scoped to a resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkloadSetting {
    pub name: String,
    pub value: String,
    pub resource: Option<String>,
}
