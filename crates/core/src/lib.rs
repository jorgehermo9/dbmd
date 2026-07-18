//! Pure domain types and invariants for normalized database structure.

mod context;
mod schema;

pub use context::{
    Backend, DatabaseContext, DatabaseContextError, SourceId, SourceIdError, SourceSnapshot,
};
pub use schema::{
    ClickHouseColumn, ClickHouseIndex, ClickHouseTable, Column, ColumnBackend, Constraint,
    ConstraintBackend, ConstraintKind, ForeignKeyAction, ForeignKeyDeferrability,
    ForeignKeyInitialTiming, ForeignKeyReference, Function, Index, IndexBackend, IndexSortOrder,
    IndexTarget, IndexTerm, PostgresColumn, PostgresIndex, PostgresTable, PostgresTableKind,
    SqliteColumn, SqliteColumnKind, SqliteConflictResolution, SqliteConstraint, SqliteIndex,
    SqliteIndexOrigin, SqliteTable, SqliteTableKind, Table, TableBackend, Trigger, TriggerEvent,
    TriggerTiming, View,
};
