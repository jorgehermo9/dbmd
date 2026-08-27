//! PostgreSQL-owned normalized catalog.

use dbmd_core::SourceSnapshot;
use serde::Serialize;

use dbmd_relational::{ForeignKeyReference, IndexSortOrder};

/// A normalized PostgreSQL source snapshot.
pub type Snapshot = SourceSnapshot<Catalog>;

/// PostgreSQL schema content in deterministic catalog order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Catalog {
    /// The connected database and its stable creation/configuration contract.
    pub database: Database,
    /// Cluster-wide databases when explicitly requested by source configuration.
    pub cluster_databases: Vec<Database>,
    /// Cluster-wide tablespaces when explicitly requested by source configuration.
    pub tablespaces: Vec<Tablespace>,
    /// User namespaces in deterministic name order.
    pub namespaces: Vec<Namespace>,
    /// Enum types in schema/name order.
    pub enums: Vec<EnumType>,
    /// Composite types in schema/name order.
    pub composite_types: Vec<CompositeType>,
    /// Domain types in schema/name order.
    pub domains: Vec<Domain>,
    /// User-defined base and shell types in schema/name order.
    pub base_types: Vec<BaseType>,
    /// User-defined range types and their paired multiranges in schema/name order.
    pub range_types: Vec<RangeType>,
    /// Sequences in schema/name order.
    pub sequences: Vec<Sequence>,
    /// Relations represented as tables in schema/name order.
    pub tables: Vec<Table>,
    /// Ordinary and materialized views in schema/name order.
    pub views: Vec<View>,
    /// Triggers in target schema/relation/name order.
    pub triggers: Vec<Trigger>,
    /// Functions in schema/name/signature order.
    pub functions: Vec<Function>,
    /// Procedures in schema/name/signature order.
    pub procedures: Vec<Procedure>,
    /// Aggregate functions in schema/name/signature order.
    pub aggregates: Vec<Aggregate>,
    /// User-defined casts in source/target type order.
    pub casts: Vec<Cast>,
    /// User-defined encoding conversions in schema/name order.
    pub conversions: Vec<Conversion>,
    /// User-defined operators in schema/name/signature order.
    pub operators: Vec<Operator>,
    /// User-defined operator families in access-method/schema/name order.
    pub operator_families: Vec<OperatorFamily>,
    /// User-defined operator classes in access-method/schema/name order.
    pub operator_classes: Vec<OperatorClass>,
    /// User-defined access methods in name order.
    pub access_methods: Vec<AccessMethod>,
    /// User-defined procedural languages in name order.
    pub languages: Vec<Language>,
    /// User-defined procedural-language transforms in type/language order.
    pub transforms: Vec<Transform>,
    /// User-authored rewrite rules in target schema/relation/name order.
    pub rules: Vec<RewriteRule>,
    /// Database event triggers in name order.
    pub event_triggers: Vec<EventTrigger>,
    /// Extended planner-statistics objects in schema/name order.
    pub statistics: Vec<ExtendedStatistics>,
    /// Foreign-data wrappers in name order.
    pub foreign_data_wrappers: Vec<ForeignDataWrapper>,
    /// Foreign servers in name order.
    pub foreign_servers: Vec<ForeignServer>,
    /// User mappings in server/user order, with sensitive option values redacted.
    pub user_mappings: Vec<UserMapping>,
    /// Text-search parsers in schema/name order.
    pub text_search_parsers: Vec<TextSearchParser>,
    /// Text-search templates in schema/name order.
    pub text_search_templates: Vec<TextSearchTemplate>,
    /// Text-search dictionaries in schema/name order.
    pub text_search_dictionaries: Vec<TextSearchDictionary>,
    /// Text-search configurations in schema/name order.
    pub text_search_configurations: Vec<TextSearchConfiguration>,
    /// Installed extensions in name order.
    pub extensions: Vec<Extension>,
    /// Logical-replication publications in name order.
    pub publications: Vec<Publication>,
    /// Logical-replication subscriptions in name order, without connection secrets.
    pub subscriptions: Vec<Subscription>,
    /// Explicit object and column grants in stable object/grantee order.
    pub privileges: Vec<ObjectPrivilege>,
    /// Default grants applied to objects created by a role.
    pub default_privileges: Vec<DefaultPrivilege>,
    /// Security-provider labels attached to catalog objects.
    pub security_labels: Vec<SecurityLabel>,
    /// Large-object metadata; binary contents are never acquired.
    pub large_objects: Vec<LargeObject>,
    /// User-created cluster roles in name order, without password material.
    pub roles: Vec<Role>,
    /// Per-role, per-database session defaults in database/role order.
    pub role_database_settings: Vec<RoleDatabaseSetting>,
    /// User-defined collations in schema/name order.
    pub collations: Vec<Collation>,
}

/// One tablespace; the host filesystem location is deliberately omitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Tablespace {
    /// Cluster-wide tablespace name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Tablespace options in server order.
    pub options: Vec<String>,
    /// Optional shared catalog comment.
    pub comment: Option<String>,
    /// Always true: the host filesystem location is deliberately discarded.
    pub location_redacted: bool,
}

/// One explicit PostgreSQL type-conversion path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Cast {
    /// Server-formatted source type.
    pub source_type: String,
    /// Server-formatted target type.
    pub target_type: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Contexts in which PostgreSQL may apply the cast.
    pub context: CastContext,
    /// Mechanism used to convert the value.
    pub method: CastMethod,
    /// Conversion function identity for function-based casts.
    pub function: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// Invocation context accepted by a PostgreSQL cast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CastContext {
    /// Only explicit `CAST` or `::` expressions.
    Explicit,
    /// Explicit expressions and assignment coercion.
    Assignment,
    /// Explicit, assignment, and implicit expression coercion.
    Implicit,
}

/// Conversion mechanism used by a PostgreSQL cast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CastMethod {
    /// Invoke the declared conversion function.
    Function,
    /// Pass through the source output and target input functions.
    InputOutput,
    /// Reinterpret a binary-compatible representation.
    Binary,
}

/// One named PostgreSQL character-set conversion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Conversion {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified conversion name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Source encoding name.
    pub source_encoding: String,
    /// Target encoding name.
    pub target_encoding: String,
    /// Qualified conversion procedure identity.
    pub function: String,
    /// Whether this is the default conversion for its encoding pair.
    pub default: bool,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// One PostgreSQL operator overload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Operator {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified operator symbol.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Operator arity and operand orientation.
    pub kind: OperatorKind,
    /// Server-formatted left operand type for binary operators.
    pub left_type: Option<String>,
    /// Server-formatted right operand type.
    pub right_type: String,
    /// Server-formatted result type.
    pub result_type: String,
    /// Qualified implementing function identity.
    pub function: String,
    /// Qualified commutator operator identity, when declared.
    pub commutator: Option<String>,
    /// Qualified negator operator identity, when declared.
    pub negator: Option<String>,
    /// Restriction selectivity estimator, when declared.
    pub restriction_selectivity: Option<String>,
    /// Join selectivity estimator, when declared.
    pub join_selectivity: Option<String>,
    /// Whether this operator can support merge joins.
    pub can_merge: bool,
    /// Whether this operator can support hash joins.
    pub can_hash: bool,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// PostgreSQL operator arity and operand orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OperatorKind {
    /// Binary infix operator.
    Binary,
    /// Unary prefix operator.
    Prefix,
}

