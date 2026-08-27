//! ClickHouse catalog introspection over the HTTP interface.

use std::{collections::BTreeMap, fmt, time::Duration};

use dbmd_core::{SourceId, SourceSnapshot};
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use thiserror::Error;

use super::catalog::{
    AccessTarget, Catalog, Column, ColumnDefaultKind, Constraint, ConstraintKind,
    DataSkippingIndex, Database, DictionaryDetails, DictionaryField, Grant, NamedCollection,
    Projection, ProjectionIndex, Quota, QuotaLimit, Resource, Role, RoleGrant, RowPolicy,
    SettingsProfile, SettingsProfileElement, Snapshot, Table, TableKind, TableReference, User,
    UserDefinedFunction, UserDefinedFunctionOrigin, UserHosts, ViewParameter, Workload,
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
    attach_dictionaries(&client, source, &mut tables)?;
    let snapshot = SourceSnapshot::new(
        source.id.clone(),
        Catalog {
            databases: load_databases(&client, source)?,
            tables,
            functions: load_functions(&client, source)?,
            users: load_users(&client, source)?,
            roles: load_roles(&client, source)?,
            grants: load_grants(&client, source)?,
            role_grants: load_role_grants(&client, source)?,
            row_policies: load_row_policies(&client, source)?,
            quotas: load_quotas(&client, source)?,
            settings_profiles: load_settings_profiles(&client, source)?,
            named_collections: load_named_collections(&client, source)?,
            resources: load_resources(&client, source)?,
            workloads: load_workloads(&client, source)?,
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
    uuid: String,
    engine_full: String,
    comment: String,
    is_external: u8,
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
            "SELECT name, engine, toString(uuid) AS uuid, engine_full, comment, is_external FROM system.databases WHERE {} ORDER BY name FORMAT JSONEachRow",
            database_filter(source.database.as_deref(), "name")
        ),
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| Database {
                name: row.name,
                engine: row.engine,
                uuid: row.uuid,
                engine_full: row.engine_full,
                external: row.is_external != 0,
                comment: optional(row.comment),
            })
            .collect()
    })
}

#[derive(Deserialize)]
struct TableRow {
    database: String,
    name: String,
    uuid: String,
    engine: String,
    is_temporary: u8,
    engine_full: String,
    as_select: String,
    parameterized_view_parameters: Vec<ViewParameter>,
    partition_key: String,
    sorting_key: String,
    primary_key: String,
    sampling_key: String,
    unique_key: String,
    storage_policy: String,
    dependencies_database: Vec<String>,
    dependencies_table: Vec<String>,
    loading_dependencies_database: Vec<String>,
    loading_dependencies_table: Vec<String>,
    loading_dependent_database: Vec<String>,
    loading_dependent_table: Vec<String>,
    target_database: String,
    target_table: String,
    definer: String,
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
            "SELECT database, name, toString(uuid) AS uuid, engine, is_temporary, engine_full, as_select, parameterized_view_parameters, partition_key, sorting_key, primary_key, sampling_key, unique_key, storage_policy, dependencies_database, dependencies_table, loading_dependencies_database, loading_dependencies_table, loading_dependent_database, loading_dependent_table, target_database, target_table, definer, comment, create_table_query FROM system.tables WHERE {} AND NOT startsWith(name, '.inner.') AND NOT startsWith(name, '.inner_id.') ORDER BY database, name FORMAT JSONEachRow",
            database_filter(source.database.as_deref(), "database")
        ),
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| {
                let kind = table_kind(&row.engine);
                let metadata =
                    crate::ddl::table_metadata(&row.engine, &row.engine_full, &row.create_table_query);
                let refresh = crate::ddl::view_refresh(&row.database, &row.create_table_query);
                let window = (kind == TableKind::WindowView)
                    .then(|| crate::ddl::window_view(&row.create_table_query));
                let definer = optional(row.definer)
                    .or_else(|| crate::ddl::view_definer(&row.create_table_query));
                let sql_security = crate::ddl::view_sql_security(&row.create_table_query);
                let target = if !matches!(kind, TableKind::MaterializedView | TableKind::WindowView)
                {
                    None
                } else if row.target_database.is_empty() || row.target_table.is_empty() {
                    crate::ddl::view_target(&row.create_table_query)
                } else {
                    Some(format!("{}.{}", row.target_database, row.target_table))
                };
                Table {
                    database: row.database,
                    name: row.name,
                    uuid: row.uuid,
                    kind,
                    engine: row.engine,
                    engine_full: row.engine_full,
                    engine_arguments: metadata.engine_arguments,
                    engine_parameters: metadata.engine_parameters,
                    settings: metadata.settings,
                    ttl_rules: metadata.ttl_rules,
                    temporary: row.is_temporary != 0,
                    as_select: optional(row.as_select),
                    parameters: row.parameterized_view_parameters,
                    partition_key: row.partition_key,
                    sorting_key: row.sorting_key,
                    primary_key: row.primary_key,
                    sampling_key: row.sampling_key,
                    unique_key: row.unique_key,
                    storage_policy: row.storage_policy,
                    target,
                    refresh,
                    window,
                    dependencies: references(row.dependencies_database, row.dependencies_table),
                    loading_dependencies: references(
                        row.loading_dependencies_database,
                        row.loading_dependencies_table,
                    ),
                    loading_dependents: references(
                        row.loading_dependent_database,
                        row.loading_dependent_table,
                    ),
                    definer,
                    sql_security,
                    comment: optional(row.comment),
                    columns: Vec::new(),
                    data_skipping_indexes: Vec::new(),
                    projections: Vec::new(),
                    constraints: Vec::new(),
                    dictionary: None,
                    definition: row.create_table_query,
                }
            })
            .collect()
    })
}

