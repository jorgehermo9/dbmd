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
    /// Workload class governed by a resource group.
    pub enum ResourceGroupKind {
        User => "user threads",
        System => "system threads",
    }
}

semantic_enum! {
    /// Catalog validity of a JSON relational duality view.
    pub enum JsonDualityViewStatus {
        Valid => "valid",
        Invalid => "invalid"
    }
}

semantic_enum! {
    /// Return family of a server loadable function.
    pub enum LoadableFunctionReturnType {
        Integer => "integer",
        Decimal => "decimal",
        Real => "real",
        Character => "character",
        Row => "row"
    }
}

semantic_enum! {
    /// Execution family of a server loadable function.
    pub enum LoadableFunctionKind {
        Scalar => "scalar",
        Aggregate => "aggregate"
    }
}

semantic_enum! {
    /// Plugin family defined by the MySQL 9.7 plugin ABI.
    pub enum PluginKind {
        LoadableFunction => "loadable function",
        StorageEngine => "storage engine",
        FullTextParser => "full-text parser",
        Daemon => "daemon",
        InformationSchema => "information schema",
        Audit => "audit",
        Replication => "replication",
        Authentication => "authentication",
        PasswordValidation => "password validation",
        GroupReplication => "group replication",
        Keyring => "keyring",
        Clone => "clone"
    }
}

semantic_enum! {
    /// License identifier defined by the MySQL 9.7 plugin ABI.
    pub enum PluginLicense {
        Proprietary => "proprietary",
        Gpl => "GPL",
        Bsd => "BSD"
    }
}

