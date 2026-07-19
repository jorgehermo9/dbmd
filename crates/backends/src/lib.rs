#![doc = include_str!("../README.md")]

pub mod postgres;
pub mod relational;
mod render_support;
pub mod sqlite;

use dbmd_core::{SourceId, SourceSnapshot};
use dbmd_render::{RenderContext, TemplateFile};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The built-in database families composed into this binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Backend {
    /// SQLite.
    Sqlite,
    /// PostgreSQL.
    Postgres,
}

impl Backend {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
        }
    }
}

/// One resolved concrete database source.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Source {
    /// Resolved SQLite source.
    Sqlite(sqlite::SqliteSource),
    /// Resolved PostgreSQL source.
    Postgres(postgres::PostgresSource),
}

impl Source {
    #[must_use]
    pub fn id(&self) -> &SourceId {
        match self {
            Self::Sqlite(source) => source.id(),
            Self::Postgres(source) => source.id(),
        }
    }

    #[must_use]
    pub const fn backend(&self) -> Backend {
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

/// Backend-owned committed fields for one named source.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "backend", rename_all = "lowercase", deny_unknown_fields)]
#[non_exhaustive]
pub enum SourceConfig {
    /// SQLite-specific committed source fields.
    Sqlite(sqlite::Config),
    /// PostgreSQL-specific committed source fields.
    Postgres(postgres::Config),
}

impl SourceConfig {
    /// Returns raw connection/path values that may contain environment references.
    #[must_use]
    pub fn environment_values(&self) -> Vec<&str> {
        match self {
            Self::Sqlite(config) => config.environment_values(),
            Self::Postgres(config) => config.environment_values(),
        }
    }

    /// Resolves backend fields into a concrete source.
    ///
    /// # Errors
    ///
    /// Returns the caller's value-resolution error or a backend source
    /// validation error.
    pub fn resolve<E>(
        &self,
        id: SourceId,
        base: &std::path::Path,
        mut resolve_value: impl FnMut(&str) -> Result<String, E>,
    ) -> Result<Source, SourceConfigResolveError<E>> {
        match self {
            Self::Sqlite(config) => config
                .resolve(id, base, |value| {
                    resolve_value(value).map_err(SourceConfigResolveError::Value)
                })
                .map(Source::Sqlite),
            Self::Postgres(config) => config
                .resolve(id, &mut resolve_value)
                .map(Source::Postgres)
                .map_err(SourceConfigResolveError::Value),
        }
    }
}

/// Why one backend-owned source configuration could not be resolved.
#[derive(Debug, Error)]
pub enum SourceConfigResolveError<E> {
    /// Application-supplied value resolution failed.
    #[error(transparent)]
    Value(E),
    /// Backend-owned source validation failed.
    #[error(transparent)]
    Backend(#[from] SourceValidationError),
}

impl<E> From<sqlite::SqliteSourceError> for SourceConfigResolveError<E> {
    fn from(error: sqlite::SqliteSourceError) -> Self {
        Self::Backend(SourceValidationError::Sqlite(error))
    }
}

/// Why backend-owned source fields could not form a concrete source.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SourceValidationError {
    /// SQLite source validation failed.
    #[error(transparent)]
    Sqlite(#[from] sqlite::SqliteSourceError),
}

/// The closed composition boundary for backend-owned catalogs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "backend", content = "catalog", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Catalog {
    /// SQLite-owned catalog.
    Sqlite(sqlite::Catalog),
    /// PostgreSQL-owned catalog.
    Postgres(postgres::Catalog),
}

impl Catalog {
    /// Returns the backend family implied by the catalog variant.
    #[must_use]
    pub const fn backend(&self) -> Backend {
        match self {
            Self::Sqlite(_) => Backend::Sqlite,
            Self::Postgres(_) => Backend::Postgres,
        }
    }
}

/// Heterogeneous built-in source snapshot used by application orchestration.
pub type Snapshot = SourceSnapshot<Catalog>;
/// Ordered heterogeneous built-in database context.
pub type DatabaseContext = dbmd_core::DatabaseContext<Catalog>;

/// Introspects one resolved source through its concrete backend module.
///
/// # Errors
///
/// Returns a backend-scoped error when the database cannot be read faithfully.
pub fn introspect(source: &Source) -> Result<Snapshot, IntrospectionError> {
    match source {
        Source::Sqlite(source) => sqlite::introspect(source)
            .map(|snapshot| compose_snapshot(snapshot, Catalog::Sqlite))
            .map_err(IntrospectionError::from),
        Source::Postgres(source) => postgres::introspect(source)
            .map(|snapshot| compose_snapshot(snapshot, Catalog::Postgres))
            .map_err(IntrospectionError::from),
    }
}

fn compose_snapshot<C>(
    snapshot: SourceSnapshot<C>,
    catalog: impl FnOnce(C) -> Catalog,
) -> Snapshot {
    let (id, display_name, backend_catalog) = snapshot.into_parts();
    let snapshot = SourceSnapshot::new(id, catalog(backend_catalog));
    match display_name {
        Some(name) => snapshot.with_display_name(name),
        None => snapshot,
    }
}

/// Builds the backend-neutral presentation context for selected snapshots.
#[must_use]
pub fn render_context(database: &DatabaseContext, nested: bool) -> RenderContext {
    RenderContext::new(
        database
            .sources()
            .iter()
            .map(|snapshot| match snapshot.catalog() {
                Catalog::Sqlite(catalog) => {
                    sqlite::render::source(snapshot.id(), snapshot.display_name(), catalog, nested)
                }
                Catalog::Postgres(catalog) => postgres::render::source(
                    snapshot.id(),
                    snapshot.display_name(),
                    catalog,
                    nested,
                ),
            })
            .collect(),
    )
}

/// Returns all backend templates compiled into this composition root.
#[must_use]
pub fn all_template_files() -> Vec<TemplateFile> {
    sqlite::render::TEMPLATES
        .iter()
        .chain(postgres::render::TEMPLATES)
        .copied()
        .collect()
}

/// Why a resolved database source could not be introspected.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IntrospectionError {
    /// SQLite introspection failed.
    #[error(transparent)]
    Sqlite(#[from] sqlite::IntrospectionError),
    /// PostgreSQL introspection failed.
    #[error(transparent)]
    Postgres(#[from] postgres::IntrospectionError),
}

impl IntrospectionError {
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
