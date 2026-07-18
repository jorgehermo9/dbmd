#![doc = include_str!("../README.md")]

pub mod postgres;
pub mod sqlite;

use dbmd_core::{Backend, SourceId, SourceSnapshot};
use thiserror::Error;

/// One resolved concrete database source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// A resolved SQLite database source.
    Sqlite(sqlite::SqliteSource),
    /// A resolved PostgreSQL database source.
    Postgres(postgres::PostgresSource),
}

impl Source {
    /// Returns the source's stable configured identity.
    #[must_use]
    pub fn id(&self) -> &SourceId {
        match self {
            Self::Sqlite(source) => source.id(),
            Self::Postgres(source) => source.id(),
        }
    }

    /// Returns the database family handled by this source adapter.
    #[must_use]
    pub fn backend(&self) -> Backend {
        match self {
            Self::Sqlite(_) => Backend::Sqlite,
            Self::Postgres(_) => Backend::Postgres,
        }
    }
}

impl From<sqlite::SqliteSource> for Source {
    fn from(source: sqlite::SqliteSource) -> Self {
        Self::Sqlite(source)
    }
}

impl From<postgres::PostgresSource> for Source {
    fn from(source: postgres::PostgresSource) -> Self {
        Self::Postgres(source)
    }
}

/// Introspects one resolved source through its concrete backend adapter.
///
/// # Errors
///
/// Returns a backend-scoped error when the database cannot be read faithfully.
pub fn introspect(source: &Source) -> Result<SourceSnapshot, IntrospectionError> {
    match source {
        Source::Sqlite(source) => sqlite::introspect(source).map_err(IntrospectionError::from),
        Source::Postgres(source) => postgres::introspect(source).map_err(IntrospectionError::from),
    }
}

/// Why a resolved database source could not be introspected.
#[derive(Debug, Error)]
pub enum IntrospectionError {
    /// SQLite connection or catalog introspection failed.
    #[error(transparent)]
    Sqlite(#[from] sqlite::IntrospectionError),
    /// PostgreSQL connection or catalog introspection failed.
    #[error(transparent)]
    Postgres(#[from] postgres::IntrospectionError),
}

impl IntrospectionError {
    /// Returns a credential-free causal diagnostic suitable for operational checks.
    #[must_use]
    pub fn diagnostic(&self) -> String {
        let error: &dyn std::error::Error = match self {
            Self::Sqlite(error) => error,
            Self::Postgres(error) => error,
        };
        let mut message = error.to_string();
        let mut source = error.source();
        while let Some(error) = source {
            message.push_str(": ");
            message.push_str(&error.to_string());
            source = error.source();
        }
        message
    }
}