fn references(databases: Vec<String>, tables: Vec<String>) -> Vec<TableReference> {
    let mut references = databases
        .into_iter()
        .zip(tables)
        .map(|(database, table)| TableReference { database, table })
        .collect::<Vec<_>>();
    references.sort_unstable_by(|left, right| {
        (&left.database, &left.table).cmp(&(&right.database, &right.table))
    });
    references
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
    character_octet_length: Option<u64>,
    numeric_precision: Option<u64>,
    numeric_precision_radix: Option<u64>,
    numeric_scale: Option<u64>,
    datetime_precision: Option<u64>,
    default_kind: String,
    default_expression: String,
    comment: String,
    compression_codec: String,
    serialization_hint: Option<String>,
    statistics: String,
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
            "SELECT database, table, name, type, position, character_octet_length, numeric_precision, numeric_precision_radix, numeric_scale, datetime_precision, default_kind, default_expression, comment, compression_codec, serialization_hint, statistics, is_in_partition_key, is_in_sorting_key, is_in_primary_key, is_in_sampling_key FROM system.columns WHERE {} ORDER BY database, table, position FORMAT JSONEachRow",
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
                character_octet_length: row.character_octet_length,
                numeric_precision: row.numeric_precision,
                numeric_precision_radix: row.numeric_precision_radix,
                numeric_scale: row.numeric_scale,
                datetime_precision: row.datetime_precision,
                default_kind: closed_value(
                    source,
                    "columns",
                    "default_kind",
                    &row.default_kind,
                    default_kind,
                )?,
                default_expression: optional(row.default_expression),
                comment: optional(row.comment),
                compression_codec: optional(row.compression_codec),
                serialization_hint: row.serialization_hint.and_then(optional),
                statistics: optional(row.statistics),
                ttl: None,
                in_partition_key: row.is_in_partition_key != 0,
                in_sorting_key: row.is_in_sorting_key != 0,
                in_primary_key: row.is_in_primary_key != 0,
                in_sampling_key: row.is_in_sampling_key != 0,
            });
    }
    attach(tables, grouped, |table, values| table.columns = values);
    for table in tables {
        let metadata =
            crate::ddl::table_metadata(&table.engine, &table.engine_full, &table.definition);
        for column in &mut table.columns {
            column.ttl = metadata.column_ttls.get(&column.name).cloned();
        }
    }
    Ok(())
}

