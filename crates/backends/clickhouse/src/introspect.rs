//! ClickHouse catalog introspection over the HTTP interface.

use std::{collections::BTreeMap, fmt, time::Duration};

use dbmd_core::{SourceId, SourceSnapshot};
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use thiserror::Error;

use super::catalog::{
    Catalog, Column, ColumnDefaultKind, Constraint, DataSkippingIndex, Database, Projection,
    Snapshot, Table, TableKind, UserDefinedFunction,
};

/// Connection-backed ClickHouse source selected for introspection.
#[derive(Clone, PartialEq, Eq)]
pub struct ClickHouseSource {
    id: SourceId,
    display_name: Option<String>,
    endpoint: String,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl ClickHouseSource {
    /// Creates a source from stable identity and a resolved HTTP endpoint.
    #[must_use]
    pub fn new(id: SourceId, endpoint: impl Into<String>) -> Self {
        Self {
            id,
            display_name: None,
            endpoint: endpoint.into(),
            database: None,
            username: None,
            password: None,
        }
    }

    /// Restricts introspection to one ClickHouse database.
    #[must_use]
    pub fn with_database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    /// Adds optional ClickHouse HTTP credentials.
    #[must_use]
    pub fn with_credentials(mut self, username: Option<String>, password: Option<String>) -> Self {
        self.username = username;
        self.password = password;
        self
    }

    /// Adds a presentation-only source name.
    #[must_use]
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Returns the stable source identity.
    #[must_use]
    pub fn id(&self) -> &SourceId {
        &self.id
    }
}

impl fmt::Debug for ClickHouseSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClickHouseSource")
            .field("id", &self.id)
            .field("display_name", &self.display_name)
            .field("endpoint", &"[REDACTED]")
            .field("database", &self.database)
            .field("username", &self.username.as_ref().map(|_| "[REDACTED]"))
            .field("password", &self.password.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

/// Reads ClickHouse system catalogs into deterministic backend-owned types.
///
/// # Errors
///
/// Returns [`IntrospectionError`] when the HTTP client cannot be built, the
/// server rejects a catalog query, transport fails, or JSON rows cannot be decoded.
pub fn introspect(source: &ClickHouseSource) -> Result<Snapshot, IntrospectionError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(IntrospectionError::Client)?;
    let mut tables = load_tables(&client, source)?;
    attach_columns(&client, source, &mut tables)?;
    attach_indexes(&client, source, &mut tables)?;
    attach_projections(&client, source, &mut tables)?;
    attach_constraints(&client, source, &mut tables)?;
    let snapshot = SourceSnapshot::new(
        source.id.clone(),
        Catalog {
            databases: load_databases(&client, source)?,
            tables,
            functions: load_functions(&client, source)?,
        },
    );
    Ok(match &source.display_name {
        Some(name) => snapshot.with_display_name(name),
        None => snapshot,
    })
}

#[derive(Deserialize)]
struct DatabaseRow {
    name: String,
    engine: String,
    comment: String,
}

fn load_databases(
    client: &Client,
    source: &ClickHouseSource,
) -> Result<Vec<Database>, IntrospectionError> {
    query::<DatabaseRow>(
        client,
        source,
        "databases",
        &format!(
            "SELECT name, engine, comment FROM system.databases WHERE {} ORDER BY name FORMAT JSONEachRow",
            database_filter(source.database.as_deref(), "name")
        ),
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| Database {
                name: row.name,
                engine: row.engine,
                comment: optional(row.comment),
            })
            .collect()
    })
}

#[derive(Deserialize)]
struct TableRow {
    database: String,
    name: String,
    engine: String,
    engine_full: String,
    partition_key: String,
    sorting_key: String,
    primary_key: String,
    sampling_key: String,
    storage_policy: String,
    comment: String,
    create_table_query: String,
}

