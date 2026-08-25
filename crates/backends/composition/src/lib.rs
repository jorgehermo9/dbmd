#![doc = include_str!("../README.md")]

use dbmd_backend_clickhouse as clickhouse;
use dbmd_backend_duckdb as duckdb;
use dbmd_backend_mariadb as mariadb;
use dbmd_backend_mysql as mysql;
use dbmd_backend_postgres as postgres;
use dbmd_backend_sqlite as sqlite;

use dbmd_core::{SourceId, SourceSnapshot};
use dbmd_render::{RenderContext, TemplateFile};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The built-in database families composed into this binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Backend {
    /// ClickHouse.
    Clickhouse,
    /// DuckDB.
    Duckdb,
    /// MariaDB.
    Mariadb,
    /// MySQL.
    Mysql,
    /// SQLite.
    Sqlite,
    /// PostgreSQL.
    Postgres,
}

impl Backend {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clickhouse => "clickhouse",
            Self::Duckdb => "duckdb",
            Self::Mariadb => "mariadb",
            Self::Mysql => "mysql",
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
        }
    }
}

/// One resolved concrete database source.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Source {
    /// Resolved ClickHouse source.
    Clickhouse(clickhouse::ClickHouseSource),
    /// Resolved DuckDB source.
    Duckdb(duckdb::DuckDbSource),
    /// Resolved MariaDB source.
    Mariadb(mariadb::MariaDbSource),
    /// Resolved MySQL source.
    Mysql(mysql::MysqlSource),
    /// Resolved SQLite source.
    Sqlite(sqlite::SqliteSource),
    /// Resolved PostgreSQL source.
    Postgres(postgres::PostgresSource),
}

impl Source {
    /// Creates a resolved SQLite source from a stable ID and database path.
    #[must_use]
    pub fn sqlite(id: SourceId, path: impl Into<std::path::PathBuf>) -> Self {
        Self::Sqlite(sqlite::SqliteSource::new(id, path))
    }

    /// Creates a resolved DuckDB source from a stable ID and database path.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is empty.
    pub fn duckdb(
        id: SourceId,
        path: impl Into<std::path::PathBuf>,
    ) -> Result<Self, duckdb::DuckDbSourceError> {
        duckdb::DuckDbSource::new(id, path).map(Self::Duckdb)
    }

    #[must_use]
    pub fn id(&self) -> &SourceId {
        match self {
            Self::Clickhouse(source) => source.id(),
            Self::Duckdb(source) => source.id(),
            Self::Mariadb(source) => source.id(),
            Self::Mysql(source) => source.id(),
            Self::Sqlite(source) => source.id(),
            Self::Postgres(source) => source.id(),
        }
    }

    #[must_use]
    pub const fn backend(&self) -> Backend {
        match self {
            Self::Clickhouse(_) => Backend::Clickhouse,
            Self::Duckdb(_) => Backend::Duckdb,
            Self::Mariadb(_) => Backend::Mariadb,
            Self::Mysql(_) => Backend::Mysql,
            Self::Sqlite(_) => Backend::Sqlite,
            Self::Postgres(_) => Backend::Postgres,
        }
    }
}