fn default_kind(value: &str) -> Option<ColumnDefaultKind> {
    match value {
        "" => Some(ColumnDefaultKind::None),
        "DEFAULT" => Some(ColumnDefaultKind::Default),
        "MATERIALIZED" => Some(ColumnDefaultKind::Materialized),
        "ALIAS" => Some(ColumnDefaultKind::Alias),
        "EPHEMERAL" => Some(ColumnDefaultKind::Ephemeral),
        _ => None,
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
    settings: BTreeMap<String, String>,
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
            "SELECT database, table, name, toString(type) AS type, arrayStringConcat(sorting_key, ', ') AS sorting_key, query, settings FROM system.projections WHERE {} ORDER BY database, table, name FORMAT JSONEachRow",
            database_filter(source.database.as_deref(), "database")
        ),
    )?;
    let definitions = tables
        .iter()
        .map(|table| {
            (
                (table.database.clone(), table.name.clone()),
                table.definition.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut grouped = BTreeMap::<(String, String), Vec<Projection>>::new();
    for row in rows {
        let key = (row.database.clone(), row.table.clone());
        let index = definitions
            .get(&key)
            .and_then(|definition| projection_index(definition, &row.name));
        grouped.entry(key).or_default().push(Projection {
            name: row.name,
            projection_type: row.projection_type,
            sorting_key: row.sorting_key,
            query: row.query,
            settings: row.settings,
            index,
        });
    }
    attach(tables, grouped, |table, values| table.projections = values);
    Ok(())
}

fn projection_index(definition: &str, name: &str) -> Option<ProjectionIndex> {
    let uppercase = definition.to_ascii_uppercase();
    let marker = format!("PROJECTION {} INDEX ", name.to_ascii_uppercase());
    let quoted_marker = format!("PROJECTION `{}` INDEX ", name.to_ascii_uppercase());
    let start = uppercase
        .find(&marker)
        .map(|index| index + marker.len())
        .or_else(|| {
            uppercase
                .find(&quoted_marker)
                .map(|index| index + quoted_marker.len())
        })?;
    let remainder = &definition[start..];
    let type_offset = remainder.to_ascii_uppercase().find(" TYPE ")?;
    let expression = remainder[..type_offset].trim().to_string();
    let type_remainder = remainder[type_offset + " TYPE ".len()..].trim_start();
    let end = type_remainder
        .find(|character: char| character.is_whitespace() || matches!(character, ',' | ')'))
        .unwrap_or(type_remainder.len());
    Some(ProjectionIndex {
        expression,
        index_type: type_remainder[..end].to_string(),
    })
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
                kind: closed_value(
                    source,
                    "constraints",
                    "type",
                    &row.constraint_type,
                    constraint_kind,
                )?,
                expression: row.expression,
            });
    }
    attach(tables, grouped, |table, values| table.constraints = values);
    Ok(())
}

#[derive(Deserialize)]
struct DictionaryRow {
    database: String,
    name: String,
    layout: String,
    key_names: Vec<String>,
    key_types: Vec<String>,
    attribute_names: Vec<String>,
    attribute_types: Vec<String>,
    source: String,
    lifetime_min: u64,
    lifetime_max: u64,
}