fn load_tables(
    client: &Client,
    source: &ClickHouseSource,
) -> Result<Vec<Table>, IntrospectionError> {
    query::<TableRow>(
        client,
        source,
        "tables",
        &format!(
            "SELECT database, name, engine, engine_full, partition_key, sorting_key, primary_key, sampling_key, storage_policy, comment, create_table_query FROM system.tables WHERE {} ORDER BY database, name FORMAT JSONEachRow",
            database_filter(source.database.as_deref(), "database")
        ),
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| {
                let kind = table_kind(&row.engine);
                let target = materialized_view_target(&row.create_table_query);
                Table {
                    database: row.database,
                    name: row.name,
                    kind,
                    engine: row.engine,
                    engine_full: row.engine_full,
                    partition_key: row.partition_key,
                    sorting_key: row.sorting_key,
                    primary_key: row.primary_key,
                    sampling_key: row.sampling_key,
                    storage_policy: row.storage_policy,
                    target,
                    comment: optional(row.comment),
                    columns: Vec::new(),
                    data_skipping_indexes: Vec::new(),
                    projections: Vec::new(),
                    constraints: Vec::new(),
                    definition: row.create_table_query,
                }
            })
            .collect()
    })
}

fn materialized_view_target(definition: &str) -> Option<String> {
    let uppercase = definition.to_ascii_uppercase();
    let start = uppercase.find(" TO ")? + 4;
    let remainder = &definition[start..];
    let end = remainder
        .find(|character: char| character.is_whitespace() || character == '(')
        .unwrap_or(remainder.len());
    let target = remainder[..end].replace('`', "");
    (!target.is_empty()).then_some(target)
}

const fn table_kind(engine: &str) -> TableKind {
    match engine.as_bytes() {
        b"View" => TableKind::View,
        b"MaterializedView" => TableKind::MaterializedView,
        b"LiveView" => TableKind::LiveView,
        b"WindowView" => TableKind::WindowView,
        b"Dictionary" => TableKind::Dictionary,
        _ => TableKind::Table,
    }
}

#[derive(Deserialize)]
struct ColumnRow {
    database: String,
    table: String,
    name: String,
    #[serde(rename = "type")]
    data_type: String,
    position: u64,
    default_kind: String,
    default_expression: String,
    comment: String,
    compression_codec: String,
    is_in_partition_key: u8,
    is_in_sorting_key: u8,
    is_in_primary_key: u8,
    is_in_sampling_key: u8,
}

fn attach_columns(
    client: &Client,
    source: &ClickHouseSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let rows = query::<ColumnRow>(
        client,
        source,
        "columns",
        &format!(
            "SELECT database, table, name, type, position, default_kind, default_expression, comment, compression_codec, is_in_partition_key, is_in_sorting_key, is_in_primary_key, is_in_sampling_key FROM system.columns WHERE {} ORDER BY database, table, position FORMAT JSONEachRow",
            database_filter(source.database.as_deref(), "database")
        ),
    )?;
    let mut grouped = BTreeMap::<(String, String), Vec<Column>>::new();
    for row in rows {
        grouped
            .entry((row.database, row.table))
            .or_default()
            .push(Column {
                name: row.name,
                data_type: row.data_type,
                position: row.position,
                default_kind: default_kind(row.default_kind),
                default_expression: optional(row.default_expression),
                comment: optional(row.comment),
                compression_codec: optional(row.compression_codec),
                in_partition_key: row.is_in_partition_key != 0,
                in_sorting_key: row.is_in_sorting_key != 0,
                in_primary_key: row.is_in_primary_key != 0,
                in_sampling_key: row.is_in_sampling_key != 0,
            });
    }
    attach(tables, grouped, |table, values| table.columns = values);
    Ok(())
}

fn default_kind(value: String) -> ColumnDefaultKind {
    match value.as_str() {
        "" => ColumnDefaultKind::None,
        "DEFAULT" => ColumnDefaultKind::Default,
        "MATERIALIZED" => ColumnDefaultKind::Materialized,
        "ALIAS" => ColumnDefaultKind::Alias,
        "EPHEMERAL" => ColumnDefaultKind::Ephemeral,
        _ => ColumnDefaultKind::Unknown(value.to_ascii_lowercase()),
    }
}

