#![doc = include_str!("../README.md")]

mod catalog;
mod config;
mod ddl;
mod introspect;
mod render;

use dbmd_core::SourceId;
use dbmd_render::{RenderSource, TemplateFile};

pub use catalog::{
    AccessTarget, Catalog, Column, ColumnDefaultKind, Constraint, ConstraintKind,
    DataSkippingIndex, Database, DictionaryDetails, DictionaryField, Grant, NamedCollection,
    NamedCollectionEntry, Projection, Quota, QuotaLimit, RefreshSchedule, Resource,
    ResourceOperation, Role, RoleGrant, RowPolicy, SettingsProfile, SettingsProfileElement,
    Snapshot, Table, TableKind, TableReference, TableTtl, TtlAction, TtlDestination, User,
    UserDefinedFunction, UserDefinedFunctionOrigin, UserHosts, ViewRefresh, ViewSqlSecurity,
    WindowView, Workload, WorkloadSetting,
};
pub use config::Config;
pub use introspect::{introspect, ClickHouseSource, IntrospectionError};

/// Maps a ClickHouse catalog into backend-owned presentation data.
#[must_use]
pub fn render_source(
    id: &SourceId,
    display_name: Option<&str>,
    catalog: &Catalog,
    nested: bool,
) -> RenderSource {
    render::source(id, display_name, catalog, nested)
}

/// Returns the ClickHouse templates compiled into this backend.
#[must_use]
pub const fn template_files() -> &'static [TemplateFile] {
    render::TEMPLATES
}
