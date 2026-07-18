//! Pure domain types and invariants for normalized database structure.

mod context;
mod schema;

pub use context::{
    Backend, DatabaseContext, DatabaseContextError, SourceId, SourceIdError, SourceSnapshot,
};
pub use schema::{
    ClickHouseColumn, ClickHouseIndex, ClickHouseTable, Column, ColumnBackend, Constraint,
    ConstraintBackend, ConstraintKind, EnumType, ForeignKeyAction, ForeignKeyDeferrability,
    ForeignKeyInitialTiming, ForeignKeyReference, Function, FunctionBackend, Index, IndexBackend,
    IndexNullsOrder, IndexSortOrder, IndexTarget, IndexTerm, Namespace, PostgresColumn,
    PostgresConstraint, PostgresFunction, PostgresFunctionParallel, PostgresFunctionVolatility,
    PostgresIndex, PostgresPolicy, PostgresPolicyCommand, PostgresTable, PostgresTableKind,
    SqliteColumn, SqliteColumnKind, SqliteConflictResolution, SqliteConstraint, SqliteIndex,
    SqliteIndexOrigin, SqliteTable, SqliteTableKind, Table, TableBackend, Trigger, TriggerEvent,
    TriggerTiming, View,
};