#[derive(Deserialize)]
struct IndexRow {
    database: String,
    table: String,
    name: String,
    #[serde(rename = "type")]
    index_type: String,
    type_full: String,
    expr: String,
    granularity: u64,
}

#[derive(Deserialize)]
struct IndexRowWithCreation {
    database: String,
    table: String,
    name: String,
    #[serde(rename = "type")]
    index_type: String,
    type_full: String,
    expr: String,
    creation: String,
    granularity: u64,
}

fn attach_indexes(
    client: &Client,
    source: &ClickHouseSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let mut grouped = BTreeMap::<(String, String), Vec<DataSkippingIndex>>::new();
    let filter = database_filter(source.database.as_deref(), "database");
    if has_system_column(client, source, "data_skipping_indices", "creation")? {
        for row in query::<IndexRowWithCreation>(
            client,
            source,
            "data-skipping indexes",
            &format!(
                "SELECT database, table, name, type, type_full, expr, toString(creation) AS creation, granularity FROM system.data_skipping_indices WHERE {filter} ORDER BY database, table, name FORMAT JSONEachRow"
            ),
        )? {
            grouped
                .entry((row.database, row.table))
                .or_default()
                .push(DataSkippingIndex {
                    name: row.name,
                    expression: row.expr,
                    index_type: row.index_type,
                    type_full: row.type_full,
                    implicit: Some(row.creation == "Implicit"),
                    granularity: row.granularity,
                });
        }
    } else {
        for row in query::<IndexRow>(
            client,
            source,
            "data-skipping indexes",
            &format!(
                "SELECT database, table, name, type, type_full, expr, granularity FROM system.data_skipping_indices WHERE {filter} ORDER BY database, table, name FORMAT JSONEachRow"
            ),
        )? {
            grouped
                .entry((row.database, row.table))
                .or_default()
                .push(DataSkippingIndex {
                    name: row.name,
                    expression: row.expr,
                    index_type: row.index_type,
                    type_full: row.type_full,
                    implicit: None,
                    granularity: row.granularity,
                });
        }
    }
    attach(tables, grouped, |table, values| {
        table.data_skipping_indexes = values
    });
    Ok(())
}

#[derive(Deserialize)]
struct CountRow {
    count: u64,
}

fn has_system_column(
    client: &Client,
    source: &ClickHouseSource,
    table: &str,
    column: &str,
) -> Result<bool, IntrospectionError> {
    query::<CountRow>(
        client,
        source,
        "catalog capability",
        &format!(
            "SELECT count() AS count FROM system.columns WHERE database = 'system' AND table = '{}' AND name = '{}' FORMAT JSONEachRow",
            escape_literal(table),
            escape_literal(column)
        ),
    )
    .map(|rows| rows.first().is_some_and(|row| row.count != 0))
}

fn has_system_table(
    client: &Client,
    source: &ClickHouseSource,
    table: &str,
) -> Result<bool, IntrospectionError> {
    query::<CountRow>(
        client,
        source,
        "catalog capability",
        &format!(
            "SELECT count() AS count FROM system.tables WHERE database = 'system' AND name = '{}' FORMAT JSONEachRow",
            escape_literal(table)
        ),
    )
    .map(|rows| rows.first().is_some_and(|row| row.count != 0))
}

#[derive(Deserialize)]
struct ProjectionRow {
    database: String,
    table: String,
    name: String,
    #[serde(rename = "type")]
    projection_type: String,
    sorting_key: String,
    query: String,
}