fn attach_dictionaries(
    client: &Client,
    source: &ClickHouseSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    if !has_system_table(client, source, "dictionaries")? {
        return Ok(());
    }
    // `source` is the server's non-secret source description. The normalized
    // table definition is separately redacted by ClickHouse. No dictionary
    // configuration value is selected directly.
    let rows = query::<DictionaryRow>(
        client,
        source,
        "dictionaries",
        &format!(
            "SELECT database, name, type AS layout, key.names AS key_names, key.types AS key_types, attribute.names AS attribute_names, attribute.types AS attribute_types, source, lifetime_min, lifetime_max FROM system.dictionaries WHERE {} ORDER BY database, name FORMAT JSONEachRow",
            database_filter(source.database.as_deref(), "database")
        ),
    )?;
    let details = rows
        .into_iter()
        .map(|row| {
            (
                (row.database, row.name),
                DictionaryDetails {
                    layout: row.layout,
                    keys: dictionary_fields(row.key_names, row.key_types),
                    attributes: dictionary_fields(row.attribute_names, row.attribute_types),
                    source: row.source,
                    lifetime_min_seconds: row.lifetime_min,
                    lifetime_max_seconds: row.lifetime_max,
                    range_min: None,
                    range_max: None,
                    settings: BTreeMap::new(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    for table in tables {
        table.dictionary = details
            .get(&(table.database.clone(), table.name.clone()))
            .cloned();
        if let Some(dictionary) = &mut table.dictionary {
            let metadata = crate::ddl::dictionary_metadata(&table.definition);
            if dictionary.layout.is_empty() {
                dictionary.layout = dictionary_layout(&table.definition).unwrap_or_default();
            }
            crate::ddl::apply_dictionary_metadata(&mut dictionary.keys, &metadata);
            crate::ddl::apply_dictionary_metadata(&mut dictionary.attributes, &metadata);
            dictionary.range_min = metadata.range_min;
            dictionary.range_max = metadata.range_max;
            dictionary.settings = metadata.settings;
            if let Some(lifetime) = metadata.lifetime_min_seconds {
                dictionary.lifetime_min_seconds = lifetime;
            }
            if let Some(lifetime) = metadata.lifetime_max_seconds {
                dictionary.lifetime_max_seconds = lifetime;
            }
        }
    }
    Ok(())
}

fn dictionary_fields(names: Vec<String>, types: Vec<String>) -> Vec<DictionaryField> {
    names
        .into_iter()
        .zip(types)
        .map(|(name, data_type)| DictionaryField {
            name,
            data_type,
            default_expression: None,
            expression: None,
            hierarchical: false,
            injective: false,
            object_id: false,
        })
        .collect()
}

fn dictionary_layout(definition: &str) -> Option<String> {
    let uppercase = definition.to_ascii_uppercase();
    let start = uppercase.find("LAYOUT(")? + "LAYOUT(".len();
    let remainder = &definition[start..];
    let end = remainder
        .find(|character: char| character == '(' || character == ')' || character.is_whitespace())
        .unwrap_or(remainder.len());
    let layout = remainder[..end].trim();
    (!layout.is_empty()).then(|| layout.to_string())
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
                kind: constraint_kind(constraint_type)
                    .expect("DDL parser only emits supported constraint kinds"),
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
    origin: String,
    create_query: String,
    syntax: String,
    arguments: String,
    returned_value: String,
}

fn load_functions(
    client: &Client,
    source: &ClickHouseSource,
) -> Result<Vec<UserDefinedFunction>, IntrospectionError> {
    query::<FunctionRow>(
        client,
        source,
        "user-defined functions",
        "SELECT name, toString(origin) AS origin, create_query, syntax, arguments, returned_value FROM system.functions WHERE origin IN ('SQLUserDefined', 'WasmUserDefined') ORDER BY name FORMAT JSONEachRow",
    )
    .and_then(|rows| {
        rows.into_iter()
            .map(|row| {
                let origin = closed_value(
                    source,
                    "user-defined functions",
                    "origin",
                    &row.origin,
                    |value| match value {
                        "SQLUserDefined" => Some(UserDefinedFunctionOrigin::SqlDefined),
                        "WasmUserDefined" => {
                            Some(UserDefinedFunctionOrigin::WebAssemblyDefined)
                        }
                        _ => None,
                    },
                )?;
                Ok(UserDefinedFunction {
                    name: row.name,
                    origin,
                    syntax: optional(row.syntax),
                    arguments: optional(row.arguments),
                    returned_value: optional(row.returned_value),
                    definition: row.create_query,
                })
            })
            .collect()
    })
}

#[derive(Deserialize)]
struct UserRow {
    name: String,
    storage: String,
    auth_type: Vec<String>,
    valid_until: Vec<String>,
    host_ip: Vec<String>,
    host_names: Vec<String>,
    host_names_regexp: Vec<String>,
    host_names_like: Vec<String>,
    default_roles_all: u8,
    default_roles_list: Vec<String>,
    default_roles_except: Vec<String>,
    grantees_any: u8,
    grantees_list: Vec<String>,
    grantees_except: Vec<String>,
    default_database: String,
}

fn load_users(client: &Client, source: &ClickHouseSource) -> Result<Vec<User>, IntrospectionError> {
    if !has_system_table(client, source, "users")? {
        return Ok(Vec::new());
    }
    // `auth_params` is deliberately not selected: it can contain password
    // hashes, salts, SSH keys, or external-authentication parameters.
    query::<UserRow>(
        client,
        source,
        "users",
        "SELECT name, storage, arrayMap(value -> toString(value), auth_type) AS auth_type, arrayMap(value -> toString(value), valid_until) AS valid_until, host_ip, host_names, host_names_regexp, host_names_like, default_roles_all, default_roles_list, default_roles_except, grantees_any, grantees_list, grantees_except, default_database FROM system.users WHERE storage != 'users_xml' ORDER BY name FORMAT JSONEachRow",
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| User {
                name: row.name,
                storage: row.storage,
                authentication_types: row.auth_type,
                valid_until: row
                    .valid_until
                    .into_iter()
                    .map(|value| (value != "1970-01-01 00:00:00").then_some(value))
                    .collect(),
                hosts: UserHosts {
                    ip: row.host_ip,
                    names: row.host_names,
                    name_regexps: row.host_names_regexp,
                    name_like_patterns: row.host_names_like,
                },
                default_roles: AccessTarget {
                    all: row.default_roles_all != 0,
                    include: row.default_roles_list,
                    except: row.default_roles_except,
                },
                grantees: AccessTarget {
                    all: row.grantees_any != 0,
                    include: row.grantees_list,
                    except: row.grantees_except,
                },
                default_database: optional(row.default_database),
            })
            .collect()
    })
}

#[derive(Deserialize)]
struct RoleRow {
    name: String,
    storage: String,
}

fn load_roles(client: &Client, source: &ClickHouseSource) -> Result<Vec<Role>, IntrospectionError> {
    if !has_system_table(client, source, "roles")? {
        return Ok(Vec::new());
    }
    query::<RoleRow>(
        client,
        source,
        "roles",
        "SELECT name, storage FROM system.roles WHERE storage != 'users_xml' ORDER BY name FORMAT JSONEachRow",
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| Role {
                name: row.name,
                storage: row.storage,
            })
            .collect()
    })
}

#[derive(Deserialize)]
struct GrantRow {
    user_name: Option<String>,
    role_name: Option<String>,
    access_type: String,
    access_object: String,
    database: Option<String>,
    table: Option<String>,
    column: Option<String>,
    is_partial_revoke: u8,
    grant_option: u8,
}

fn load_grants(
    client: &Client,
    source: &ClickHouseSource,
) -> Result<Vec<Grant>, IntrospectionError> {
    if !has_system_table(client, source, "grants")? {
        return Ok(Vec::new());
    }
    query::<GrantRow>(
        client,
        source,
        "grants",
        "SELECT user_name, role_name, toString(access_type) AS access_type, access_object, database, table, column, is_partial_revoke, grant_option FROM system.grants WHERE user_name IN (SELECT name FROM system.users WHERE storage != 'users_xml') OR role_name IN (SELECT name FROM system.roles WHERE storage != 'users_xml') ORDER BY ifNull(user_name, ''), ifNull(role_name, ''), access_type, ifNull(database, ''), ifNull(table, ''), ifNull(column, '') FORMAT JSONEachRow",
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| Grant {
                user: row.user_name,
                role: row.role_name,
                access_type: row.access_type,
                access_object: optional(row.access_object),
                database: row.database,
                table: row.table,
                column: row.column,
                partial_revoke: row.is_partial_revoke != 0,
                grant_option: row.grant_option != 0,
            })
            .collect()
    })
}

