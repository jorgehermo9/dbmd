//! SQLite backend module.

mod catalog;
mod config;
mod introspect;
pub(crate) mod render;

pub use catalog::{
    Catalog, Column, ColumnKind, ConflictResolution, Constraint, Index, IndexOrigin, Snapshot,
    Table, TableKind, Trigger, TriggerEvent, TriggerTiming, View,
};
pub use config::{Config, ConfigResolveError};
pub use introspect::{introspect, IntrospectionError, SqliteSource, SqliteSourceError};
