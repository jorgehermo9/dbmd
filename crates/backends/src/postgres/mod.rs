//! PostgreSQL backend module.

mod catalog;
mod config;
mod introspect;
pub(crate) mod render;

pub use catalog::{
    Catalog, Column, Constraint, ConstraintTrigger, EnumType, Function, FunctionParallel,
    FunctionVolatility, Index, Policy, PolicyCommand, Snapshot, Table, TableKind, Trigger,
    TriggerEnabled, TriggerEvent, TriggerOrientation, TriggerTiming, View,
};
pub use config::Config;
pub use introspect::{introspect, IntrospectionError, PostgresSource};