/// One operator family for an index access method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct OperatorFamily {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified family name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Index access method using the family.
    pub access_method: String,
    /// Strategy operators registered in this family.
    pub operators: Vec<OperatorFamilyOperator>,
    /// Support functions registered in this family.
    pub functions: Vec<OperatorFamilyFunction>,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// One strategy operator registered in an operator family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct OperatorFamilyOperator {
    /// Left input type.
    pub left_type: String,
    /// Right input type.
    pub right_type: String,
    /// Access-method strategy number.
    pub strategy: i16,
    /// Search or ordering role.
    pub purpose: OperatorPurpose,
    /// Qualified operator identity.
    pub operator: String,
    /// Access method used to interpret the operator.
    pub access_method: String,
    /// Sort family for ordering operators.
    pub sort_family: Option<String>,
}

/// Purpose of an operator-family strategy member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OperatorPurpose {
    /// Search operator interpreted by the family access method.
    Search,
    /// Ordering operator interpreted through a B-tree sort family.
    Ordering,
}

/// One support function registered in an operator family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct OperatorFamilyFunction {
    /// Left input type.
    pub left_type: String,
    /// Right input type.
    pub right_type: String,
    /// Access-method support-function number.
    pub number: i16,
    /// Qualified function identity.
    pub function: String,
}

/// One operator class selecting a default family contract for a data type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct OperatorClass {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified class name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Index access method using the class.
    pub access_method: String,
    /// Qualified containing operator family.
    pub family: String,
    /// Server-formatted indexed input type.
    pub input_type: String,
    /// Whether this is the default class for the type/access-method pair.
    pub default: bool,
    /// Server-formatted index key storage type when different from the input type.
    pub key_type: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// One PostgreSQL table or index access method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct AccessMethod {
    /// Global access-method name.
    pub name: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Relation family implemented by the access method.
    pub kind: AccessMethodKind,
    /// Qualified handler function identity.
    pub handler: String,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// Relation family implemented by a PostgreSQL access method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AccessMethodKind {
    /// Table storage access method.
    Table,
    /// Index access method.
    Index,
}

/// One installed PostgreSQL procedural language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Language {
    /// Global language name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Whether this is a procedural language rather than an internal one.
    pub procedural: bool,
    /// Whether unprivileged users may create functions in the language.
    pub trusted: bool,
    /// Qualified call-handler identity.
    pub handler: Option<String>,
    /// Qualified anonymous-block handler identity.
    pub inline_handler: Option<String>,
    /// Qualified validator identity.
    pub validator: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// One type adaptation contract for a procedural language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Transform {
    /// Server-formatted adapted data type.
    pub data_type: String,
    /// Procedural language receiving the adapted values.
    pub language: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Function converting SQL values into the language representation.
    pub from_sql: Option<String>,
    /// Function converting language values back into SQL representation.
    pub to_sql: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// One user-authored PostgreSQL query-rewrite rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct RewriteRule {
    pub namespace: String,
    pub name: String,
    pub target: String,
    pub event: RewriteRuleEvent,
    pub instead: bool,
    pub enabled: TriggerEnabled,
    pub extension: Option<String>,
    pub comment: Option<String>,
    pub definition: String,
}

/// Command event intercepted by a query-rewrite rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RewriteRuleEvent {
    Select,
    Update,
    Insert,
    Delete,
}

/// One PostgreSQL database-level DDL event trigger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct EventTrigger {
    pub name: String,
    pub owner: String,
    pub event: EventTriggerEvent,
    pub tags: Vec<String>,
    pub function: String,
    pub enabled: TriggerEnabled,
    pub extension: Option<String>,
    pub comment: Option<String>,
    pub definition: String,
}

/// Database event that fires a PostgreSQL event trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum EventTriggerEvent {
    Login,
    DdlCommandStart,
    DdlCommandEnd,
    SqlDrop,
    TableRewrite,
}

/// One PostgreSQL extended planner-statistics object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ExtendedStatistics {
    pub namespace: String,
    pub name: String,
    pub owner: String,
    pub kinds: Vec<StatisticsKind>,
    pub columns: Vec<String>,
    pub expressions: Vec<String>,
    pub target: i32,
    pub extension: Option<String>,
    pub comment: Option<String>,
    pub definition: String,
}

/// Dependency information collected by an extended-statistics object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum StatisticsKind {
    NdDistinct,
    Dependencies,
    MostCommonValues,
    Expressions,
}

/// One PostgreSQL foreign-data wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ForeignDataWrapper {
    /// Global wrapper name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Optional handler function identity.
    pub handler: Option<String>,
    /// Optional validator function identity.
    pub validator: Option<String>,
    /// Ordered wrapper options with sensitive values redacted.
    pub options: Vec<String>,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// One named PostgreSQL foreign server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ForeignServer {
    /// Global server name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Foreign-data wrapper used by the server.
    pub wrapper: String,
    /// Optional implementation-specific server type.
    pub server_type: Option<String>,
    /// Optional implementation-specific server version.
    pub version: Option<String>,
    /// Ordered server options with sensitive values redacted.
    pub options: Vec<String>,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// One foreign-server mapping for a local role or `PUBLIC`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct UserMapping {
    /// Foreign server receiving the remote identity.
    pub server: String,
    /// Local role name or `PUBLIC`.
    pub user: String,
    /// Ordered mapping options with sensitive values redacted.
    pub options: Vec<String>,
}

/// One PostgreSQL text-search parser implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct TextSearchParser {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified parser name.
    pub name: String,
    /// Parser start function identity.
    pub start_function: String,
    /// Parser token function identity.
    pub token_function: String,
    /// Parser end function identity.
    pub end_function: String,
    /// Parser headline function identity.
    pub headline_function: String,
    /// Parser token-types function identity.
    pub token_types_function: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// One PostgreSQL text-search dictionary template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct TextSearchTemplate {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified template name.
    pub name: String,
    /// Optional dictionary initialization function identity.
    pub init_function: Option<String>,
    /// Dictionary lexize function identity.
    pub lexize_function: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// One configured PostgreSQL text-search dictionary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct TextSearchDictionary {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified dictionary name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Qualified text-search template identity.
    pub template: String,
    /// Server-normalized template options.
    pub options: Option<String>,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// One PostgreSQL text-search configuration and its token mappings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct TextSearchConfiguration {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified configuration name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Qualified parser identity.
    pub parser: String,
    /// Token mappings in parser token-type order.
    pub mappings: Vec<TextSearchMapping>,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// Ordered dictionary chain selected for one parser token type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct TextSearchMapping {
    /// Parser token-type alias.
    pub token_type: String,
    /// Qualified dictionary identities in lookup order.
    pub dictionaries: Vec<String>,
}

/// The PostgreSQL database selected by the source connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Database {
    pub name: String,
    pub owner: String,
    pub encoding: String,
    pub locale_provider: DatabaseLocaleProvider,
    pub lc_collate: String,
    pub lc_ctype: String,
    pub locale: Option<String>,
    pub icu_rules: Option<String>,
    pub collation_version: Option<String>,
    pub tablespace: String,
    pub template: bool,
    pub allow_connections: bool,
    pub connection_limit: i32,
    pub configuration: Vec<String>,
    pub comment: Option<String>,
}

/// Locale implementation selected when the database was created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DatabaseLocaleProvider {
    Builtin,
    Libc,
    Icu,
}

