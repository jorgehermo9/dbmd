use std::{collections::HashSet, fmt, str::FromStr};

use serde::Serialize;
use thiserror::Error;

use crate::{Function, Table, Trigger, View};

/// The stable, validated identifier of a configured database source.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct SourceId(String);

impl SourceId {
    /// Returns the source identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for SourceId {
    type Err = SourceIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() {
            return Err(SourceIdError::Empty);
        }

        if let Some((index, character)) = value.char_indices().find(|(_, character)| {
            !character.is_ascii_alphanumeric() && !matches!(character, '_' | '-')
        }) {
            return Err(SourceIdError::InvalidCharacter { character, index });
        }

        Ok(Self(value.to_string()))
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Why a raw source identifier could not become a [`SourceId`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SourceIdError {
    /// The identifier was empty.
    #[error("source ID cannot be empty")]
    Empty,
    /// The identifier contained a character outside the documented slug grammar.
    #[error("source ID contains invalid character `{character}` at byte {index}")]
    InvalidCharacter {
        /// The rejected character.
        character: char,
        /// The byte offset of the rejected character.
        index: usize,
    },
}

/// A database family whose metadata semantics dbmd understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    /// SQLite.
    Sqlite,
    /// PostgreSQL.
    Postgres,
    /// ClickHouse.
    ClickHouse,
}

/// The normalized, point-in-time structural description of one source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceSnapshot {
    /// Stable source identity used for selection and paths.
    pub id: SourceId,
    /// Optional presentation-only source name.
    pub display_name: Option<String>,
    /// Database family that produced this snapshot.
    pub backend: Backend,
    /// Tables ordered according to core normalization policy.
    pub tables: Vec<Table>,
    /// Views ordered according to core normalization policy.
    pub views: Vec<View>,
    /// Triggers ordered according to core normalization policy.
    pub triggers: Vec<Trigger>,
    /// Functions ordered according to core normalization policy.
    pub functions: Vec<Function>,
}

impl SourceSnapshot {
    /// Creates an empty structural snapshot for a source.
    #[must_use]
    pub fn new(id: SourceId, backend: Backend) -> Self {
        Self {
            id,
            display_name: None,
            backend,
            tables: Vec::new(),
            views: Vec::new(),
            triggers: Vec::new(),
            functions: Vec::new(),
        }
    }
}

/// The ordered source snapshots selected for one application operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DatabaseContext {
    sources: Vec<SourceSnapshot>,
}

impl DatabaseContext {
    /// Creates a database context while preserving the supplied source order.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseContextError::Empty`] when no sources are supplied and
    /// [`DatabaseContextError::DuplicateSourceId`] when an identifier occurs more than once.
    pub fn new(sources: Vec<SourceSnapshot>) -> Result<Self, DatabaseContextError> {
        if sources.is_empty() {
            return Err(DatabaseContextError::Empty);
        }

        let mut seen = HashSet::with_capacity(sources.len());
        for source in &sources {
            if !seen.insert(&source.id) {
                return Err(DatabaseContextError::DuplicateSourceId(source.id.clone()));
            }
        }

        Ok(Self { sources })
    }

    /// Returns selected sources in their resolved operation order.
    #[must_use]
    pub fn sources(&self) -> &[SourceSnapshot] {
        &self.sources
    }
}

/// Why source snapshots could not form a valid [`DatabaseContext`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DatabaseContextError {
    /// The operation selected no sources.
    #[error("database context must contain at least one source")]
    Empty,
    /// The same stable source identifier appeared more than once.
    #[error("database context contains duplicate source ID `{0}`")]
    DuplicateSourceId(SourceId),
}