semantic_enum! {
    /// Runtime state reported for a server plugin.
    pub enum PluginStatus {
        Active => "active",
        Inactive => "inactive",
        Disabled => "disabled",
        Deleting => "deleting",
        Deleted => "deleted",
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
    /// TLS requirement attached to an account.
    pub enum TlsRequirement {
        None => "none",
        Any => "encrypted transport",
        X509 => "valid X.509 certificate",
        Specified => "specified certificate properties",
    }
}

semantic_enum! {
    /// Kind of object to which a privilege applies.
    pub enum PrivilegeObjectKind {
        Global => "global",
        Schema => "schema",
        Table => "table",
        Column => "column",
        Function => "function",
        Procedure => "procedure",
        Proxy => "proxy account",
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
    /// Scheduling shape of a MySQL event.
    pub enum ScheduledEventKind {
        OneTime => "one time",
        Recurring => "recurring",
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Catalog {
    pub schemas: Vec<Schema>,
    pub tables: Vec<Table>,
    pub views: Vec<View>,
    pub routines: Vec<Routine>,
    pub triggers: Vec<Trigger>,
    pub events: Vec<Event>,
    pub libraries: Vec<Library>,
    pub servers: Vec<ServerDefinition>,
    pub spatial_reference_systems: Vec<SpatialReferenceSystem>,
    pub tablespaces: Vec<Tablespace>,
    pub resource_groups: Vec<ResourceGroup>,
    pub loadable_functions: Vec<LoadableFunction>,
    pub plugins: Vec<Plugin>,
    pub components: Vec<Component>,
    pub accounts: Vec<Account>,
    pub role_grants: Vec<RoleGrant>,
    pub default_roles: Vec<DefaultRole>,
    pub privileges: Vec<Privilege>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ServerDefinition {
    pub name: String,
    pub wrapper: String,
    pub host: String,
    pub database: String,
    pub username: String,
    pub port: u16,
    pub socket: String,
    pub owner: String,
    pub password_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct SpatialReferenceSystem {
    pub id: u32,
    pub name: String,
    pub organization: Option<String>,
    pub organization_id: Option<u32>,
    pub definition: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Tablespace {
    pub name: String,
    pub engine: String,
    pub row_format: Option<String>,
    pub page_size: Option<u64>,
    pub autoextend_size: u64,
    pub space_type: String,
    pub encryption: Option<String>,
    pub engine_attribute: Option<String>,
    pub file_locations_redacted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ResourceGroup {
    pub name: String,
    pub kind: ResourceGroupKind,
    pub enabled: bool,
    pub virtual_cpus: String,
    pub thread_priority: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct LoadableFunction {
    pub name: String,
    pub return_type: LoadableFunctionReturnType,
    pub library: Option<String>,
    pub kind: LoadableFunctionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Component {
    pub urn: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Account {
    pub user: String,
    pub host: String,
    pub authentication_factors: Vec<AuthenticationFactor>,
    pub locked: bool,
    pub password_expired: bool,
    pub password_lifetime_days: Option<u64>,
    pub password_reuse_history: Option<u64>,
    pub password_reuse_interval_days: Option<u64>,
    pub require_current_password: Option<bool>,
    pub dual_password_configured: bool,
    pub tls_requirement: TlsRequirement,
    pub tls_cipher: Option<String>,
    pub x509_issuer: Option<String>,
    pub x509_subject: Option<String>,
    pub max_queries_per_hour: u64,
    pub max_updates_per_hour: u64,
    pub max_connections_per_hour: u64,
    pub max_user_connections: u64,
    pub comment: Option<String>,
    pub attributes_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct AuthenticationFactor {
    pub position: u8,
    pub plugin: String,
    pub credential_configured: bool,
    pub passwordless: bool,
    pub registration_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct RoleGrant {
    pub role_user: String,
    pub role_host: String,
    pub member_user: String,
    pub member_host: String,
    pub admin_option: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct DefaultRole {
    pub user: String,
    pub host: String,
    pub role_user: String,
    pub role_host: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Privilege {
    pub grantee: String,
    pub object_kind: PrivilegeObjectKind,
    pub object_identity: String,
    pub privilege: String,
    pub grantable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Schema {
    pub name: String,
    pub default_character_set: String,
    pub default_collation: String,
    pub default_encryption: bool,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Table {
    pub schema: String,
    pub name: String,
    pub engine: Option<String>,
    pub row_format: Option<String>,
    pub collation: Option<String>,
    pub comment: Option<String>,
    pub create_options: Option<String>,
    pub engine_attribute: Option<String>,
    pub secondary_engine_attribute: Option<String>,
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
#[non_exhaustive]
pub struct Column {
    pub name: String,
    pub position: u64,
    pub data_type: String,
    pub column_type: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub extra: String,
    pub generation_expression: Option<String>,
    pub character_set: Option<String>,
    pub collation: Option<String>,
    pub comment: Option<String>,
    pub visible: Option<bool>,
    pub srs_id: Option<u32>,
    pub engine_attribute: Option<String>,
    pub secondary_engine_attribute: Option<String>,
    pub masking_policy_configured: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ConstraintKind {
    PrimaryKey,
    Unique,
    ForeignKey,
    Check,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
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
    pub enforced: Option<bool>,
    pub engine_attribute: Option<String>,
    pub secondary_engine_attribute: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Index {
    pub name: String,
    pub unique: bool,
    pub index_type: String,
    pub visible: Option<bool>,
    pub comment: Option<String>,
    pub disabled_reason: Option<String>,
    pub terms: Vec<IndexTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct IndexTerm {
    pub position: u64,
    pub column: Option<String>,
    pub expression: Option<String>,
    pub prefix_length: Option<u64>,
    pub sort_order: Option<IndexSortOrder>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Partition {
    pub name: String,
    pub subpartition: Option<String>,
    pub method: Option<String>,
    pub subpartition_method: Option<String>,
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
#[non_exhaustive]
pub struct View {
    pub schema: String,
    pub name: String,
    pub kind: ViewKind,
    pub definition: String,
    pub check_option: ViewCheckOption,
    pub updatable: bool,
    pub security: SqlSecurity,
    pub definer: String,
    pub character_set: String,
    pub collation: String,
    pub create_statement: String,
    pub duality: Option<JsonDualityView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct JsonDualityView {
    pub json_column_name: String,
    pub root_table_schema: String,
    pub root_table_name: String,
    pub allow_insert: bool,
    pub allow_update: bool,
    pub allow_delete: bool,
    pub read_only: bool,
    pub status: JsonDualityViewStatus,
    pub tables: Vec<JsonDualityTable>,
    pub columns: Vec<JsonDualityColumn>,
    pub links: Vec<JsonDualityLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct JsonDualityTable {
    pub schema: String,
    pub name: String,
    pub where_clause: Option<String>,
    pub allow_insert: bool,
    pub allow_update: bool,
    pub allow_delete: bool,
    pub read_only: bool,
    pub root: bool,
    pub id: u64,
    pub parent_id: Option<u64>,
    pub parent_relationship: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct JsonDualityColumn {
    pub table_schema: String,
    pub table_name: String,
    pub root_table: bool,
    pub table_id: u64,
    pub column_name: String,
    pub json_key_name: String,
    pub allow_insert: bool,
    pub allow_update: bool,
    pub allow_delete: bool,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct JsonDualityLink {
    pub parent_schema: String,
    pub parent_table: String,
    pub child_schema: String,
    pub child_table: String,
    pub parent_column: String,
    pub child_column: String,
    pub join_type: String,
    pub json_key_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ViewKind {
    Sql,
    JsonRelationalDuality,
}

impl View {
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.schema, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Routine {
    pub schema: String,
    pub name: String,
    pub kind: RoutineKind,
    pub return_type: Option<String>,
    pub body: String,
    pub definition: Option<String>,
    pub create_statement: String,
    pub external_language: Option<String>,
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
    pub libraries: Vec<RoutineLibrary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct RoutineLibrary {
    pub schema: String,
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Library {
    pub schema: String,
    pub name: String,
    pub definition: String,
    pub language: String,
    pub sql_mode: String,
    pub comment: Option<String>,
    pub creator: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Parameter {
    pub position: u64,
    pub mode: Option<ParameterMode>,
    pub name: Option<String>,
    pub data_type: String,
    pub dtd_identifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Trigger {
    pub schema: String,
    pub name: String,
    pub table: String,
    pub event: TriggerEvent,
    pub timing: TriggerTiming,
    pub orientation: TriggerOrientation,
    pub statement: String,
    pub action_order: u64,
    pub sql_mode: String,
    pub definer: String,
    pub character_set: String,
    pub collation: String,
    pub database_collation: String,
    pub create_statement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Event {
    pub schema: String,
    pub name: String,
    pub definer: String,
    pub time_zone: String,
    pub kind: ScheduledEventKind,
    pub execute_at: Option<String>,
    pub interval_value: Option<String>,
    pub interval_field: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! semantic_cases {
        ($($name:ident: $value:expr => $display:literal, $serialized:literal;)+) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!($value.display_name(), $display);
                    assert_eq!(
                        serde_json::to_string(&$value).expect("semantic enum should serialize"),
                        $serialized
                    );
                }
            )+
        };
    }

    semantic_cases! {
        presents_user_resource_group: ResourceGroupKind::User => "user threads", "\"user\"";
        presents_system_resource_group: ResourceGroupKind::System => "system threads", "\"system\"";
        presents_valid_json_duality_view: JsonDualityViewStatus::Valid => "valid", "\"valid\"";
        presents_invalid_json_duality_view: JsonDualityViewStatus::Invalid => "invalid", "\"invalid\"";
        presents_integer_loadable_return_type: LoadableFunctionReturnType::Integer => "integer", "\"integer\"";
        presents_decimal_loadable_return_type: LoadableFunctionReturnType::Decimal => "decimal", "\"decimal\"";
        presents_real_loadable_return_type: LoadableFunctionReturnType::Real => "real", "\"real\"";
        presents_character_loadable_return_type: LoadableFunctionReturnType::Character => "character", "\"character\"";
        presents_row_loadable_return_type: LoadableFunctionReturnType::Row => "row", "\"row\"";
        presents_scalar_loadable_function: LoadableFunctionKind::Scalar => "scalar", "\"scalar\"";
        presents_aggregate_loadable_function: LoadableFunctionKind::Aggregate => "aggregate", "\"aggregate\"";
        presents_loadable_function_plugin: PluginKind::LoadableFunction => "loadable function", "\"loadable_function\"";
        presents_storage_engine_plugin: PluginKind::StorageEngine => "storage engine", "\"storage_engine\"";
        presents_full_text_parser_plugin: PluginKind::FullTextParser => "full-text parser", "\"full_text_parser\"";
        presents_daemon_plugin: PluginKind::Daemon => "daemon", "\"daemon\"";
        presents_information_schema_plugin: PluginKind::InformationSchema => "information schema", "\"information_schema\"";
        presents_audit_plugin: PluginKind::Audit => "audit", "\"audit\"";
        presents_replication_plugin: PluginKind::Replication => "replication", "\"replication\"";
        presents_authentication_plugin: PluginKind::Authentication => "authentication", "\"authentication\"";
        presents_password_validation_plugin: PluginKind::PasswordValidation => "password validation", "\"password_validation\"";
        presents_group_replication_plugin: PluginKind::GroupReplication => "group replication", "\"group_replication\"";
        presents_keyring_plugin: PluginKind::Keyring => "keyring", "\"keyring\"";
        presents_clone_plugin: PluginKind::Clone => "clone", "\"clone\"";
        presents_proprietary_plugin_license: PluginLicense::Proprietary => "proprietary", "\"proprietary\"";
        presents_gpl_plugin_license: PluginLicense::Gpl => "GPL", "\"gpl\"";
        presents_bsd_plugin_license: PluginLicense::Bsd => "BSD", "\"bsd\"";
        presents_active_plugin_status: PluginStatus::Active => "active", "\"active\"";
        presents_inactive_plugin_status: PluginStatus::Inactive => "inactive", "\"inactive\"";
        presents_disabled_plugin_status: PluginStatus::Disabled => "disabled", "\"disabled\"";
        presents_deleting_plugin_status: PluginStatus::Deleting => "deleting", "\"deleting\"";
        presents_deleted_plugin_status: PluginStatus::Deleted => "deleted", "\"deleted\"";
        presents_off_plugin_load_option: PluginLoadOption::Off => "off", "\"off\"";
        presents_on_plugin_load_option: PluginLoadOption::On => "on", "\"on\"";
        presents_force_plugin_load_option: PluginLoadOption::Force => "required", "\"force\"";
        presents_permanent_plugin_load_option: PluginLoadOption::ForcePlusPermanent => "required and permanent", "\"force_plus_permanent\"";
        presents_no_tls_requirement: TlsRequirement::None => "none", "\"none\"";
        presents_any_tls_requirement: TlsRequirement::Any => "encrypted transport", "\"any\"";
        presents_x509_tls_requirement: TlsRequirement::X509 => "valid X.509 certificate", "\"x509\"";
        presents_specified_tls_requirement: TlsRequirement::Specified => "specified certificate properties", "\"specified\"";
        presents_global_privilege_object: PrivilegeObjectKind::Global => "global", "\"global\"";
        presents_schema_privilege_object: PrivilegeObjectKind::Schema => "schema", "\"schema\"";
        presents_table_privilege_object: PrivilegeObjectKind::Table => "table", "\"table\"";
        presents_column_privilege_object: PrivilegeObjectKind::Column => "column", "\"column\"";
        presents_function_privilege_object: PrivilegeObjectKind::Function => "function", "\"function\"";
        presents_procedure_privilege_object: PrivilegeObjectKind::Procedure => "procedure", "\"procedure\"";
        presents_proxy_privilege_object: PrivilegeObjectKind::Proxy => "proxy account", "\"proxy\"";
        presents_none_view_check_option: ViewCheckOption::None => "none", "\"none\"";
        presents_cascaded_view_check_option: ViewCheckOption::Cascaded => "cascaded", "\"cascaded\"";
        presents_local_view_check_option: ViewCheckOption::Local => "local", "\"local\"";
        presents_definer_sql_security: SqlSecurity::Definer => "definer", "\"definer\"";
        presents_invoker_sql_security: SqlSecurity::Invoker => "invoker", "\"invoker\"";
        presents_function_routine_kind: RoutineKind::Function => "function", "\"function\"";
        presents_procedure_routine_kind: RoutineKind::Procedure => "procedure", "\"procedure\"";
        presents_contains_sql_data_access: RoutineDataAccess::ContainsSql => "contains SQL", "\"contains_sql\"";
        presents_no_sql_data_access: RoutineDataAccess::NoSql => "no SQL", "\"no_sql\"";
        presents_reads_sql_data_access: RoutineDataAccess::ReadsSqlData => "reads SQL data", "\"reads_sql_data\"";
        presents_modifies_sql_data_access: RoutineDataAccess::ModifiesSqlData => "modifies SQL data", "\"modifies_sql_data\"";
        presents_in_parameter_mode: ParameterMode::In => "in", "\"in\"";
        presents_out_parameter_mode: ParameterMode::Out => "out", "\"out\"";
        presents_inout_parameter_mode: ParameterMode::InOut => "in/out", "\"in_out\"";
        presents_insert_trigger_event: TriggerEvent::Insert => "insert", "\"insert\"";
        presents_update_trigger_event: TriggerEvent::Update => "update", "\"update\"";
        presents_delete_trigger_event: TriggerEvent::Delete => "delete", "\"delete\"";
        presents_before_trigger_timing: TriggerTiming::Before => "before", "\"before\"";
        presents_after_trigger_timing: TriggerTiming::After => "after", "\"after\"";
        presents_row_trigger_orientation: TriggerOrientation::Row => "for each row", "\"row\"";
        presents_one_time_scheduled_event: ScheduledEventKind::OneTime => "one time", "\"one_time\"";
        presents_recurring_scheduled_event: ScheduledEventKind::Recurring => "recurring", "\"recurring\"";
        presents_enabled_scheduled_event: ScheduledEventStatus::Enabled => "enabled", "\"enabled\"";
        presents_disabled_scheduled_event: ScheduledEventStatus::Disabled => "disabled", "\"disabled\"";
        presents_replica_disabled_scheduled_event: ScheduledEventStatus::ReplicaSideDisabled => "disabled on replica", "\"replica_side_disabled\"";
        presents_preserved_scheduled_event: ScheduledEventCompletion::Preserve => "preserve", "\"preserve\"";
        presents_dropped_scheduled_event: ScheduledEventCompletion::Drop => "drop", "\"drop\"";
    }
}
