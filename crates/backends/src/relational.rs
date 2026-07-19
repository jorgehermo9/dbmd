//! Relational value types reused only where backend semantics are equivalent.

use serde::Serialize;

/// A backend-defined namespace that can qualify schema objects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Namespace {
    /// Namespace name.
    pub name: String,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// Constraint category with equivalent relational meaning across supported backends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
    /// Not-null requirement.
    NotNull,
    /// PostgreSQL-style exclusion constraint.
    Exclusion,
}

/// A relational foreign-key target and referential behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ForeignKeyReference {
    /// Target namespace.
    pub namespace: String,
    /// Target table.
    pub table: String,
    /// Target columns in referenced order.
    pub columns: Vec<String>,
    /// Action when referenced keys update.
    pub on_update: ForeignKeyAction,
    /// Action when referenced rows delete.
    pub on_delete: ForeignKeyAction,
    /// Optional match-mode name.
    pub match_name: Option<String>,
    /// Deferral behavior.
    pub deferrability: ForeignKeyDeferrability,
}

/// Whether and when a foreign key may be deferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ForeignKeyDeferrability {
    /// Whether enforcement may be deferred.
    pub deferrable: bool,
    /// Initial enforcement timing.
    pub initially: ForeignKeyInitialTiming,
}

impl Default for ForeignKeyDeferrability {
    fn default() -> Self {
        Self {
            deferrable: false,
            initially: ForeignKeyInitialTiming::Immediate,
        }
    }
}

/// Initial enforcement timing of a foreign key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ForeignKeyInitialTiming {
    /// Enforce immediately.
    Immediate,
    /// Start deferred until transaction end.
    Deferred,
}

/// Behavior applied to child rows when a referenced key changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ForeignKeyAction {
    /// Perform no automatic action.
    NoAction,
    /// Reject the referenced change.
    Restrict,
    /// Set referencing columns to null.
    SetNull,
    /// Set referencing columns to defaults.
    SetDefault,
    /// Cascade the referenced change.
    Cascade,
}

/// One ordered key term of a relational index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct IndexTerm {
    /// Column, expression, or row identifier selected by the term.
    pub target: IndexTarget,
    /// Effective collation.
    pub collation: Option<String>,
    /// Backend operator class.
    pub operator_class: Option<String>,
    /// Effective sort direction.
    pub order: IndexSortOrder,
    /// Effective null placement when meaningful.
    pub nulls_order: Option<IndexNullsOrder>,
}

/// Effective placement of null values in an ordered index term.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IndexNullsOrder {
    /// Null values sort first.
    First,
    /// Null values sort last.
    Last,
}

/// Value selected by an index key term.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IndexTarget {
    /// A named column.
    Column(String),
    /// A stored index expression.
    Expression(String),
    /// SQLite row identifier.
    RowId,
}

/// Effective ordering of an index key term.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IndexSortOrder {
    /// Ascending order.
    Ascending,
    /// Descending order.
    Descending,
}