#[derive(Deserialize)]
struct RoleGrantRow {
    user_name: Option<String>,
    role_name: Option<String>,
    granted_role_name: String,
    granted_role_is_default: u8,
    with_admin_option: u8,
}

fn load_role_grants(
    client: &Client,
    source: &ClickHouseSource,
) -> Result<Vec<RoleGrant>, IntrospectionError> {
    if !has_system_table(client, source, "role_grants")? {
        return Ok(Vec::new());
    }
    query::<RoleGrantRow>(
        client,
        source,
        "role grants",
        "SELECT user_name, role_name, granted_role_name, granted_role_is_default, with_admin_option FROM system.role_grants WHERE user_name IN (SELECT name FROM system.users WHERE storage != 'users_xml') OR role_name IN (SELECT name FROM system.roles WHERE storage != 'users_xml') ORDER BY ifNull(user_name, ''), ifNull(role_name, ''), granted_role_name FORMAT JSONEachRow",
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| RoleGrant {
                user: row.user_name,
                role: row.role_name,
                granted_role: row.granted_role_name,
                default: row.granted_role_is_default != 0,
                admin_option: row.with_admin_option != 0,
            })
            .collect()
    })
}

#[derive(Deserialize)]
struct RowPolicyRow {
    name: String,
    short_name: String,
    database: String,
    table: String,
    storage: String,
    select_filter: Option<String>,
    is_restrictive: u8,
    apply_to_all: u8,
    apply_to_list: Vec<String>,
    apply_to_except: Vec<String>,
}

fn load_row_policies(
    client: &Client,
    source: &ClickHouseSource,
) -> Result<Vec<RowPolicy>, IntrospectionError> {
    if !has_system_table(client, source, "row_policies")? {
        return Ok(Vec::new());
    }
    query::<RowPolicyRow>(
        client,
        source,
        "row policies",
        &format!(
            "SELECT name, short_name, database, table, storage, select_filter, is_restrictive, apply_to_all, apply_to_list, apply_to_except FROM system.row_policies WHERE storage != 'users_xml' AND {} ORDER BY name FORMAT JSONEachRow",
            database_filter(source.database.as_deref(), "database")
        ),
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| RowPolicy {
                name: row.name,
                short_name: row.short_name,
                database: row.database,
                table: optional(row.table),
                storage: row.storage,
                select_filter: row.select_filter,
                restrictive: row.is_restrictive != 0,
                target: AccessTarget {
                    all: row.apply_to_all != 0,
                    include: row.apply_to_list,
                    except: row.apply_to_except,
                },
            })
            .collect()
    })
}

#[derive(Deserialize)]
struct QuotaRow {
    name: String,
    storage: String,
    keys: Vec<String>,
    apply_to_all: u8,
    apply_to_list: Vec<String>,
    apply_to_except: Vec<String>,
    ipv4_prefix_bits: Option<u8>,
    ipv6_prefix_bits: Option<u8>,
}

#[derive(Deserialize)]
struct QuotaLimitRow {
    quota_name: String,
    duration: u64,
    is_randomized_interval: u8,
    max_queries: Option<u64>,
    max_query_selects: Option<u64>,
    max_query_inserts: Option<u64>,
    max_errors: Option<u64>,
    max_result_rows: Option<u64>,
    max_result_bytes: Option<u64>,
    max_read_rows: Option<u64>,
    max_read_bytes: Option<u64>,
    max_execution_time: Option<String>,
    max_written_bytes: Option<u64>,
    max_failed_sequential_authentications: Option<u64>,
    max_queries_per_normalized_hash: Option<u64>,
}