impl From<clickhouse::ClickHouseSource> for Source {
    fn from(source: clickhouse::ClickHouseSource) -> Self {
        Self::Clickhouse(source)
    }
}
impl From<duckdb::DuckDbSource> for Source {
    fn from(source: duckdb::DuckDbSource) -> Self {
        Self::Duckdb(source)
    }
}
impl From<mariadb::MariaDbSource> for Source {
    fn from(source: mariadb::MariaDbSource) -> Self {
        Self::Mariadb(source)
    }
}
impl From<mysql::MysqlSource> for Source {
    fn from(source: mysql::MysqlSource) -> Self {
        Self::Mysql(source)
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
    /// ClickHouse-specific committed source fields.
    Clickhouse(clickhouse::Config),
    /// DuckDB-specific committed source fields.
    Duckdb(duckdb::Config),
    /// MariaDB-specific committed source fields.
    Mariadb(mariadb::Config),
    /// MySQL-specific committed source fields.
    Mysql(mysql::Config),
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
            Self::Clickhouse(config) => config.environment_values(),
            Self::Duckdb(config) => config.environment_values(),
            Self::Mariadb(config) => config.environment_values(),
            Self::Mysql(config) => config.environment_values(),
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
            Self::Clickhouse(config) => config
                .resolve(id, &mut resolve_value)
                .map(Source::Clickhouse)
                .map_err(SourceConfigResolveError::Value),
            Self::Duckdb(config) => config
                .resolve(id, base, &mut resolve_value)
                .map(Source::Duckdb)
                .map_err(|error| match error {
                    duckdb::DuckDbConfigError::Value(error) => {
                        SourceConfigResolveError::Value(error)
                    }
                    duckdb::DuckDbConfigError::Source(error) => {
                        SourceConfigResolveError::Backend(SourceValidationError::Duckdb(error))
                    }
                }),
            Self::Mariadb(config) => config
                .resolve(id, &mut resolve_value)
                .map(Source::Mariadb)
                .map_err(SourceConfigResolveError::Value),
            Self::Mysql(config) => config
                .resolve(id, &mut resolve_value)
                .map(Source::Mysql)
                .map_err(SourceConfigResolveError::Value),
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
    /// DuckDB source validation failed.
    #[error(transparent)]
    Duckdb(#[from] duckdb::DuckDbSourceError),
    /// SQLite source validation failed.
    #[error(transparent)]
    Sqlite(#[from] sqlite::SqliteSourceError),
}

/// The closed composition boundary for backend-owned catalogs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "backend", content = "catalog", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Catalog {
    /// ClickHouse-owned catalog.
    Clickhouse(Box<clickhouse::Catalog>),
    /// DuckDB-owned catalog.
    Duckdb(Box<duckdb::Catalog>),
    /// MariaDB-owned catalog.
    Mariadb(Box<mariadb::Catalog>),
    /// MySQL-owned catalog.
    Mysql(Box<mysql::Catalog>),
    /// SQLite-owned catalog.
    Sqlite(Box<sqlite::Catalog>),
    /// PostgreSQL-owned catalog.
    Postgres(Box<postgres::Catalog>),
}

impl Catalog {
    /// Returns the backend family implied by the catalog variant.
    #[must_use]
    pub const fn backend(&self) -> Backend {
        match self {
            Self::Clickhouse(_) => Backend::Clickhouse,
            Self::Duckdb(_) => Backend::Duckdb,
            Self::Mariadb(_) => Backend::Mariadb,
            Self::Mysql(_) => Backend::Mysql,
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
        Source::Clickhouse(source) => clickhouse::introspect(source)
            .map(|snapshot| {
                compose_snapshot(snapshot, |catalog| Catalog::Clickhouse(Box::new(catalog)))
            })
            .map_err(IntrospectionError::from),
        Source::Duckdb(source) => duckdb::introspect(source)
            .map(|snapshot| {
                compose_snapshot(snapshot, |catalog| Catalog::Duckdb(Box::new(catalog)))
            })
            .map_err(IntrospectionError::from),
        Source::Mariadb(source) => mariadb::introspect(source)
            .map(|snapshot| {
                compose_snapshot(snapshot, |catalog| Catalog::Mariadb(Box::new(catalog)))
            })
            .map_err(IntrospectionError::from),
        Source::Mysql(source) => mysql::introspect(source)
            .map(|snapshot| compose_snapshot(snapshot, |catalog| Catalog::Mysql(Box::new(catalog))))
            .map_err(IntrospectionError::from),
        Source::Sqlite(source) => sqlite::introspect(source)
            .map(|snapshot| {
                compose_snapshot(snapshot, |catalog| Catalog::Sqlite(Box::new(catalog)))
            })
            .map_err(IntrospectionError::from),
        Source::Postgres(source) => postgres::introspect(source)
            .map(|snapshot| {
                compose_snapshot(snapshot, |catalog| Catalog::Postgres(Box::new(catalog)))
            })
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
                Catalog::Clickhouse(catalog) => clickhouse::render_source(
                    snapshot.id(),
                    snapshot.display_name(),
                    catalog,
                    nested,
                ),
                Catalog::Duckdb(catalog) => {
                    duckdb::render_source(snapshot.id(), snapshot.display_name(), catalog, nested)
                }
                Catalog::Mariadb(catalog) => {
                    mariadb::render_source(snapshot.id(), snapshot.display_name(), catalog, nested)
                }
                Catalog::Mysql(catalog) => {
                    mysql::render_source(snapshot.id(), snapshot.display_name(), catalog, nested)
                }
                Catalog::Sqlite(catalog) => {
                    sqlite::render_source(snapshot.id(), snapshot.display_name(), catalog, nested)
                }
                Catalog::Postgres(catalog) => {
                    postgres::render_source(snapshot.id(), snapshot.display_name(), catalog, nested)
                }
            })
            .collect(),
    )
}

/// Returns all backend templates compiled into this composition root.
#[must_use]
pub fn all_template_files() -> Vec<TemplateFile> {
    sqlite::template_files()
        .iter()
        .chain(postgres::template_files())
        .chain(clickhouse::template_files())
        .chain(duckdb::template_files())
        .chain(mariadb::template_files())
        .chain(mysql::template_files())
        .copied()
        .collect()
}

/// Why a resolved database source could not be introspected.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IntrospectionError {
    /// ClickHouse introspection failed.
    #[error(transparent)]
    Clickhouse(#[from] clickhouse::IntrospectionError),
    /// DuckDB introspection failed.
    #[error(transparent)]
    Duckdb(Box<duckdb::IntrospectionError>),
    /// MariaDB introspection failed.
    #[error(transparent)]
    Mariadb(#[from] mariadb::IntrospectionError),
    /// MySQL introspection failed.
    #[error(transparent)]
    Mysql(#[from] mysql::IntrospectionError),
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
            Self::Clickhouse(error) => error,
            Self::Duckdb(error) => error.as_ref(),
            Self::Mariadb(error) => error,
            Self::Mysql(error) => error,
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

impl From<duckdb::IntrospectionError> for IntrospectionError {
    fn from(error: duckdb::IntrospectionError) -> Self {
        Self::Duckdb(Box::new(error))
    }
}
