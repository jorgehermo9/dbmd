#![doc = include_str!("../README.md")]

/// Shared Markdown-ready presentation values used by relational backend crates.
pub mod presentation;

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

impl Namespace {
    /// Creates a namespace with its optional catalog comment.
    #[must_use]
    pub fn new(name: impl Into<String>, comment: Option<String>) -> Self {
        Self {
            name: name.into(),
            comment,
        }
    }
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

impl ForeignKeyReference {
    /// Creates a foreign-key reference with default relational actions.
    #[must_use]
    pub fn new(
        namespace: impl Into<String>,
        table: impl Into<String>,
        columns: Vec<String>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            table: table.into(),
            columns,
            on_update: ForeignKeyAction::NoAction,
            on_delete: ForeignKeyAction::NoAction,
            match_name: None,
            deferrability: ForeignKeyDeferrability::default(),
        }
    }

    /// Sets update and delete actions.
    #[must_use]
    pub const fn with_actions(
        mut self,
        on_update: ForeignKeyAction,
        on_delete: ForeignKeyAction,
    ) -> Self {
        self.on_update = on_update;
        self.on_delete = on_delete;
        self
    }

    /// Sets the backend match-mode name.
    #[must_use]
    pub fn with_match_name(mut self, match_name: Option<String>) -> Self {
        self.match_name = match_name;
        self
    }

    /// Sets foreign-key deferral behavior.
    #[must_use]
    pub const fn with_deferrability(mut self, deferrability: ForeignKeyDeferrability) -> Self {
        self.deferrability = deferrability;
        self
    }
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

impl ForeignKeyDeferrability {
    /// Creates explicit foreign-key deferral behavior.
    #[must_use]
    pub const fn new(deferrable: bool, initially: ForeignKeyInitialTiming) -> Self {
        Self {
            deferrable,
            initially,
        }
    }
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
    /// Enforce immediately.
    Immediate,
    /// Start deferred until transaction end.
    Deferred,
}

/// Behavior applied to child rows when a referenced key changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
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

/// Effective ordering of an index key term.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexSortOrder {
    /// Ascending order.
    Ascending,
    /// Descending order.
    Descending,
}