/// One user-visible PostgreSQL schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Namespace {
    pub name: String,
    pub owner: String,
    /// Owning extension when this is an extension support schema.
    pub extension: Option<String>,
    pub comment: Option<String>,
}

/// One PostgreSQL collation object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Collation {
    pub namespace: String,
    pub name: String,
    pub owner: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    pub provider: CollationProvider,
    pub deterministic: bool,
    pub encoding: Option<String>,
    pub lc_collate: Option<String>,
    pub lc_ctype: Option<String>,
    pub locale: Option<String>,
    pub icu_rules: Option<String>,
    pub version: Option<String>,
    pub comment: Option<String>,
}

/// One installed PostgreSQL extension and its owned catalog objects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Extension {
    /// Cluster-wide extension name; extension identities are not schema-qualified.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Schema containing most or all exported objects.
    pub namespace: String,
    /// Whether the extension can be relocated to another schema.
    pub relocatable: bool,
    /// Installed extension version.
    pub version: String,
    /// Tables/sequences whose user data is preserved by extension-aware dumps.
    pub configuration: Vec<ExtensionConfiguration>,
    /// Objects owned internally by the extension in stable external-address order.
    pub members: Vec<ExtensionMember>,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// One extension configuration relation and its optional dump filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ExtensionConfiguration {
    /// Qualified configuration table or sequence identity.
    pub relation: String,
    /// Optional `WHERE` condition selecting user-owned rows for dumps.
    pub condition: Option<String>,
}

/// Stable server-independent address of an extension-owned object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ExtensionMember {
    /// PostgreSQL object-address type accepted by `pg_get_object_address`.
    pub object_type: String,
    /// Hierarchical object-name components.
    pub names: Vec<String>,
    /// Additional identity arguments, such as a routine signature.
    pub arguments: Vec<String>,
}

/// Provider responsible for PostgreSQL collation behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CollationProvider {
    DatabaseDefault,
    Builtin,
    Libc,
    Icu,
}

/// One PostgreSQL logical-replication publication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Publication {
    pub name: String,
    pub owner: String,
    pub all_tables: bool,
    pub publish_insert: bool,
    pub publish_update: bool,
    pub publish_delete: bool,
    pub publish_truncate: bool,
    pub publish_via_partition_root: bool,
    pub generated_columns: PublicationGeneratedColumns,
    pub schemas: Vec<String>,
    pub tables: Vec<PublicationTable>,
    pub comment: Option<String>,
}

/// Generated-column behavior for a PostgreSQL publication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationGeneratedColumns {
    None,
    Stored,
}

/// One explicitly published table and its optional projection/filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct PublicationTable {
    pub namespace: String,
    pub name: String,
    pub columns: Option<Vec<String>>,
    pub row_filter: Option<String>,
}

/// One PostgreSQL logical-replication subscription.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Subscription {
    /// Database-local subscription name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Whether replication workers may run.
    pub enabled: bool,
    /// Whether the publisher sends binary values when supported.
    pub binary: bool,
    /// In-progress transaction streaming behavior.
    pub streaming: SubscriptionStreaming,
    /// Current two-phase commit state.
    pub two_phase: SubscriptionTwoPhase,
    /// Whether worker errors automatically disable the subscription.
    pub disable_on_error: bool,
    /// Whether publisher connections require password authentication.
    pub password_required: bool,
    /// Whether apply workers operate as the subscription owner.
    pub run_as_owner: bool,
    /// Whether publisher slots are eligible for standby synchronization.
    pub failover: bool,
    /// Upstream replication slot, or none for a detached subscription.
    pub slot_name: Option<String>,
    /// Apply-worker transaction durability policy.
    pub synchronous_commit: SynchronousCommit,
    /// Upstream publication names in declared order.
    pub publications: Vec<String>,
    /// Upstream replication-origin filter.
    pub origin: SubscriptionOrigin,
    /// Pending transaction finish LSN selected by `ALTER SUBSCRIPTION ... SKIP`.
    pub skip_lsn: Option<String>,
    /// Always true: connection info is deliberately discarded during acquisition.
    pub connection_redacted: bool,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// Transaction durability policy used by subscription workers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SynchronousCommit {
    Off,
    Local,
    RemoteWrite,
    On,
    RemoteApply,
}

/// Which upstream replication origins a subscription accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SubscriptionOrigin {
    None,
    Any,
}

/// Transaction streaming mode for a subscription apply worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SubscriptionStreaming {
    /// Do not stream in-progress transactions.
    Off,
    /// Spill streamed changes and apply them after commit.
    On,
    /// Apply streamed changes with a parallel worker when available.
    Parallel,
}

/// Two-phase commit state requested/attained by a subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SubscriptionTwoPhase {
    /// Two-phase replication is disabled.
    Disabled,
    /// Two-phase replication is awaiting initial synchronization.
    Pending,
    /// Two-phase replication is enabled.
    Enabled,
}

/// One user-created PostgreSQL cluster role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Role {
    /// Cluster-wide role name.
    pub name: String,
    /// Whether the role bypasses all permission checks.
    pub superuser: bool,
    /// Whether granted privileges are inherited automatically.
    pub inherit: bool,
    /// Whether the role may create or administer roles.
    pub create_role: bool,
    /// Whether the role may create databases.
    pub create_database: bool,
    /// Whether the role may initiate a login session.
    pub login: bool,
    /// Whether the role may initiate replication connections.
    pub replication: bool,
    /// Whether the role bypasses row-level security.
    pub bypass_row_level_security: bool,
    /// Maximum concurrent connections, or `-1` for no limit.
    pub connection_limit: i32,
    /// Password expiration timestamp in server-normalized text form.
    pub valid_until: Option<String>,
    /// Whether authentication material exists; its value is never acquired.
    pub password_configured: bool,
    /// Role-wide session defaults in deterministic name order.
    pub configuration: Vec<String>,
    /// Roles granted directly to this member.
    pub memberships: Vec<RoleMembership>,
    /// Optional shared catalog comment.
    pub comment: Option<String>,
}

/// One direct PostgreSQL role-membership grant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct RoleMembership {
    /// Granted role.
    pub role: String,
    /// Role that issued the grant.
    pub grantor: String,
    /// Whether the member may administer this membership.
    pub admin: bool,
    /// Whether the member inherits privileges automatically.
    pub inherit: bool,
    /// Whether the member may switch to the granted role.
    pub set: bool,
}

/// Session defaults scoped to one role in one database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct RoleDatabaseSetting {
    /// Database receiving the scoped defaults.
    pub database: String,
    /// Role receiving the scoped defaults.
    pub role: String,
    /// Settings in deterministic name order.
    pub settings: Vec<String>,
}

/// One explicit PostgreSQL object or column privilege.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ObjectPrivilege {
    /// Semantic family of the granted object.
    pub object_kind: PrivilegeObjectKind,
    /// Server-formatted stable object identity.
    pub object_identity: String,
    /// Role that issued the grant.
    pub grantor: String,
    /// Receiving role, or `PUBLIC`.
    pub grantee: String,
    /// Semantic privilege granted on the object.
    pub privilege: PrivilegeKind,
    /// Whether the grantee may grant this privilege onward.
    pub grantable: bool,
}