fn load_quotas(
    client: &Client,
    source: &ClickHouseSource,
) -> Result<Vec<Quota>, IntrospectionError> {
    if !has_system_table(client, source, "quotas")? {
        return Ok(Vec::new());
    }
    let mut limits = BTreeMap::<String, Vec<QuotaLimit>>::new();
    if has_system_table(client, source, "quota_limits")? {
        for row in query::<QuotaLimitRow>(
            client,
            source,
            "quota limits",
            "SELECT quota_name, duration, is_randomized_interval, max_queries, max_query_selects, max_query_inserts, max_errors, max_result_rows, max_result_bytes, max_read_rows, max_read_bytes, toString(max_execution_time) AS max_execution_time, max_written_bytes, max_failed_sequential_authentications, max_queries_per_normalized_hash FROM system.quota_limits ORDER BY quota_name, duration, is_randomized_interval FORMAT JSONEachRow",
        )? {
            limits.entry(row.quota_name).or_default().push(QuotaLimit {
                duration_seconds: row.duration,
                randomized: row.is_randomized_interval != 0,
                max_queries: row.max_queries,
                max_query_selects: row.max_query_selects,
                max_query_inserts: row.max_query_inserts,
                max_errors: row.max_errors,
                max_result_rows: row.max_result_rows,
                max_result_bytes: row.max_result_bytes,
                max_read_rows: row.max_read_rows,
                max_read_bytes: row.max_read_bytes,
                max_execution_time: row.max_execution_time,
                max_written_bytes: row.max_written_bytes,
                max_failed_sequential_authentications: row.max_failed_sequential_authentications,
                max_queries_per_normalized_hash: row.max_queries_per_normalized_hash,
            });
        }
    }
    query::<QuotaRow>(
        client,
        source,
        "quotas",
        "SELECT name, storage, arrayMap(value -> toString(value), keys) AS keys, apply_to_all, apply_to_list, apply_to_except, ipv4_prefix_bits, ipv6_prefix_bits FROM system.quotas WHERE storage != 'users_xml' ORDER BY name FORMAT JSONEachRow",
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| Quota {
                limits: limits.remove(&row.name).unwrap_or_default(),
                name: row.name,
                storage: row.storage,
                keys: row.keys,
                target: AccessTarget {
                    all: row.apply_to_all != 0,
                    include: row.apply_to_list,
                    except: row.apply_to_except,
                },
                ipv4_prefix_bits: row.ipv4_prefix_bits,
                ipv6_prefix_bits: row.ipv6_prefix_bits,
            })
            .collect()
    })
}

#[derive(Deserialize)]
struct SettingsProfileRow {
    name: String,
    storage: String,
    apply_to_all: u8,
    apply_to_list: Vec<String>,
    apply_to_except: Vec<String>,
}

#[derive(Deserialize)]
struct SettingsProfileElementRow {
    profile_name: Option<String>,
    index: u64,
    setting_name: Option<String>,
    value: Option<String>,
    min: Option<String>,
    max: Option<String>,
    writability: Option<String>,
    inherit_profile: Option<String>,
}

fn load_settings_profiles(
    client: &Client,
    source: &ClickHouseSource,
) -> Result<Vec<SettingsProfile>, IntrospectionError> {
    if !has_system_table(client, source, "settings_profiles")? {
        return Ok(Vec::new());
    }
    let mut elements = BTreeMap::<String, Vec<SettingsProfileElement>>::new();
    if has_system_table(client, source, "settings_profile_elements")? {
        for row in query::<SettingsProfileElementRow>(
            client,
            source,
            "settings profile elements",
            "SELECT profile_name, index, setting_name, value, min, max, toString(writability) AS writability, inherit_profile FROM system.settings_profile_elements WHERE profile_name IN (SELECT name FROM system.settings_profiles WHERE storage != 'users_xml') ORDER BY profile_name, index FORMAT JSONEachRow",
        )? {
            let Some(profile_name) = row.profile_name else {
                continue;
            };
            elements
                .entry(profile_name)
                .or_default()
                .push(SettingsProfileElement {
                    index: row.index,
                    setting_name: row.setting_name,
                    value: row.value,
                    minimum: row.min,
                    maximum: row.max,
                    writability: row.writability,
                    inherited_profile: row.inherit_profile,
                });
        }
    }
    query::<SettingsProfileRow>(
        client,
        source,
        "settings profiles",
        "SELECT name, storage, apply_to_all, apply_to_list, apply_to_except FROM system.settings_profiles WHERE storage != 'users_xml' ORDER BY name FORMAT JSONEachRow",
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| SettingsProfile {
                elements: elements.remove(&row.name).unwrap_or_default(),
                name: row.name,
                storage: row.storage,
                target: AccessTarget {
                    all: row.apply_to_all != 0,
                    include: row.apply_to_list,
                    except: row.apply_to_except,
                },
            })
            .collect()
    })
}

