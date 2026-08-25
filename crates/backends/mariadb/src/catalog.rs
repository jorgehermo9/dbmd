use dbmd_core::SourceSnapshot;
use dbmd_relational::{ForeignKeyAction, ForeignKeyMatch, IndexSortOrder};
use serde::Serialize;

pub type Snapshot = SourceSnapshot<Catalog>;

macro_rules! semantic_enum {
    ($(#[$meta:meta])* pub enum $name:ident { $($variant:ident => $label:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
        #[serde(rename_all = "snake_case")]
        #[non_exhaustive]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            #[must_use]
            pub const fn display_name(self) -> &'static str {
                match self {
                    $(Self::$variant => $label),+
                }
            }
        }
    };
}

semantic_enum! {
    /// Declaration site reported for a check constraint.
    pub enum CheckConstraintLevel {
        Column => "column",
        Table => "table",
    }
}

semantic_enum! {
    /// SQL view check behavior.
    pub enum ViewCheckOption {
        None => "none",
        Cascaded => "cascaded",
        Local => "local",
    }
}

semantic_enum! {
    /// Execution strategy selected for a view.
    pub enum ViewAlgorithm {
        Undefined => "undefined",
        Merge => "merge",
        TemporaryTable => "temporary table",
    }
}

semantic_enum! {
    /// Invoker used for permission checks on a stored object.
    pub enum SqlSecurity {
        Definer => "definer",
        Invoker => "invoker",
    }
}

semantic_enum! {
    /// Stored routine kind.
    pub enum RoutineKind {
        Function => "function",
        Procedure => "procedure",
    }
}

semantic_enum! {
    /// Declared SQL data access behavior for a routine.
    pub enum RoutineDataAccess {
        ContainsSql => "contains SQL",
        NoSql => "no SQL",
        ReadsSqlData => "reads SQL data",
        ModifiesSqlData => "modifies SQL data",
    }
}

semantic_enum! {
    /// Stored routine parameter direction.
    pub enum ParameterMode {
        In => "in",
        Out => "out",
        InOut => "in/out",
    }
}

semantic_enum! {
    /// Row event that activates a trigger.
    pub enum TriggerEvent {
        Insert => "insert",
        Update => "update",
        Delete => "delete",
    }
}

semantic_enum! {
    /// Trigger execution timing relative to its row event.
    pub enum TriggerTiming {
        Before => "before",
        After => "after",
    }
}

semantic_enum! {
    /// Trigger execution orientation.
    pub enum TriggerOrientation {
        Row => "for each row",
    }
}

semantic_enum! {
    /// Scheduling shape of a MariaDB event.
    pub enum ScheduledEventKind {
        OneTime => "one time",
        Recurring => "recurring",
    }
}

semantic_enum! {
    /// Unit used by a recurring scheduled event.
    pub enum ScheduledIntervalUnit {
        Year => "year",
        Quarter => "quarter",
        Month => "month",
        Week => "week",
        Day => "day",
        Hour => "hour",
        Minute => "minute",
        Second => "second",
        Microsecond => "microsecond",
        YearMonth => "year to month",
        DayHour => "day to hour",
        DayMinute => "day to minute",
        DaySecond => "day to second",
        HourMinute => "hour to minute",
        HourSecond => "hour to second",
        MinuteSecond => "minute to second",
        DayMicrosecond => "day to microsecond",
        HourMicrosecond => "hour to microsecond",
        MinuteMicrosecond => "minute to microsecond",
        SecondMicrosecond => "second to microsecond",
    }
}

semantic_enum! {
    /// Storage behavior of a generated column.
    pub enum GeneratedColumnStorage {
        Virtual => "virtual",
        Stored => "stored",
    }
}

semantic_enum! {
    /// Partition routing strategy.
    pub enum PartitionMethod {
        Range => "range",
        RangeColumns => "range columns",
        List => "list",
        ListColumns => "list columns",
        Hash => "hash",
        LinearHash => "linear hash",
        Key => "key",
        LinearKey => "linear key",
        SystemTime => "system time",
    }
}

semantic_enum! {
    /// Runtime state of a scheduled event.
    pub enum ScheduledEventStatus {
        Enabled => "enabled",
        Disabled => "disabled",
        ReplicaSideDisabled => "disabled on replica",
    }
}

semantic_enum! {
    /// Whether a completed scheduled event remains in the catalog.
    pub enum ScheduledEventCompletion {
        Preserve => "preserve",
        Drop => "drop",
    }
}

semantic_enum! {
    /// Return-value family encoded by `mysql.func.ret`.
    pub enum LoadableFunctionReturnType {
        String => "string",
        Real => "real number",
        Integer => "integer",
        Row => "row",
        Decimal => "decimal",
        Temporal => "temporal",
    }
}

semantic_enum! {
    /// Execution shape of a loadable function.
    pub enum LoadableFunctionKind {
        Scalar => "loadable function",
        Aggregate => "aggregate loadable function",
    }
}

semantic_enum! {
    /// Runtime state reported for a server plugin.
    pub enum PluginStatus {
        Active => "active",
        Inactive => "inactive",
        Disabled => "disabled",
        Deleted => "deleted",
    }
}