/// PostgreSQL object families that accept access privileges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PrivilegeObjectKind {
    /// A database.
    Database,
    /// A schema.
    Schema,
    /// A table or partitioned table.
    Table,
    /// One table-like relation column.
    TableColumn,
    /// A sequence.
    Sequence,
    /// An ordinary view.
    View,
    /// A materialized view.
    MaterializedView,
    /// A foreign table.
    ForeignTable,
    /// A function overload.
    Function,
    /// A procedure overload.
    Procedure,
    /// An aggregate overload.
    Aggregate,
    /// A data type.
    Type,
    /// A domain.
    Domain,
    /// A procedural language.
    Language,
    /// A large object.
    LargeObject,
    /// A foreign-data wrapper.
    ForeignDataWrapper,
    /// A foreign server.
    ForeignServer,
    /// A server configuration parameter.
    Parameter,
    /// A tablespace.
    Tablespace,
}

/// PostgreSQL access privilege expressed independently of ACL letter codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PrivilegeKind {
    /// Read rows, values, or large-object pages.
    Select,
    /// Insert rows.
    Insert,
    /// Update rows, columns, or large-object pages.
    Update,
    /// Delete rows.
    Delete,
    /// Truncate a relation.
    Truncate,
    /// Create foreign-key references.
    References,
    /// Create triggers.
    Trigger,
    /// Run table maintenance commands.
    Maintain,
    /// Use a schema, sequence, type, language, wrapper, or server.
    Usage,
    /// Create objects in a database, schema, or tablespace.
    Create,
    /// Connect to a database.
    Connect,
    /// Create temporary relations in a database.
    Temporary,
    /// Execute a routine.
    Execute,
    /// Set a configuration parameter for the current session.
    Set,
    /// Change a configuration parameter globally.
    AlterSystem,
}

/// One default privilege applied to subsequently created objects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct DefaultPrivilege {
    /// Role whose future objects receive the default grant.
    pub owner: String,
    /// Schema scope, or none for a database-wide default.
    pub namespace: Option<String>,
    /// Family of future objects governed by the default.
    pub object_kind: DefaultPrivilegeObject,
    /// Role that issued the grant.
    pub grantor: String,
    /// Receiving role, or `PUBLIC`.
    pub grantee: String,
    /// Semantic privilege applied by default.
    pub privilege: PrivilegeKind,
    /// Whether the grantee may grant this privilege onward.
    pub grantable: bool,
}

/// Object family governed by an `ALTER DEFAULT PRIVILEGES` entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DefaultPrivilegeObject {
    /// Future tables, views, materialized views, and foreign tables.
    Tables,
    /// Future sequences.
    Sequences,
    /// Future functions and procedures.
    Routines,
    /// Future types and domains.
    Types,
    /// Future schemas.
    Schemas,
    /// Future large objects.
    LargeObjects,
}

/// One security-provider label attached to a PostgreSQL catalog object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct SecurityLabel {
    /// Semantic family of the labeled object.
    pub object_kind: SecurityLabelObjectKind,
    /// Server-formatted stable object identity.
    pub object_identity: String,
    /// Label provider registered with PostgreSQL.
    pub provider: String,
    /// Provider-owned label value.
    pub label: String,
}

/// PostgreSQL object families accepted by `SECURITY LABEL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SecurityLabelObjectKind {
    /// An aggregate overload.
    Aggregate,
    /// A database.
    Database,
    /// A domain.
    Domain,
    /// An event trigger.
    EventTrigger,
    /// A foreign table.
    ForeignTable,
    /// A function overload.
    Function,
    /// A large object.
    LargeObject,
    /// A materialized view.
    MaterializedView,
    /// A procedure overload.
    Procedure,
    /// A logical-replication publication.
    Publication,
    /// A cluster role.
    Role,
    /// A schema.
    Schema,
    /// A sequence.
    Sequence,
    /// A logical-replication subscription.
    Subscription,
    /// A table or partitioned table.
    Table,
    /// One table-like relation column.
    TableColumn,
    /// A tablespace.
    Tablespace,
    /// A data type.
    Type,
    /// An ordinary view.
    View,
}

/// Metadata for one PostgreSQL large object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct LargeObject {
    /// Database-local large-object identifier.
    pub oid: u32,
    /// Owning role.
    pub owner: String,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Always true: data pages are deliberately never acquired.
    pub contents_omitted: bool,
}

/// One PostgreSQL enum type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct EnumType {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified type name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Labels in enum sort order.
    pub values: Vec<String>,
}

/// One standalone PostgreSQL composite type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct CompositeType {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified type name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Attributes in ordinal order.
    pub attributes: Vec<CompositeAttribute>,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Reconstructable definition of the current composite type.
    pub definition: String,
}

/// One attribute declared by a PostgreSQL composite type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct CompositeAttribute {
    /// Attribute name.
    pub name: String,
    /// Server-formatted attribute type including any type modifier.
    pub data_type: String,
    /// Effective qualified collation for collatable attributes.
    pub collation: Option<String>,
    /// Optional attribute comment.
    pub comment: Option<String>,
}

/// One PostgreSQL domain type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Domain {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified domain name.
    pub name: String,
    /// Server-formatted underlying type including any type modifier.
    pub base_type: String,
    /// Effective qualified collation for collatable domains.
    pub collation: Option<String>,
    /// Server-formatted domain default expression.
    pub default: Option<String>,
    /// Whether the domain rejects null values.
    pub not_null: bool,
    /// Owning role.
    pub owner: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Check constraints in deterministic name order.
    pub constraints: Vec<DomainConstraint>,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Reconstructable definition of the current domain state.
    pub definition: String,
}

/// One named PostgreSQL domain check constraint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct DomainConstraint {
    /// Constraint name.
    pub name: String,
    /// Server-normalized constraint definition.
    pub definition: String,
    /// Whether existing values have been validated.
    pub validated: bool,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// One user-defined PostgreSQL base type or unresolved shell type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct BaseType {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified type name.
    pub name: String,
    /// Owning role when PostgreSQL exposes one for the catalog entry.
    pub owner: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Whether this is a complete base type rather than a shell placeholder.
    pub defined: bool,
    /// Base-type implementation properties. Shell entries have no reliable details.
    pub details: Option<BaseTypeDetails>,
    /// Automatically associated array type, when defined.
    pub array_type: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Reconstructable definition of the current type state.
    pub definition: String,
}

/// Implementation contract for a defined PostgreSQL base type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct BaseTypeDetails {
    /// Fixed byte length, or a negative PostgreSQL variable-width sentinel.
    pub internal_length: i16,
    /// Whether values use PostgreSQL's pass-by-value ABI.
    pub passed_by_value: bool,
    /// Parser category code.
    pub category: String,
    /// Whether this is the preferred implicit-cast target in its category.
    pub preferred: bool,
    /// Array input delimiter.
    pub delimiter: String,
    /// Required text input function identity.
    pub input: String,
    /// Required text output function identity.
    pub output: String,
    /// Optional binary input function identity.
    pub receive: Option<String>,
    /// Optional binary output function identity.
    pub send: Option<String>,
    /// Optional type-modifier input function identity.
    pub type_modifier_input: Option<String>,
    /// Optional type-modifier output function identity.
    pub type_modifier_output: Option<String>,
    /// Optional custom statistics-analysis function identity.
    pub analyze: Option<String>,
    /// Optional subscripting-handler function identity.
    pub subscript: Option<String>,
    /// Element type returned by subscripting, when declared.
    pub element_type: Option<String>,
    /// Storage alignment contract.
    pub alignment: TypeAlignment,
    /// TOAST/storage strategy.
    pub storage: TypeStorage,
    /// Whether the type supports collation.
    pub collatable: bool,
    /// External-form default value, when configured.
    pub default: Option<String>,
}