#[derive(Deserialize)]
struct NamedCollectionRow {
    name: String,
    keys: Vec<String>,
    source: String,
    create_query: String,
}

fn load_named_collections(
    client: &Client,
    source: &ClickHouseSource,
) -> Result<Vec<NamedCollection>, IntrospectionError> {
    if !has_system_table(client, source, "named_collections")? {
        return Ok(Vec::new());
    }
    // Never select `collection`: its raw values can contain secrets. In 26.6,
    // `create_query` is server-redacted to `[HIDDEN]` before query results are
    // returned; the sentinel-backed contract test guards that boundary.
    query::<NamedCollectionRow>(
        client,
        source,
        "named collections",
        "SELECT name, arraySort(mapKeys(collection)) AS keys, source, create_query FROM system.named_collections ORDER BY name FORMAT JSONEachRow",
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| NamedCollection {
                name: row.name,
                entries: crate::ddl::named_collection_entries(row.keys, &row.create_query),
                definition: optional(row.create_query),
                source: row.source,
            })
            .collect()
    })
}

#[derive(Deserialize)]
struct ResourceRow {
    name: String,
    read_disks: Vec<String>,
    write_disks: Vec<String>,
    unit: String,
    create_query: String,
}

fn load_resources(
    client: &Client,
    source: &ClickHouseSource,
) -> Result<Vec<Resource>, IntrospectionError> {
    if !has_system_table(client, source, "resources")? {
        return Ok(Vec::new());
    }
    query::<ResourceRow>(
        client,
        source,
        "resources",
        "SELECT name, read_disks, write_disks, unit, create_query FROM system.resources ORDER BY name FORMAT JSONEachRow",
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| Resource {
                name: row.name,
                operations: crate::ddl::resource_operations(&row.create_query),
                read_disks: row.read_disks,
                write_disks: row.write_disks,
                unit: row.unit,
                definition: row.create_query,
            })
            .collect()
    })
}

#[derive(Deserialize)]
struct WorkloadRow {
    name: String,
    parent: String,
    create_query: String,
}

