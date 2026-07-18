//! Relational value types reused only where backend semantics are equivalent.

use serde::Serialize;

/// A backend-defined namespace that can qualify schema objects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Namespace {
    /// Namespace name.
    pub name: String,
    /// Optional catalog comment.
    pub comment: Option<String>,
}

/// Constraint category with equivalent relational meaning across supported backends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintKind {
    PrimaryKey,
    ForeignKey,
    Unique,
    Check,
    NotNull,
    Exclusion,
}

/// A relational foreign-key target and referential behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ForeignKeyReference {
    pub namespace: String,
    pub table: String,
    pub columns: Vec<String>,
    pub on_update: ForeignKeyAction,
    pub on_delete: ForeignKeyAction,
    pub match_name: Option<String>,
    pub deferrability: ForeignKeyDeferrability,
}

/// Whether and when a foreign key may be deferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ForeignKeyDeferrability {
    pub deferrable: bool,
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
pub enum ForeignKeyInitialTiming {
    Immediate,
    Deferred,
}

/// Behavior applied to child rows when a referenced key changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForeignKeyAction {
    NoAction,
    Restrict,
    SetNull,
    SetDefault,
    Cascade,
}

/// One ordered key term of a relational index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndexTerm {
    pub target: IndexTarget,
    pub collation: Option<String>,
    pub operator_class: Option<String>,
    pub order: IndexSortOrder,
    pub nulls_order: Option<IndexNullsOrder>,
}

/// Effective placement of null values in an ordered index term.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexNullsOrder {
    First,
    Last,
}

/// Value selected by an index key term.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexTarget {
    Column(String),
    Expression(String),
    RowId,
}

/// Effective ordering of an index key term.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexSortOrder {
    Ascending,
    Descending,
}