fn attach_projections(
    client: &Client,
    source: &ClickHouseSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let rows = query::<ProjectionRow>(
        client,
        source,
        "projections",
        &format!(
            "SELECT database, table, name, toString(type) AS type, arrayStringConcat(sorting_key, ', ') AS sorting_key, query FROM system.projections WHERE {} ORDER BY database, table, name FORMAT JSONEachRow",
            database_filter(source.database.as_deref(), "database")
        ),
    )?;
    let mut grouped = BTreeMap::<(String, String), Vec<Projection>>::new();
    for row in rows {
        grouped
            .entry((row.database, row.table))
            .or_default()
            .push(Projection {
                name: row.name,
                projection_type: row.projection_type,
                sorting_key: row.sorting_key,
                query: row.query,
            });
    }
    attach(tables, grouped, |table, values| table.projections = values);
    Ok(())
}

#[derive(Deserialize)]
struct ConstraintRow {
    database: String,
    table: String,
    name: String,
    #[serde(rename = "type")]
    constraint_type: String,
    expression: String,
}

fn attach_constraints(
    client: &Client,
    source: &ClickHouseSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    if !has_system_table(client, source, "constraints")? {
        for table in tables {
            table.constraints = constraints_from_definition(&table.definition);
        }
        return Ok(());
    }

    let rows = query::<ConstraintRow>(
        client,
        source,
        "constraints",
        &format!(
            "SELECT database, table, name, type, expression FROM system.constraints WHERE {} ORDER BY database, table, name FORMAT JSONEachRow",
            database_filter(source.database.as_deref(), "database")
        ),
    )?;
    let mut grouped = BTreeMap::<(String, String), Vec<Constraint>>::new();
    for row in rows {
        grouped
            .entry((row.database, row.table))
            .or_default()
            .push(Constraint {
                name: row.name,
                constraint_type: row.constraint_type,
                expression: row.expression,
            });
    }
    attach(tables, grouped, |table, values| table.constraints = values);
    Ok(())
}

fn constraints_from_definition(definition: &str) -> Vec<Constraint> {
    let uppercase = definition.to_ascii_uppercase();
    let mut constraints = Vec::new();
    let mut search_from = 0;

    while let Some(relative_start) = uppercase[search_from..].find("CONSTRAINT ") {
        let keyword_start = search_from + relative_start;
        let name_start = keyword_start + "CONSTRAINT ".len();
        let remainder = &definition[name_start..];
        let name_end = remainder
            .find(char::is_whitespace)
            .unwrap_or(remainder.len());
        let name = remainder[..name_end].trim_matches('`');
        let after_name = remainder[name_end..].trim_start();
        let upper_after_name = after_name.to_ascii_uppercase();
        let (constraint_type, expression_start) = if upper_after_name.starts_with("CHECK ") {
            ("CHECK", "CHECK ".len())
        } else if upper_after_name.starts_with("ASSUME ") {
            ("ASSUME", "ASSUME ".len())
        } else {
            search_from = name_start;
            continue;
        };

        let expression_remainder = &after_name[expression_start..];
        let expression_end = expression_boundary(expression_remainder);
        let expression = expression_remainder[..expression_end].trim();
        if !name.is_empty() && !expression.is_empty() {
            constraints.push(Constraint {
                name: name.to_string(),
                constraint_type: constraint_type.to_string(),
                expression: expression.to_string(),
            });
        }

        search_from = name_start.saturating_add(name_end).min(definition.len());
    }

    constraints
}

fn expression_boundary(expression: &str) -> usize {
    let mut depth = 0_u32;
    let mut quoted = false;
    let mut previous = '\0';

    for (offset, character) in expression.char_indices() {
        match character {
            '\'' if previous != '\\' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted && depth == 0 => return offset,
            ')' if !quoted => depth -= 1,
            ',' if !quoted && depth == 0 => return offset,
            _ => {}
        }
        previous = character;
    }

    expression.len()
}

