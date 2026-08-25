#![doc = include_str!("../README.md")]

mod catalog;
mod config;
mod introspect;
mod render;

use dbmd_core::SourceId;
use dbmd_render::{RenderSource, TemplateFile};

pub use catalog::{
    AccessMethod, AccessMethodKind, Aggregate, AggregateFinalModify, AggregateKind, BaseType,
    BaseTypeDetails, Cast, CastContext, CastMethod, Catalog, Collation, CollationProvider, Column,
    ColumnCompression, ColumnStorage, CompositeAttribute, CompositeType, Constraint,
    ConstraintKind, ConstraintTrigger, Conversion, Database, DatabaseLocaleProvider,
    DefaultPrivilege, DefaultPrivilegeObject, Domain, DomainConstraint, EnumType, EventTrigger,
    EventTriggerEvent, ExtendedStatistics, Extension, ExtensionConfiguration, ExtensionMember,
    ForeignDataWrapper, ForeignServer, ForeignTable, Function, FunctionKind, FunctionParallel,
    FunctionVolatility, GeneratedColumn, GeneratedColumnKind, IdentityGeneration, Index,
    IndexNullsOrder, IndexTarget, IndexTerm, Language, LargeObject, MultirangeType, Namespace,
    ObjectPrivilege, Operator, OperatorClass, OperatorFamily, OperatorFamilyFunction,
    OperatorFamilyOperator, OperatorKind, OperatorPurpose, Policy, PolicyCommand, PrivilegeKind,
    PrivilegeObjectKind, Procedure, Publication, PublicationGeneratedColumns, PublicationTable,
    RangeType, RelationPersistence, ReplicaIdentity, RewriteRule, RewriteRuleEvent, Role,
    RoleDatabaseSetting, RoleMembership, SecurityLabel, SecurityLabelObjectKind, Sequence,
    SequencePersistence, Snapshot, StatisticsKind, Subscription, SubscriptionOrigin,
    SubscriptionStreaming, SubscriptionTwoPhase, SynchronousCommit, Table, TableKind, Tablespace,
    TextSearchConfiguration, TextSearchDictionary, TextSearchMapping, TextSearchParser,
    TextSearchTemplate, Transform, Trigger, TriggerEnabled, TriggerEvent, TriggerOrientation,
    TriggerTiming, TypeAlignment, TypeStorage, UserMapping, View, ViewCheckOption,
};
pub use config::Config;
pub use introspect::{introspect, IntrospectionError, PostgresSource};

/// Maps a PostgreSQL catalog into backend-owned presentation data.
#[must_use]
pub fn render_source(
    id: &SourceId,
    display_name: Option<&str>,
    catalog: &Catalog,
    nested: bool,
) -> RenderSource {
    render::source(id, display_name, catalog, nested)
}

/// Returns the PostgreSQL templates compiled into this backend.
#[must_use]
pub const fn template_files() -> &'static [TemplateFile] {
    render::TEMPLATES
}