/// PostgreSQL type storage alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TypeAlignment {
    Char,
    Short,
    Int,
    Double,
}

/// PostgreSQL base-type TOAST/storage strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TypeStorage {
    Plain,
    External,
    Main,
    Extended,
}

/// One PostgreSQL range type and its automatically paired multirange type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct RangeType {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified range type name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Server-formatted range subtype.
    pub subtype: String,
    /// Qualified B-tree operator class used for subtype ordering.
    pub subtype_operator_class: String,
    /// Effective qualified collation for collatable subtypes.
    pub collation: Option<String>,
    /// Optional canonicalization function identity.
    pub canonical: Option<String>,
    /// Optional subtype-difference function identity.
    pub subtype_diff: Option<String>,
    /// Automatically paired multirange type.
    pub multirange: MultirangeType,
    /// Optional range-type comment.
    pub comment: Option<String>,
    /// Reconstructable definition of the current range and multirange pair.
    pub definition: String,
}

/// PostgreSQL multirange type paired with a range type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct MultirangeType {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified multirange type name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Optional catalog comment independent from the range comment.
    pub comment: Option<String>,
}

/// One PostgreSQL sequence generator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Sequence {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified sequence name.
    pub name: String,
    /// Owning role.
    pub owner: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Server-formatted integer data type.
    pub data_type: String,
    /// Initial sequence value.
    pub start: i64,
    /// Minimum permitted value.
    pub minimum: i64,
    /// Maximum permitted value.
    pub maximum: i64,
    /// Signed step between values.
    pub increment: i64,
    /// Number of values preallocated by a session.
    pub cache: i64,
    /// Whether values wrap at the configured bound.
    pub cycle: bool,
    /// Relation persistence mode.
    pub persistence: SequencePersistence,
    /// Qualified owning column when the sequence is attached to one.
    pub owned_by: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Server-normalized reconstruction of the current sequence definition.
    pub definition: String,
}

/// Persistence mode of a PostgreSQL sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SequencePersistence {
    /// Ordinary persistent sequence.
    Permanent,
    /// Unlogged sequence whose state is not WAL-replicated.
    Unlogged,
}

/// One PostgreSQL table-like relation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Table {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified relation name.
    pub name: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
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
    /// Owning role.
    pub owner: String,
    /// WAL persistence behavior.
    pub persistence: RelationPersistence,
    /// Effective table access method when the relation has or selects one.
    pub access_method: Option<String>,
    /// Underlying composite type for a typed table.
    pub typed_table: Option<String>,
    /// Logical-replication row identity contract.
    pub replica_identity: ReplicaIdentity,
    /// Access-method storage options in server order.
    pub options: Vec<String>,
    /// Foreign-table connection contract.
    pub foreign: Option<ForeignTable>,
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
    /// Effective qualified collation for collatable columns.
    pub collation: Option<String>,
    /// Effective nullability when known.
    pub nullable: Option<bool>,
    /// Server-formatted default expression.
    pub default: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Enum labels when the column type is an enum.
    pub enum_values: Vec<String>,
    /// Identity generation mode.
    pub identity: Option<IdentityGeneration>,
    /// Generated-column expression and storage behavior.
    pub generated: Option<GeneratedColumn>,
    /// Effective TOAST storage strategy.
    pub storage: ColumnStorage,
    /// Explicit compression method, or none for the type/default method.
    pub compression: Option<ColumnCompression>,
    /// Per-column planner statistics target; `-1` means the database default.
    pub statistics_target: i32,
    /// Type-specific column options.
    pub options: Vec<String>,
    /// Foreign-data-wrapper column options.
    pub foreign_options: Vec<String>,
    /// Whether the column has a declaration local to this relation.
    pub locally_defined: bool,
    /// Number of direct inheritance ancestors contributing this column.
    pub inheritance_count: i32,
}

/// Override behavior for a PostgreSQL identity column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IdentityGeneration {
    Always,
    ByDefault,
}

/// Effective PostgreSQL column storage strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ColumnStorage {
    Plain,
    External,
    Main,
    Extended,
}

/// Explicit TOAST compression method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ColumnCompression {
    Pglz,
    Lz4,
}

/// Persistent relation WAL behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RelationPersistence {
    Permanent,
    Unlogged,
}

/// Logical-replication row identity selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ReplicaIdentity {
    Default,
    Nothing,
    Full,
    Index,
}

/// Foreign-data wrapper linkage and per-table options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ForeignTable {
    pub server: String,
    pub wrapper: String,
    pub options: Vec<String>,
}

/// One PostgreSQL generated-column declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct GeneratedColumn {
    /// Server-formatted generation expression.
    pub expression: String,
    /// Whether PostgreSQL computes the value on read or stores it on write.
    pub kind: GeneratedColumnKind,
}

/// PostgreSQL generated-column storage behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GeneratedColumnKind {
    /// Compute the generated value when the column is read.
    Virtual,
    /// Compute and store the generated value when the row is written.
    Stored,
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
    /// Exclusion operators in constrained-column order.
    pub exclusion_operators: Vec<String>,
    /// Whether enforcement may be deferred.
    pub deferrable: bool,
    /// Whether enforcement starts deferred.
    pub initially_deferred: bool,
    /// Whether PostgreSQL enforces the constraint for new or changed rows.
    pub enforced: bool,
    /// Whether the constraint has been validated.
    pub validated: bool,
    /// Whether the constraint is locally defined.
    pub locally_defined: bool,
    /// Whether the constraint excludes inheritance.
    pub no_inherit: bool,
    /// Whether the constraint uses temporal key semantics.
    pub temporal: bool,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// PostgreSQL constraint category.
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
    /// Named or system-generated not-null constraint.
    NotNull,
    /// Exclusion constraint.
    Exclusion,
}

/// One PostgreSQL index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Index {
    /// Index name.
    pub name: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
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
    /// Owning table role (indexes do not have an independent owner).
    pub owner: String,
    /// Explicit index tablespace.
    pub tablespace: Option<String>,
    /// Access-method storage parameters in server order.
    pub options: Vec<String>,
    /// Whether this is a partitioned index relation.
    pub partitioned: bool,
    /// Qualified parent index for an attached index partition.
    pub parent_index: Option<String>,
    /// Constraint whose physical implementation is this index.
    pub constraint: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// One ordered PostgreSQL index key term.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct IndexTerm {
    /// Column or stored expression selected by the term.
    pub target: IndexTarget,
    /// Effective collation.
    pub collation: Option<String>,
    /// Operator class.
    pub operator_class: Option<String>,
    /// Operator-class parameters in server order.
    pub operator_class_parameters: Vec<String>,
    /// Effective sort direction.
    pub order: IndexSortOrder,
    /// Effective null placement.
    pub nulls_order: Option<IndexNullsOrder>,
}

/// Effective placement of null values in a PostgreSQL index term.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IndexNullsOrder {
    /// Null values sort first.
    First,
    /// Null values sort last.
    Last,
}