fn load_workloads(
    client: &Client,
    source: &ClickHouseSource,
) -> Result<Vec<Workload>, IntrospectionError> {
    if !has_system_table(client, source, "workloads")? {
        return Ok(Vec::new());
    }
    query::<WorkloadRow>(
        client,
        source,
        "workloads",
        "SELECT name, parent, create_query FROM system.workloads ORDER BY name FORMAT JSONEachRow",
    )
    .map(|rows| {
        rows.into_iter()
            .map(|row| Workload {
                name: row.name,
                parent: optional(row.parent),
                settings: crate::ddl::workload_settings(&row.create_query),
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

fn closed_value<T>(
    source: &ClickHouseSource,
    operation: &'static str,
    field: &'static str,
    value: &str,
    decode: impl FnOnce(&str) -> Option<T>,
) -> Result<T, IntrospectionError> {
    decode(value).ok_or_else(|| IntrospectionError::UnknownClosedValue {
        source_id: source.id.clone(),
        operation,
        field,
        value: value.to_string(),
    })
}

fn constraint_kind(value: &str) -> Option<ConstraintKind> {
    match value {
        "CHECK" => Some(ConstraintKind::Check),
        "ASSUME" => Some(ConstraintKind::Assume),
        _ => None,
    }
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
    /// A documented closed server vocabulary returned an unsupported value.
    #[error(
        "unknown ClickHouse {operation} field `{field}` value `{value}` for source `{source_id}`"
    )]
    UnknownClosedValue {
        /// Stable source identity.
        source_id: SourceId,
        /// Catalog operation.
        operation: &'static str,
        /// Native catalog field.
        field: &'static str,
        /// Native value rejected at the acquisition boundary.
        value: String,
    },
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use dbmd_core::SourceId;

    use super::{
        closed_value, constraint_kind, database_filter, default_kind, dictionary_fields,
        dictionary_layout, optional, references, table_kind, ClickHouseSource,
    };
    use crate::{ColumnDefaultKind, ConstraintKind, TableKind};

    macro_rules! decoder_cases {
        ($($name:ident: $actual:expr => $expected:expr;)+) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!($actual, Some($expected));
                }
            )+
        };
    }

    decoder_cases! {
        decodes_absent_column_default: default_kind("") => ColumnDefaultKind::None;
        decodes_default_column_default: default_kind("DEFAULT") => ColumnDefaultKind::Default;
        decodes_materialized_column_default: default_kind("MATERIALIZED") => ColumnDefaultKind::Materialized;
        decodes_alias_column_default: default_kind("ALIAS") => ColumnDefaultKind::Alias;
        decodes_ephemeral_column_default: default_kind("EPHEMERAL") => ColumnDefaultKind::Ephemeral;
        decodes_check_constraint: constraint_kind("CHECK") => ConstraintKind::Check;
        decodes_assume_constraint: constraint_kind("ASSUME") => ConstraintKind::Assume;
    }

    #[test]
    fn rejects_unknown_column_default_kind() {
        assert_eq!(default_kind("GENERATED"), None);
    }

    #[test]
    fn rejects_unknown_constraint_kind() {
        assert_eq!(constraint_kind("UNIQUE"), None);
    }

    #[test]
    fn classifies_every_clickhouse_table_family() {
        for (engine, expected) in [
            ("View", TableKind::View),
            ("MaterializedView", TableKind::MaterializedView),
            ("LiveView", TableKind::LiveView),
            ("WindowView", TableKind::WindowView),
            ("Dictionary", TableKind::Dictionary),
            ("MergeTree", TableKind::Table),
        ] {
            assert_eq!(table_kind(engine), expected, "engine {engine}");
        }
    }

    #[test]
    fn pairs_only_complete_table_references_in_identity_order() {
        let values = references(
            vec![
                "warehouse".to_string(),
                "analytics".to_string(),
                "analytics".to_string(),
                "ignored".to_string(),
            ],
            vec![
                "events".to_string(),
                "users".to_string(),
                "events".to_string(),
            ],
        );

        assert_eq!(values.len(), 3);
        assert_eq!(values[0].database, "analytics");
        assert_eq!(values[0].table, "events");
        assert_eq!(values[1].database, "analytics");
        assert_eq!(values[1].table, "users");
        assert_eq!(values[2].database, "warehouse");
        assert_eq!(values[2].table, "events");
    }

    #[test]
    fn pairs_only_complete_dictionary_fields_in_catalog_order() {
        let values = dictionary_fields(
            vec!["id".to_string(), "ignored".to_string()],
            vec!["UInt64".to_string()],
        );

        assert_eq!(values.len(), 1);
        assert_eq!(values[0].name, "id");
        assert_eq!(values[0].data_type, "UInt64");
    }

    #[test]
    fn extracts_simple_and_parameterized_dictionary_layouts() {
        assert_eq!(
            dictionary_layout("CREATE DICTIONARY d LAYOUT(HASHED())"),
            Some("HASHED".to_string())
        );
        assert_eq!(
            dictionary_layout("CREATE DICTIONARY d LAYOUT(COMPLEX_KEY_CACHE(SIZE_IN_CELLS 10))"),
            Some("COMPLEX_KEY_CACHE".to_string())
        );
    }

    #[test]
    fn rejects_absent_or_empty_dictionary_layouts() {
        assert_eq!(dictionary_layout("CREATE DICTIONARY d"), None);
        assert_eq!(dictionary_layout("CREATE DICTIONARY d LAYOUT()"), None);
    }

    #[test]
    fn escapes_configured_database_filter_literals() {
        assert_eq!(
            database_filter(Some("tenant\\'one"), "database"),
            "database = 'tenant\\\\\\'one'"
        );
    }

    #[test]
    fn excludes_every_clickhouse_system_namespace_by_default() {
        assert_eq!(
            database_filter(None, "database"),
            "database NOT IN ('system', 'information_schema', 'INFORMATION_SCHEMA')"
        );
    }

    #[test]
    fn normalizes_only_empty_optional_catalog_strings_to_absence() {
        assert_eq!(optional(String::new()), None);
        assert_eq!(optional(" ".to_string()), Some(" ".to_string()));
        assert_eq!(optional("value".to_string()), Some("value".to_string()));
    }

    #[test]
    fn rejects_unknown_closed_values_with_source_operation_field_and_native_value() {
        let source = ClickHouseSource::new(
            SourceId::from_str("analytics").expect("test source ID should be valid"),
            "http://database.invalid",
        );

        let error = closed_value(
            &source,
            "columns",
            "default_kind",
            "FUTURE_KIND",
            default_kind,
        )
        .expect_err("unknown closed value must not leak as an opaque string");
        let message = error.to_string();

        assert!(message.contains("source `analytics`"));
        assert!(message.contains("columns"));
        assert!(message.contains("default_kind"));
        assert!(message.contains("FUTURE_KIND"));
    }
}