fn attach<T>(
    tables: &mut [Table],
    mut grouped: BTreeMap<(String, String), Vec<T>>,
    mut assign: impl FnMut(&mut Table, Vec<T>),
) {
    for table in tables {
        if let Some(values) = grouped.remove(&(table.database.clone(), table.name.clone())) {
            assign(table, values);
        }
    }
}

#[derive(Deserialize)]
struct FunctionRow {
    name: String,
    create_query: String,
}

fn load_functions(
    client: &Client,
    source: &ClickHouseSource,
) -> Result<Vec<UserDefinedFunction>, IntrospectionError> {
    let catalog = if has_system_table(client, source, "user_defined_functions")? {
        "system.user_defined_functions"
    } else {
        "system.functions WHERE origin = 'SQLUserDefined'"
    };
    query::<FunctionRow>(
        client,
        source,
        "user-defined functions",
        &format!("SELECT name, create_query FROM {catalog} ORDER BY name FORMAT JSONEachRow"),
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| UserDefinedFunction {
                name: row.name,
                definition: row.create_query,
            })
            .collect()
    })
}

fn query<T: DeserializeOwned>(
    client: &Client,
    source: &ClickHouseSource,
    operation: &'static str,
    sql: &str,
) -> Result<Vec<T>, IntrospectionError> {
    let mut request = client.post(&source.endpoint).body(sql.to_string());
    if let Some(database) = &source.database {
        request = request.query(&[("database", database)]);
    }
    if let Some(username) = &source.username {
        request = request.header("X-ClickHouse-User", username);
    }
    if let Some(password) = &source.password {
        request = request.header("X-ClickHouse-Key", password);
    }
    let response = request
        .send()
        .map_err(|error| IntrospectionError::Request {
            source_id: source.id.clone(),
            operation,
            source: error,
        })?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|error| IntrospectionError::Request {
            source_id: source.id.clone(),
            operation,
            source: error,
        })?;
    if !status.is_success() {
        return Err(IntrospectionError::Server {
            source_id: source.id.clone(),
            operation,
            status: status.as_u16(),
            message: body.trim().to_string(),
        });
    }
    body.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(line_index, line)| {
            serde_json::from_str(line).map_err(|error| IntrospectionError::Decode {
                source_id: source.id.clone(),
                operation,
                line: line_index + 1,
                source: error,
            })
        })
        .collect()
}

fn database_filter(database: Option<&str>, column: &str) -> String {
    match database {
        Some(database) => format!("{column} = '{}'", escape_literal(database)),
        None => format!("{column} NOT IN ('system', 'information_schema', 'INFORMATION_SCHEMA')"),
    }
}

fn escape_literal(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn optional(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

/// Why ClickHouse introspection could not produce a trustworthy snapshot.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IntrospectionError {
    /// The blocking HTTP client could not be constructed.
    #[error("could not build ClickHouse HTTP client: {0}")]
    Client(#[source] reqwest::Error),
    /// A ClickHouse request failed before a response was available.
    #[error("could not query ClickHouse {operation} for source `{source_id}`")]
    Request {
        /// Stable source identity.
        source_id: SourceId,
        /// Catalog operation.
        operation: &'static str,
        /// Transport failure.
        #[source]
        source: reqwest::Error,
    },
    /// ClickHouse rejected a catalog query.
    #[error("ClickHouse rejected {operation} query for source `{source_id}` with HTTP {status}: {message}")]
    Server {
        /// Stable source identity.
        source_id: SourceId,
        /// Catalog operation.
        operation: &'static str,
        /// HTTP status.
        status: u16,
        /// Credential-free server diagnostic.
        message: String,
    },
    /// A JSONEachRow catalog row did not match the supported contract.
    #[error("could not decode ClickHouse {operation} row {line} for source `{source_id}`")]
    Decode {
        /// Stable source identity.
        source_id: SourceId,
        /// Catalog operation.
        operation: &'static str,
        /// One-based response line.
        line: usize,
        /// JSON decoding failure.
        #[source]
        source: serde_json::Error,
    },
}