/// Value selected by a PostgreSQL index key term.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IndexTarget {
    /// A named column.
    Column(String),
    /// A stored index expression.
    Expression(String),
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
    /// Optional catalog comment.
    pub comment: Option<String>,
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
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Server-normalized query definition.
    pub definition: String,
    /// Whether this is a materialized view.
    pub materialized: bool,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// View columns in ordinal order.
    pub columns: Vec<Column>,
    /// Indexes attached to a materialized view, in name order.
    pub indexes: Vec<Index>,
    /// Owning role.
    pub owner: String,
    /// WAL persistence behavior.
    pub persistence: RelationPersistence,
    /// Materialized-view access method.
    pub access_method: Option<String>,
    /// Explicit tablespace for a materialized view.
    pub tablespace: Option<String>,
    /// Relation options in server order.
    pub options: Vec<String>,
    /// Whether a materialized view currently contains populated data.
    pub populated: bool,
    /// Whether an ordinary view acts as a security barrier.
    pub security_barrier: bool,
    /// Whether permissions are checked as the invoking user.
    pub security_invoker: bool,
    /// Automatically applied check option for an updatable view.
    pub check_option: Option<ViewCheckOption>,
}

/// PostgreSQL view check-option mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ViewCheckOption {
    Local,
    Cascaded,
}

/// One PostgreSQL trigger, including partition clones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Trigger {
    /// Target schema, which owns the trigger identity.
    pub namespace: String,
    /// Trigger name, unique per target relation.
    pub name: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
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
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Identity-argument signature including parentheses.
    pub signature: String,
    /// Complete server-formatted argument declarations, including modes and defaults.
    pub arguments: String,
    /// Server-normalized definition when available.
    pub definition: Option<String>,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Server-formatted return type.
    pub return_type: String,
    /// Owning role.
    pub owner: String,
    /// Ordinary or window-function behavior.
    pub kind: FunctionKind,
    /// Implementation language.
    pub language: String,
    /// Volatility contract.
    pub volatility: FunctionVolatility,
    /// Parallel-safety contract.
    pub parallel: FunctionParallel,
    /// Whether execution uses definer rather than invoker privileges.
    pub security_definer: bool,
    /// Whether null input bypasses execution and yields null.
    pub strict: bool,
    /// Whether the function is safe to evaluate before security-barrier predicates.
    pub leakproof: bool,
    /// Whether the function returns a set.
    pub returns_set: bool,
    /// Planner support function identity when configured.
    pub support_function: Option<String>,
    /// Planner execution-cost estimate in server canonical text form.
    pub cost: String,
    /// Estimated result rows for a set-returning function.
    pub rows: Option<String>,
    /// Routine-local configuration settings in deterministic catalog order.
    pub configuration: Vec<String>,
    /// Types using language-specific transforms in declaration order.
    pub transforms: Vec<String>,
}

/// PostgreSQL function execution family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FunctionKind {
    Ordinary,
    Window,
}

/// One PostgreSQL procedure overload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Procedure {
    /// Owning schema.
    pub namespace: String,
    /// Unqualified procedure name.
    pub name: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    /// Identity-argument signature including parentheses.
    pub signature: String,
    /// Complete server-formatted argument declarations, including modes/defaults.
    pub arguments: String,
    /// Server-normalized current definition.
    pub definition: String,
    /// Optional catalog comment.
    pub comment: Option<String>,
    /// Owning role.
    pub owner: String,
    /// Implementation language.
    pub language: String,
    /// Whether execution uses definer rather than invoker privileges.
    pub security_definer: bool,
    /// Routine-local configuration settings in deterministic catalog order.
    pub configuration: Vec<String>,
    /// Types using language-specific transforms in declaration order.
    pub transforms: Vec<String>,
}

/// One PostgreSQL aggregate overload and its transition machinery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Aggregate {
    pub namespace: String,
    pub name: String,
    /// Owning extension when this is an extension support object.
    pub extension: Option<String>,
    pub signature: String,
    pub arguments: String,
    pub owner: String,
    pub result_type: String,
    pub kind: AggregateKind,
    pub direct_arguments: i16,
    pub transition_function: String,
    pub final_function: Option<String>,
    pub combine_function: Option<String>,
    pub serialization_function: Option<String>,
    pub deserialization_function: Option<String>,
    pub moving_transition_function: Option<String>,
    pub moving_inverse_function: Option<String>,
    pub moving_final_function: Option<String>,
    pub final_extra_arguments: bool,
    pub moving_final_extra_arguments: bool,
    pub final_modify: AggregateFinalModify,
    pub moving_final_modify: AggregateFinalModify,
    pub sort_operator: Option<String>,
    pub transition_type: String,
    pub transition_space: i32,
    pub moving_transition_type: Option<String>,
    pub moving_transition_space: i32,
    pub initial_condition: Option<String>,
    pub moving_initial_condition: Option<String>,
    pub parallel: FunctionParallel,
    pub comment: Option<String>,
}

/// PostgreSQL aggregate invocation family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AggregateKind {
    Normal,
    OrderedSet,
    HypotheticalSet,
}