semantic_enum! {
    /// Server plugin API family.
    pub enum PluginKind {
        Udf => "user-defined function",
        StorageEngine => "storage engine",
        FullTextParser => "full-text parser",
        Daemon => "daemon",
        InformationSchema => "information schema",
        Audit => "audit",
        Replication => "replication",
        Authentication => "authentication",
        PasswordValidation => "password validation",
        Encryption => "encryption",
        DataType => "data type",
        Function => "native function",
    }
}

semantic_enum! {
    /// License family declared by a plugin.
    pub enum PluginLicense {
        Proprietary => "proprietary",
        Gpl => "GPL",
        Bsd => "BSD",
    }
}

semantic_enum! {
    /// Server-start activation policy for a plugin.
    pub enum PluginLoadOption {
        Off => "off",
        On => "on",
        Force => "required",
        ForcePlusPermanent => "required and permanent",
    }
}

semantic_enum! {
    /// Stability level declared by a MariaDB plugin.
    pub enum PluginMaturity {
        Unknown => "unknown",
        Experimental => "experimental",
        Alpha => "alpha",
        Beta => "beta",
        Gamma => "gamma",
        Stable => "stable",
    }
}

semantic_enum! {
    /// TLS requirement attached to an account.
    pub enum TlsRequirement {
        None => "none",
        Any => "encrypted transport",
        X509 => "valid X.509 certificate",
        Specified => "specified certificate properties",
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Catalog {
    pub schemas: Vec<Schema>,
    pub tables: Vec<Table>,
    pub views: Vec<View>,
    pub sequences: Vec<Sequence>,
    pub routines: Vec<Routine>,
    pub packages: Vec<Package>,
    pub triggers: Vec<Trigger>,
    pub events: Vec<Event>,
    pub servers: Vec<ServerDefinition>,
    pub loadable_functions: Vec<LoadableFunction>,
    pub plugins: Vec<Plugin>,
    pub accounts: Vec<Account>,
    pub role_memberships: Vec<RoleMembership>,
    pub privileges: Vec<Privilege>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Schema {
    pub name: String,
    pub default_character_set: String,
    pub default_collation: String,
    pub comment: Option<String>,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Table {
    pub schema: String,
    pub name: String,
    pub engine: Option<String>,
    pub row_format: Option<String>,
    pub collation: Option<String>,
    pub comment: Option<String>,
    pub create_options: Option<String>,
    pub system_versioned: bool,
    pub system_time_period: Option<SystemTimePeriod>,
    pub application_time_periods: Vec<ApplicationTimePeriod>,
    pub columns: Vec<Column>,
    pub constraints: Vec<Constraint>,
    pub indexes: Vec<Index>,
    pub partitions: Vec<Partition>,
    pub definition: String,
}
impl Table {
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.schema, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SystemTimePeriod {
    pub start_column: String,
    pub end_column: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApplicationTimePeriod {
    pub name: String,
    pub start_column: String,
    pub end_column: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Column {
    pub name: String,
    pub position: u64,
    pub data_type: String,
    pub column_type: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub extra: String,
    pub generation_expression: Option<String>,
    pub generated_storage: Option<GeneratedColumnStorage>,
    pub character_set: Option<String>,
    pub collation: Option<String>,
    pub comment: Option<String>,
    pub visible: bool,
    pub system_time_period_start: bool,
    pub system_time_period_end: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintKind {
    PrimaryKey,
    Unique,
    ForeignKey,
    Check,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Constraint {
    pub name: String,
    pub kind: ConstraintKind,
    pub columns: Vec<String>,
    pub referenced_schema: Option<String>,
    pub referenced_table: Option<String>,
    pub referenced_columns: Vec<String>,
    pub match_type: Option<ForeignKeyMatch>,
    pub on_update: Option<ForeignKeyAction>,
    pub on_delete: Option<ForeignKeyAction>,
    pub expression: Option<String>,
    pub check_level: Option<CheckConstraintLevel>,
    pub period: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Index {
    pub name: String,
    pub unique: bool,
    pub index_type: String,
    pub ignored: Option<bool>,
    pub comment: Option<String>,
    pub catalog_comment: Option<String>,
    pub period: Option<String>,
    pub vector_options: Option<VectorIndexOptions>,
    pub terms: Vec<IndexTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VectorIndexOptions {
    pub m: Option<u64>,
    pub distance: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndexTerm {
    pub position: u64,
    pub column: String,
    pub prefix_length: Option<u64>,
    pub sort_order: Option<IndexSortOrder>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Partition {
    pub name: String,
    pub subpartition: Option<String>,
    pub method: Option<PartitionMethod>,
    pub subpartition_method: Option<PartitionMethod>,
    pub expression: Option<String>,
    pub subpartition_expression: Option<String>,
    pub description: Option<String>,
    pub ordinal: u64,
    pub subpartition_ordinal: Option<u64>,
    pub tablespace: Option<String>,
    pub comment: Option<String>,
    pub nodegroup: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct View {
    pub schema: String,
    pub name: String,
    pub definition: String,
    pub check_option: ViewCheckOption,
    pub updatable: bool,
    pub security: SqlSecurity,
    pub definer: String,
    pub character_set: String,
    pub collation: String,
    pub algorithm: ViewAlgorithm,
    pub create_statement: String,
}
impl View {
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.schema, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Sequence {
    pub schema: String,
    pub name: String,
    pub data_type: String,
    pub numeric_precision: u64,
    pub numeric_precision_radix: u64,
    pub numeric_scale: u64,
    pub start_value: String,
    pub minimum_value: String,
    pub maximum_value: String,
    pub increment: String,
    pub cache: Option<u64>,
    pub cycle: bool,
    pub engine: Option<String>,
    pub comment: Option<String>,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Routine {
    pub schema: String,
    pub name: String,
    pub kind: RoutineKind,
    pub return_type: Option<String>,
    pub definition: Option<String>,
    pub deterministic: bool,
    pub data_access: RoutineDataAccess,
    pub security: SqlSecurity,
    pub definer: String,
    pub comment: Option<String>,
    pub parameters: Vec<Parameter>,
    pub sql_mode: String,
    pub character_set_client: String,
    pub collation_connection: String,
    pub database_collation: String,
    pub create_statement: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Parameter {
    pub position: u64,
    pub mode: Option<ParameterMode>,
    pub name: Option<String>,
    pub data_type: String,
    pub dtd_identifier: String,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Package {
    pub schema: String,
    pub name: String,
    pub definer: String,
    pub security: SqlSecurity,
    pub comment: Option<String>,
    pub specification: StoredProgramDefinition,
    pub body: Option<StoredProgramDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StoredProgramDefinition {
    pub definition: String,
    pub sql_mode: String,
    pub character_set_client: String,
    pub collation_connection: String,
    pub database_collation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Trigger {
    pub schema: String,
    pub name: String,
    pub table: String,
    pub events: Vec<TriggerEvent>,
    pub update_columns: Vec<String>,
    pub timing: TriggerTiming,
    pub orientation: TriggerOrientation,
    pub statement: String,
    pub action_order: u64,
    pub sql_mode: String,
    pub definer: String,
    pub character_set_client: String,
    pub collation_connection: String,
    pub database_collation: String,
    pub create_statement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Event {
    pub schema: String,
    pub name: String,
    pub definer: String,
    pub time_zone: String,
    pub kind: ScheduledEventKind,
    pub execute_at: Option<String>,
    pub interval_value: Option<String>,
    pub interval_unit: Option<ScheduledIntervalUnit>,
    pub starts: Option<String>,
    pub ends: Option<String>,
    pub status: ScheduledEventStatus,
    pub completion: ScheduledEventCompletion,
    pub comment: Option<String>,
    pub definition: String,
    pub sql_mode: String,
    pub originator: u64,
    pub character_set_client: String,
    pub collation_connection: String,
    pub database_collation: String,
    pub create_statement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoadableFunction {
    pub name: String,
    pub return_type: LoadableFunctionReturnType,
    pub library: String,
    pub kind: LoadableFunctionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub status: PluginStatus,
    pub kind: PluginKind,
    pub type_version: String,
    pub library: Option<String>,
    pub library_version: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub license: PluginLicense,
    pub load_option: PluginLoadOption,
    pub maturity: PluginMaturity,
    pub authentication_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerDefinition {
    pub name: String,
    pub wrapper: String,
    pub host: Option<String>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub port: Option<u16>,
    pub socket: Option<String>,
    pub owner: Option<String>,
    pub options: Vec<ServerOption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerOption {
    pub name: String,
    pub value: Option<String>,
    pub sensitive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountKind {
    User,
    Role,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Account {
    pub name: String,
    pub host: String,
    pub kind: AccountKind,
    pub authentication_plugins: Vec<String>,
    pub password_expired: bool,
    pub password_lifetime_days: Option<u64>,
    pub account_locked: bool,
    pub default_role: Option<String>,
    pub tls_requirement: TlsRequirement,
    pub tls_cipher: Option<String>,
    pub x509_issuer: Option<String>,
    pub x509_subject: Option<String>,
    pub max_queries_per_hour: Option<u64>,
    pub max_updates_per_hour: Option<u64>,
    pub max_connections_per_hour: Option<u64>,
    pub max_user_connections: Option<u64>,
    pub max_statement_time: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoleMembership {
    pub user: String,
    pub host: String,
    pub role: String,
    pub admin_option: bool,
}

semantic_enum! {
    /// Kind of object to which an account privilege applies.
    pub enum PrivilegeObjectKind {
        Global => "global",
        Schema => "schema",
        Table => "table",
        Column => "column",
        Function => "function",
        Procedure => "procedure",
        Package => "package",
        PackageBody => "package body",
        Proxy => "proxy account",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Privilege {
    pub grantee: String,
    pub object_kind: PrivilegeObjectKind,
    pub schema: Option<String>,
    pub object: Option<String>,
    pub column: Option<String>,
    pub privilege: String,
    pub grantable: bool,
}