/// Whether an aggregate final function may mutate its transition state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AggregateFinalModify {
    ReadOnly,
    Shareable,
    ReadWrite,
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

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use serde_json::{json, to_value, Value};

    use super::*;

    fn assert_serializes_as<T>(value: T, expected: Value)
    where
        T: Serialize,
    {
        assert_eq!(
            to_value(value).expect("semantic catalog value should serialize"),
            expected
        );
    }

    #[test]
    fn cast_context_serialization_is_semantic() {
        assert_serializes_as(CastContext::Explicit, json!("explicit"));
        assert_serializes_as(CastContext::Assignment, json!("assignment"));
        assert_serializes_as(CastContext::Implicit, json!("implicit"));
    }

    #[test]
    fn cast_method_serialization_is_semantic() {
        assert_serializes_as(CastMethod::Function, json!("function"));
        assert_serializes_as(CastMethod::InputOutput, json!("input_output"));
        assert_serializes_as(CastMethod::Binary, json!("binary"));
    }

    #[test]
    fn operator_kind_serialization_is_semantic() {
        assert_serializes_as(OperatorKind::Binary, json!("binary"));
        assert_serializes_as(OperatorKind::Prefix, json!("prefix"));
    }

    #[test]
    fn operator_purpose_serialization_is_semantic() {
        assert_serializes_as(OperatorPurpose::Search, json!("search"));
        assert_serializes_as(OperatorPurpose::Ordering, json!("ordering"));
    }

    #[test]
    fn access_method_kind_serialization_is_semantic() {
        assert_serializes_as(AccessMethodKind::Table, json!("table"));
        assert_serializes_as(AccessMethodKind::Index, json!("index"));
    }

    #[test]
    fn rewrite_rule_event_serialization_is_semantic() {
        assert_serializes_as(RewriteRuleEvent::Select, json!("select"));
        assert_serializes_as(RewriteRuleEvent::Update, json!("update"));
        assert_serializes_as(RewriteRuleEvent::Insert, json!("insert"));
        assert_serializes_as(RewriteRuleEvent::Delete, json!("delete"));
    }

    #[test]
    fn event_trigger_event_serialization_is_semantic() {
        assert_serializes_as(EventTriggerEvent::Login, json!("login"));
        assert_serializes_as(
            EventTriggerEvent::DdlCommandStart,
            json!("ddl_command_start"),
        );
        assert_serializes_as(EventTriggerEvent::DdlCommandEnd, json!("ddl_command_end"));
        assert_serializes_as(EventTriggerEvent::SqlDrop, json!("sql_drop"));
        assert_serializes_as(EventTriggerEvent::TableRewrite, json!("table_rewrite"));
    }

    #[test]
    fn statistics_kind_serialization_is_semantic() {
        assert_serializes_as(StatisticsKind::NdDistinct, json!("nd_distinct"));
        assert_serializes_as(StatisticsKind::Dependencies, json!("dependencies"));
        assert_serializes_as(
            StatisticsKind::MostCommonValues,
            json!("most_common_values"),
        );
        assert_serializes_as(StatisticsKind::Expressions, json!("expressions"));
    }

    #[test]
    fn database_locale_provider_serialization_is_semantic() {
        assert_serializes_as(DatabaseLocaleProvider::Builtin, json!("builtin"));
        assert_serializes_as(DatabaseLocaleProvider::Libc, json!("libc"));
        assert_serializes_as(DatabaseLocaleProvider::Icu, json!("icu"));
    }

    #[test]
    fn collation_provider_serialization_is_semantic() {
        assert_serializes_as(
            CollationProvider::DatabaseDefault,
            json!("database_default"),
        );
        assert_serializes_as(CollationProvider::Builtin, json!("builtin"));
        assert_serializes_as(CollationProvider::Libc, json!("libc"));
        assert_serializes_as(CollationProvider::Icu, json!("icu"));
    }

    #[test]
    fn publication_generated_columns_serialization_is_semantic() {
        assert_serializes_as(PublicationGeneratedColumns::None, json!("none"));
        assert_serializes_as(PublicationGeneratedColumns::Stored, json!("stored"));
    }

    #[test]
    fn synchronous_commit_serialization_is_semantic() {
        assert_serializes_as(SynchronousCommit::Off, json!("off"));
        assert_serializes_as(SynchronousCommit::Local, json!("local"));
        assert_serializes_as(SynchronousCommit::RemoteWrite, json!("remote_write"));
        assert_serializes_as(SynchronousCommit::On, json!("on"));
        assert_serializes_as(SynchronousCommit::RemoteApply, json!("remote_apply"));
    }

    #[test]
    fn subscription_origin_serialization_is_semantic() {
        assert_serializes_as(SubscriptionOrigin::None, json!("none"));
        assert_serializes_as(SubscriptionOrigin::Any, json!("any"));
    }

    #[test]
    fn subscription_streaming_serialization_is_semantic() {
        assert_serializes_as(SubscriptionStreaming::Off, json!("off"));
        assert_serializes_as(SubscriptionStreaming::On, json!("on"));
        assert_serializes_as(SubscriptionStreaming::Parallel, json!("parallel"));
    }

    #[test]
    fn subscription_two_phase_serialization_is_semantic() {
        assert_serializes_as(SubscriptionTwoPhase::Disabled, json!("disabled"));
        assert_serializes_as(SubscriptionTwoPhase::Pending, json!("pending"));
        assert_serializes_as(SubscriptionTwoPhase::Enabled, json!("enabled"));
    }

    #[test]
    fn privilege_object_kind_serialization_is_semantic() {
        let cases = [
            (PrivilegeObjectKind::Database, "database"),
            (PrivilegeObjectKind::Schema, "schema"),
            (PrivilegeObjectKind::Table, "table"),
            (PrivilegeObjectKind::TableColumn, "table_column"),
            (PrivilegeObjectKind::Sequence, "sequence"),
            (PrivilegeObjectKind::View, "view"),
            (PrivilegeObjectKind::MaterializedView, "materialized_view"),
            (PrivilegeObjectKind::ForeignTable, "foreign_table"),
            (PrivilegeObjectKind::Function, "function"),
            (PrivilegeObjectKind::Procedure, "procedure"),
            (PrivilegeObjectKind::Aggregate, "aggregate"),
            (PrivilegeObjectKind::Type, "type"),
            (PrivilegeObjectKind::Domain, "domain"),
            (PrivilegeObjectKind::Language, "language"),
            (PrivilegeObjectKind::LargeObject, "large_object"),
            (
                PrivilegeObjectKind::ForeignDataWrapper,
                "foreign_data_wrapper",
            ),
            (PrivilegeObjectKind::ForeignServer, "foreign_server"),
            (PrivilegeObjectKind::Parameter, "parameter"),
            (PrivilegeObjectKind::Tablespace, "tablespace"),
        ];
        for (value, expected) in cases {
            assert_serializes_as(value, json!(expected));
        }
    }

    #[test]
    fn privilege_kind_serialization_is_semantic() {
        let cases = [
            (PrivilegeKind::Select, "select"),
            (PrivilegeKind::Insert, "insert"),
            (PrivilegeKind::Update, "update"),
            (PrivilegeKind::Delete, "delete"),
            (PrivilegeKind::Truncate, "truncate"),
            (PrivilegeKind::References, "references"),
            (PrivilegeKind::Trigger, "trigger"),
            (PrivilegeKind::Maintain, "maintain"),
            (PrivilegeKind::Usage, "usage"),
            (PrivilegeKind::Create, "create"),
            (PrivilegeKind::Connect, "connect"),
            (PrivilegeKind::Temporary, "temporary"),
            (PrivilegeKind::Execute, "execute"),
            (PrivilegeKind::Set, "set"),
            (PrivilegeKind::AlterSystem, "alter_system"),
        ];
        for (value, expected) in cases {
            assert_serializes_as(value, json!(expected));
        }
    }

    #[test]
    fn default_privilege_object_serialization_is_semantic() {
        let cases = [
            (DefaultPrivilegeObject::Tables, "tables"),
            (DefaultPrivilegeObject::Sequences, "sequences"),
            (DefaultPrivilegeObject::Routines, "routines"),
            (DefaultPrivilegeObject::Types, "types"),
            (DefaultPrivilegeObject::Schemas, "schemas"),
            (DefaultPrivilegeObject::LargeObjects, "large_objects"),
        ];
        for (value, expected) in cases {
            assert_serializes_as(value, json!(expected));
        }
    }

    #[test]
    fn security_label_object_kind_serialization_is_semantic() {
        let cases = [
            (SecurityLabelObjectKind::Aggregate, "aggregate"),
            (SecurityLabelObjectKind::Database, "database"),
            (SecurityLabelObjectKind::Domain, "domain"),
            (SecurityLabelObjectKind::EventTrigger, "event_trigger"),
            (SecurityLabelObjectKind::ForeignTable, "foreign_table"),
            (SecurityLabelObjectKind::Function, "function"),
            (SecurityLabelObjectKind::LargeObject, "large_object"),
            (
                SecurityLabelObjectKind::MaterializedView,
                "materialized_view",
            ),
            (SecurityLabelObjectKind::Procedure, "procedure"),
            (SecurityLabelObjectKind::Publication, "publication"),
            (SecurityLabelObjectKind::Role, "role"),
            (SecurityLabelObjectKind::Schema, "schema"),
            (SecurityLabelObjectKind::Sequence, "sequence"),
            (SecurityLabelObjectKind::Subscription, "subscription"),
            (SecurityLabelObjectKind::Table, "table"),
            (SecurityLabelObjectKind::TableColumn, "table_column"),
            (SecurityLabelObjectKind::Tablespace, "tablespace"),
            (SecurityLabelObjectKind::Type, "type"),
            (SecurityLabelObjectKind::View, "view"),
        ];
        for (value, expected) in cases {
            assert_serializes_as(value, json!(expected));
        }
    }

    #[test]
    fn type_alignment_serialization_is_semantic() {
        assert_serializes_as(TypeAlignment::Char, json!("char"));
        assert_serializes_as(TypeAlignment::Short, json!("short"));
        assert_serializes_as(TypeAlignment::Int, json!("int"));
        assert_serializes_as(TypeAlignment::Double, json!("double"));
    }

    #[test]
    fn type_storage_serialization_is_semantic() {
        assert_serializes_as(TypeStorage::Plain, json!("plain"));
        assert_serializes_as(TypeStorage::External, json!("external"));
        assert_serializes_as(TypeStorage::Main, json!("main"));
        assert_serializes_as(TypeStorage::Extended, json!("extended"));
    }

    #[test]
    fn sequence_persistence_serialization_is_semantic() {
        assert_serializes_as(SequencePersistence::Permanent, json!("permanent"));
        assert_serializes_as(SequencePersistence::Unlogged, json!("unlogged"));
    }

    #[test]
    fn identity_generation_serialization_is_semantic() {
        assert_serializes_as(IdentityGeneration::Always, json!("always"));
        assert_serializes_as(IdentityGeneration::ByDefault, json!("by_default"));
    }

    #[test]
    fn column_storage_serialization_is_semantic() {
        assert_serializes_as(ColumnStorage::Plain, json!("plain"));
        assert_serializes_as(ColumnStorage::External, json!("external"));
        assert_serializes_as(ColumnStorage::Main, json!("main"));
        assert_serializes_as(ColumnStorage::Extended, json!("extended"));
    }

    #[test]
    fn column_compression_serialization_is_semantic() {
        assert_serializes_as(ColumnCompression::Pglz, json!("pglz"));
        assert_serializes_as(ColumnCompression::Lz4, json!("lz4"));
    }

    #[test]
    fn relation_persistence_serialization_is_semantic() {
        assert_serializes_as(RelationPersistence::Permanent, json!("permanent"));
        assert_serializes_as(RelationPersistence::Unlogged, json!("unlogged"));
    }

    #[test]
    fn replica_identity_serialization_is_semantic() {
        assert_serializes_as(ReplicaIdentity::Default, json!("default"));
        assert_serializes_as(ReplicaIdentity::Nothing, json!("nothing"));
        assert_serializes_as(ReplicaIdentity::Full, json!("full"));
        assert_serializes_as(ReplicaIdentity::Index, json!("index"));
    }

    #[test]
    fn generated_column_kind_serialization_is_semantic() {
        assert_serializes_as(GeneratedColumnKind::Virtual, json!("virtual"));
        assert_serializes_as(GeneratedColumnKind::Stored, json!("stored"));
    }

    #[test]
    fn constraint_kind_serialization_is_semantic() {
        let cases = [
            (ConstraintKind::PrimaryKey, "primary_key"),
            (ConstraintKind::ForeignKey, "foreign_key"),
            (ConstraintKind::Unique, "unique"),
            (ConstraintKind::Check, "check"),
            (ConstraintKind::NotNull, "not_null"),
            (ConstraintKind::Exclusion, "exclusion"),
        ];
        for (value, expected) in cases {
            assert_serializes_as(value, json!(expected));
        }
    }

    #[test]
    fn index_nulls_order_serialization_is_semantic() {
        assert_serializes_as(IndexNullsOrder::First, json!("first"));
        assert_serializes_as(IndexNullsOrder::Last, json!("last"));
    }

    #[test]
    fn index_target_serialization_preserves_the_semantic_payload() {
        assert_serializes_as(IndexTarget::Column("id".into()), json!({"column": "id"}));
        assert_serializes_as(
            IndexTarget::Expression("lower(name)".into()),
            json!({"expression": "lower(name)"}),
        );
    }

    #[test]
    fn table_kind_serialization_is_semantic() {
        assert_serializes_as(TableKind::Table, json!("table"));
        assert_serializes_as(TableKind::PartitionedTable, json!("partitioned_table"));
        assert_serializes_as(TableKind::Partition, json!("partition"));
        assert_serializes_as(TableKind::ForeignTable, json!("foreign_table"));
    }

    #[test]
    fn policy_command_serialization_is_semantic() {
        assert_serializes_as(PolicyCommand::All, json!("all"));
        assert_serializes_as(PolicyCommand::Select, json!("select"));
        assert_serializes_as(PolicyCommand::Insert, json!("insert"));
        assert_serializes_as(PolicyCommand::Update, json!("update"));
        assert_serializes_as(PolicyCommand::Delete, json!("delete"));
    }

    #[test]
    fn view_check_option_serialization_is_semantic() {
        assert_serializes_as(ViewCheckOption::Local, json!("local"));
        assert_serializes_as(ViewCheckOption::Cascaded, json!("cascaded"));
    }

    #[test]
    fn trigger_timing_serialization_is_semantic() {
        assert_serializes_as(TriggerTiming::Before, json!("before"));
        assert_serializes_as(TriggerTiming::After, json!("after"));
        assert_serializes_as(TriggerTiming::InsteadOf, json!("instead_of"));
    }

    #[test]
    fn trigger_orientation_serialization_is_semantic() {
        assert_serializes_as(TriggerOrientation::Row, json!("row"));
        assert_serializes_as(TriggerOrientation::Statement, json!("statement"));
    }

    #[test]
    fn trigger_event_serialization_preserves_semantic_payloads() {
        assert_serializes_as(TriggerEvent::Delete, json!({"event": "delete"}));
        assert_serializes_as(TriggerEvent::Insert, json!({"event": "insert"}));
        assert_serializes_as(
            TriggerEvent::Update {
                columns: vec!["name".into()],
            },
            json!({"event": "update", "columns": ["name"]}),
        );
        assert_serializes_as(TriggerEvent::Truncate, json!({"event": "truncate"}));
    }

    #[test]
    fn trigger_enabled_serialization_is_semantic() {
        assert_serializes_as(TriggerEnabled::Origin, json!("origin"));
        assert_serializes_as(TriggerEnabled::Disabled, json!("disabled"));
        assert_serializes_as(TriggerEnabled::Replica, json!("replica"));
        assert_serializes_as(TriggerEnabled::Always, json!("always"));
    }

    #[test]
    fn function_kind_serialization_is_semantic() {
        assert_serializes_as(FunctionKind::Ordinary, json!("ordinary"));
        assert_serializes_as(FunctionKind::Window, json!("window"));
    }

    #[test]
    fn aggregate_kind_serialization_is_semantic() {
        assert_serializes_as(AggregateKind::Normal, json!("normal"));
        assert_serializes_as(AggregateKind::OrderedSet, json!("ordered_set"));
        assert_serializes_as(AggregateKind::HypotheticalSet, json!("hypothetical_set"));
    }

    #[test]
    fn aggregate_final_modify_serialization_is_semantic() {
        assert_serializes_as(AggregateFinalModify::ReadOnly, json!("read_only"));
        assert_serializes_as(AggregateFinalModify::Shareable, json!("shareable"));
        assert_serializes_as(AggregateFinalModify::ReadWrite, json!("read_write"));
    }

    #[test]
    fn function_volatility_serialization_is_semantic() {
        assert_serializes_as(FunctionVolatility::Immutable, json!("immutable"));
        assert_serializes_as(FunctionVolatility::Stable, json!("stable"));
        assert_serializes_as(FunctionVolatility::Volatile, json!("volatile"));
    }

    #[test]
    fn function_parallel_serialization_is_semantic() {
        assert_serializes_as(FunctionParallel::Safe, json!("safe"));
        assert_serializes_as(FunctionParallel::Restricted, json!("restricted"));
        assert_serializes_as(FunctionParallel::Unsafe, json!("unsafe"));
    }
}
