use std::{
    collections::BTreeMap,
    fmt,
    path::{Path, PathBuf},
};

use dbmd_core::{SourceId, SourceSnapshot};
use duckdb::{
    types::{Type as SqlType, Value},
    AccessMode, Config, Connection, Row,
};
use thiserror::Error;

use super::{
    Catalog, Column, Constraint, ConstraintKind, Database, Extension, ExtensionInstallMode,
    Function, FunctionKind, FunctionStability, Index, Schema, Secret, Sequence, Snapshot, Table,
    Type, View,
};

#[derive(Clone, PartialEq, Eq)]
pub struct DuckDbSource {
    id: SourceId,
    display_name: Option<String>,
    path: PathBuf,
    attachments: Vec<DuckDbAttachment>,
    secret_directory: Option<PathBuf>,
    extension_directory: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DuckDbAttachment {
    name: String,
    path: PathBuf,
    read_only: bool,
}

impl DuckDbSource {
    pub fn new(id: SourceId, path: impl Into<PathBuf>) -> Result<Self, DuckDbSourceError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(DuckDbSourceError::EmptyPath);
        }
        Ok(Self {
            id,
            display_name: None,
            path,
            attachments: Vec::new(),
            secret_directory: None,
            extension_directory: None,
        })
    }
    #[must_use]
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }
    #[must_use]
    pub fn id(&self) -> &SourceId {
        &self.id
    }
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Adds a database that will be attached before catalog introspection.
    ///
    /// # Errors
    ///
    /// Returns [`DuckDbSourceError`] when the name or path is empty, the name
    /// is reserved or duplicated, or either value contains a NUL byte.
    pub fn with_attached_database(
        mut self,
        name: impl Into<String>,
        path: impl Into<PathBuf>,
        read_only: bool,
    ) -> Result<Self, DuckDbSourceError> {
        let name = name.into();
        let path = path.into();
        validate_attachment(&name, &path, &self.attachments)?;
        self.attachments.push(DuckDbAttachment {
            name,
            path,
            read_only,
        });
        Ok(self)
    }

    /// Selects the directory from which DuckDB loads persistent secrets.
    ///
    /// The directory only controls discovery. Introspection reads and exposes
    /// non-sensitive metadata; credential material is never queried.
    ///
    /// # Errors
    ///
    /// Returns [`DuckDbSourceError`] when the directory path is empty or
    /// contains a NUL byte.
    pub fn with_secret_directory(
        mut self,
        path: impl Into<PathBuf>,
    ) -> Result<Self, DuckDbSourceError> {
        let path = path.into();
        validate_secret_directory(&path)?;
        self.secret_directory = Some(path);
        Ok(self)
    }

    /// Selects the directory used to discover DuckDB extensions.
    ///
    /// # Errors
    ///
    /// Returns [`DuckDbSourceError`] when the directory path is empty or
    /// contains a NUL byte.
    pub fn with_extension_directory(
        mut self,
        path: impl Into<PathBuf>,
    ) -> Result<Self, DuckDbSourceError> {
        let path = path.into();
        validate_extension_directory(&path)?;
        self.extension_directory = Some(path);
        Ok(self)
    }
}
impl fmt::Debug for DuckDbSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DuckDbSource")
            .field("id", &self.id)
            .field("display_name", &self.display_name)
            .field("path", &self.path)
            .field("attachments", &self.attachments)
            .field(
                "secret_directory",
                &self.secret_directory.as_ref().map(|_| "[CONFIGURED]"),
            )
            .field(
                "extension_directory",
                &self.extension_directory.as_ref().map(|_| "[CONFIGURED]"),
            )
            .finish()
    }
}

pub fn introspect(source: &DuckDbSource) -> Result<Snapshot, IntrospectionError> {
    let mut config = Config::default()
        .access_mode(AccessMode::ReadOnly)
        .map_err(|error| IntrospectionError::Connect {
            source_id: source.id.clone(),
            source: error,
        })?;
    if let Some(secret_directory) = &source.secret_directory {
        config = config
            .with("secret_directory", secret_directory.to_string_lossy())
            .map_err(|error| IntrospectionError::Connect {
                source_id: source.id.clone(),
                source: error,
            })?;
    }
    if let Some(extension_directory) = &source.extension_directory {
        config = config
            .with("extension_directory", extension_directory.to_string_lossy())
            .map_err(|error| IntrospectionError::Connect {
                source_id: source.id.clone(),
                source: error,
            })?;
    }
    let connection = Connection::open_with_flags(&source.path, config).map_err(|error| {
        IntrospectionError::Connect {
            source_id: source.id.clone(),
            source: error,
        }
    })?;
    for attachment in &source.attachments {
        attach_database(&connection, source, attachment)?;
    }
    let mut tables = load_tables(&connection, source)?;
    attach_columns(&connection, source, &mut tables)?;
    attach_constraints(&connection, source, &mut tables)?;
    attach_indexes(&connection, source, &mut tables)?;
    let mut types = load_types(&connection, source)?;
    attach_type_definitions(&connection, source, &mut types)?;
    let catalog = Catalog {
        databases: load_databases(&connection, source)?,
        schemas: load_schemas(&connection, source)?,
        tables,
        views: load_views(&connection, source)?,
        sequences: load_sequences(&connection, source)?,
        types,
        functions: load_functions(&connection, source)?,
        extensions: load_extensions(&connection, source)?,
        secrets: load_secrets(&connection, source)?,
    };
    let snapshot = SourceSnapshot::new(source.id.clone(), catalog);
    Ok(match &source.display_name {
        Some(name) => snapshot.with_display_name(name),
        None => snapshot,
    })
}

fn attach_database(
    connection: &Connection,
    source: &DuckDbSource,
    attachment: &DuckDbAttachment,
) -> Result<(), IntrospectionError> {
    let path = attachment.path.to_string_lossy().replace('\'', "''");
    let name = attachment.name.replace('"', "\"\"");
    let access = if attachment.read_only {
        " (READ_ONLY)"
    } else {
        ""
    };
    connection
        .execute_batch(&format!("ATTACH '{path}' AS \"{name}\"{access}"))
        .map_err(|error| IntrospectionError::Attach {
            source_id: source.id.clone(),
            database: attachment.name.clone(),
            source: error,
        })
}

fn load_databases(
    connection: &Connection,
    source: &DuckDbSource,
) -> Result<Vec<Database>, IntrospectionError> {
    map(connection, source, "databases", "SELECT database_name, path, comment, type, readonly, tags, map_from_entries(list_filter(map_entries(options), lambda entry: lower(entry.key) IN ('block_size', 'row_group_size', 'storage_version', 'compression', 'type', 'encryption_cipher', 'recovery_mode'))) FROM duckdb_databases() WHERE NOT internal ORDER BY database_name", |row| Ok(Database { name: row.get(0)?, path: row.get(1)?, comment: row.get(2)?, database_type: row.get(3)?, readonly: row.get(4)?, tags: string_map(row, 5)?, options: string_map(row, 6)? }))
}
fn load_schemas(
    connection: &Connection,
    source: &DuckDbSource,
) -> Result<Vec<Schema>, IntrospectionError> {
    map(connection, source, "schemas", "SELECT database_name, schema_name, comment, tags FROM duckdb_schemas() WHERE NOT internal ORDER BY database_name, schema_name", |row| Ok(Schema { database: row.get(0)?, name: row.get(1)?, comment: row.get(2)?, tags: string_map(row, 3)? }))
}
fn load_tables(
    connection: &Connection,
    source: &DuckDbSource,
) -> Result<Vec<Table>, IntrospectionError> {
    map(connection, source, "tables", "SELECT database_name, schema_name, table_name, comment, temporary, sql, tags FROM duckdb_tables() WHERE NOT internal ORDER BY database_name, schema_name, table_name", |row| Ok(Table { database: row.get(0)?, schema: row.get(1)?, name: row.get(2)?, comment: row.get(3)?, temporary: row.get(4)?, definition: row.get::<_, Option<String>>(5)?.unwrap_or_default(), tags: string_map(row, 6)?, columns: Vec::new(), constraints: Vec::new(), indexes: Vec::new() }))
}

fn attach_columns(
    connection: &Connection,
    source: &DuckDbSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let values = map(connection, source, "columns", "SELECT database_name, schema_name, table_name, column_name, column_index, data_type, numeric_precision, numeric_precision_radix, numeric_scale, is_nullable, column_default, comment FROM duckdb_columns() WHERE NOT internal ORDER BY database_name, schema_name, table_name, column_index", |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, Column { name: row.get(3)?, position: row.get(4)?, data_type: row.get(5)?, numeric_precision: row.get(6)?, numeric_precision_radix: row.get(7)?, numeric_scale: row.get(8)?, nullable: row.get(9)?, default: row.get(10)?, comment: row.get(11)?, generated_expression: None })))?;
    let mut grouped = BTreeMap::new();
    for (database, schema, table, value) in values {
        grouped
            .entry((database, schema, table))
            .or_insert_with(Vec::new)
            .push(value);
    }
    attach(tables, grouped, |table, value| table.columns = value);
    for table in tables {
        for column in &mut table.columns {
            if is_generated_column(&table.definition, &column.name) {
                column.generated_expression = column.default.take();
            }
        }
    }
    Ok(())
}

fn is_generated_column(definition: &str, column: &str) -> bool {
    column_declarations(definition).any(|declaration| {
        let Some((name, remainder)) = declaration_name(declaration) else {
            return false;
        };
        name.eq_ignore_ascii_case(column)
            && remainder
                .to_ascii_uppercase()
                .contains("GENERATED ALWAYS AS")
    })
}

fn column_declarations(definition: &str) -> impl Iterator<Item = &str> {
    let body = definition
        .find('(')
        .and_then(|start| definition.rfind(')').map(|end| &definition[start + 1..end]))
        .unwrap_or("");
    let mut declarations = Vec::new();
    let mut start = 0;
    let mut depth = 0_u32;
    let mut quote = None;
    let mut previous = '\0';
    for (index, character) in body.char_indices() {
        if let Some(delimiter) = quote {
            if character == delimiter && previous != delimiter {
                quote = None;
            } else if character == delimiter && previous == delimiter {
                previous = '\0';
                continue;
            }
        } else {
            match character {
                '\'' | '"' => quote = Some(character),
                '(' => depth += 1,
                ')' => depth = depth.saturating_sub(1),
                ',' if depth == 0 => {
                    declarations.push(body[start..index].trim());
                    start = index + 1;
                }
                _ => {}
            }
        }
        previous = character;
    }
    declarations.push(body[start..].trim());
    declarations.into_iter()
}

fn declaration_name(declaration: &str) -> Option<(String, &str)> {
    let declaration = declaration.trim();
    if let Some(quoted) = declaration.strip_prefix('"') {
        let mut name = String::new();
        let mut characters = quoted.char_indices().peekable();
        while let Some((index, character)) = characters.next() {
            if character == '"' {
                if characters.peek().is_some_and(|(_, next)| *next == '"') {
                    name.push('"');
                    characters.next();
                    continue;
                }
                return Some((name, &quoted[index + 1..]));
            }
            name.push(character);
        }
        None
    } else {
        let end = declaration.find(char::is_whitespace)?;
        Some((declaration[..end].to_string(), &declaration[end..]))
    }
}

fn attach_constraints(
    connection: &Connection,
    source: &DuckDbSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let values = map(connection, source, "constraints", "SELECT database_name, schema_name, table_name, constraint_index, constraint_name, constraint_type, constraint_text, expression, array_to_string(constraint_column_names, chr(31)), referenced_table, array_to_string(referenced_column_names, chr(31)) FROM duckdb_constraints() ORDER BY database_name, schema_name, table_name, constraint_index", |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, Constraint { catalog_index: row.get(3)?, name: row.get(4)?, kind: semantic(row.get(5)?, "constraint_type", constraint_kind)?, text: row.get(6)?, expression: row.get(7)?, columns: split(row.get::<_, Option<String>>(8)?), referenced_table: row.get(9)?, referenced_columns: split(row.get::<_, Option<String>>(10)?) })))?;
    let mut grouped = BTreeMap::new();
    for (database, schema, table, value) in values {
        grouped
            .entry((database, schema, table))
            .or_insert_with(Vec::new)
            .push(value);
    }
    attach(tables, grouped, |table, value| table.constraints = value);
    Ok(())
}

fn attach_indexes(
    connection: &Connection,
    source: &DuckDbSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let values = map(connection, source, "indexes", "SELECT database_name, schema_name, table_name, index_name, is_unique, is_primary, expressions, comment, sql, tags FROM duckdb_indexes() ORDER BY database_name, schema_name, table_name, index_name", |row| {
        let definition = row.get::<_, Option<String>>(8)?.unwrap_or_default();
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, Index { name: row.get(3)?, index_type: index_type(&definition), unique: row.get(4)?, primary: row.get(5)?, expressions: row.get(6)?, comment: row.get(7)?, definition, tags: string_map(row, 9)? }))
    })?;
    let mut grouped = BTreeMap::new();
    for (database, schema, table, value) in values {
        grouped
            .entry((database, schema, table))
            .or_insert_with(Vec::new)
            .push(value);
    }
    attach(tables, grouped, |table, value| table.indexes = value);
    Ok(())
}

fn load_views(
    connection: &Connection,
    source: &DuckDbSource,
) -> Result<Vec<View>, IntrospectionError> {
    map(connection, source, "views", "SELECT database_name, schema_name, view_name, comment, temporary, sql, tags FROM duckdb_views() WHERE NOT internal ORDER BY database_name, schema_name, view_name", |row| Ok(View { database: row.get(0)?, schema: row.get(1)?, name: row.get(2)?, comment: row.get(3)?, temporary: row.get(4)?, definition: row.get::<_, Option<String>>(5)?.unwrap_or_default(), tags: string_map(row, 6)? }))
}
fn load_sequences(
    connection: &Connection,
    source: &DuckDbSource,
) -> Result<Vec<Sequence>, IntrospectionError> {
    map(connection, source, "sequences", "SELECT database_name, schema_name, sequence_name, comment, temporary, start_value, min_value, max_value, increment_by, cycle, sql, tags FROM duckdb_sequences() ORDER BY database_name, schema_name, sequence_name", |row| Ok(Sequence { database: row.get(0)?, schema: row.get(1)?, name: row.get(2)?, comment: row.get(3)?, temporary: row.get(4)?, start: row.get(5)?, minimum: row.get(6)?, maximum: row.get(7)?, increment: row.get(8)?, cycle: row.get(9)?, definition: row.get(10)?, tags: string_map(row, 11)? }))
}
fn load_types(
    connection: &Connection,
    source: &DuckDbSource,
) -> Result<Vec<Type>, IntrospectionError> {
    map(connection, source, "types", "SELECT database_name, schema_name, type_name, logical_type, type_size, type_category, array_to_string(labels, chr(31)), comment, tags FROM duckdb_types() WHERE NOT internal ORDER BY database_name, schema_name, type_name", |row| {
        let logical_type: String = row.get(3)?;
        Ok(Type { database: row.get(0)?, schema: row.get(1)?, name: row.get(2)?, definition: logical_type.clone(), logical_type, size: row.get(4)?, category: nonempty(row.get(5)?), labels: split(row.get(6)?), comment: row.get(7)?, tags: string_map(row, 8)? })
    })
}

fn attach_type_definitions(
    connection: &Connection,
    source: &DuckDbSource,
    types: &mut [Type],
) -> Result<(), IntrospectionError> {
    for value in types {
        let qualified_name = [&value.database, &value.schema, &value.name]
            .map(|part| format!("\"{}\"", part.replace('"', "\"\"")))
            .join(".");
        value.definition = connection
            .query_row(
                &format!("SELECT typeof(NULL::{qualified_name})"),
                [],
                |row| row.get(0),
            )
            .map_err(|error| query_error(source, "type definitions", error))?;
    }
    Ok(())
}
fn load_functions(
    connection: &Connection,
    source: &DuckDbSource,
) -> Result<Vec<Function>, IntrospectionError> {
    map(connection, source, "functions", "SELECT database_name, schema_name, function_name, function_type, description, comment, return_type, array_to_string(parameters, chr(31)), array_to_string(parameter_types, chr(31)), varargs, macro_definition, has_side_effects, stability, tags FROM duckdb_functions() WHERE NOT internal ORDER BY database_name, schema_name, function_name, function_type", |row| Ok(Function { database: row.get::<_, Option<String>>(0)?.unwrap_or_default(), schema: row.get::<_, Option<String>>(1)?.unwrap_or_default(), name: row.get(2)?, kind: semantic(row.get(3)?, "function_type", function_kind)?, description: row.get(4)?, comment: row.get(5)?, return_type: row.get(6)?, parameters: split(row.get(7)?), parameter_types: split(row.get(8)?), varargs: row.get(9)?, definition: row.get(10)?, side_effects: row.get(11)?, stability: optional_semantic(row.get(12)?, "stability", function_stability)?, tags: string_map(row, 13)? }))
}
fn load_extensions(
    connection: &Connection,
    source: &DuckDbSource,
) -> Result<Vec<Extension>, IntrospectionError> {
    map(connection, source, "extensions", "SELECT extension_name, loaded, installed, extension_version, description, array_to_string(aliases, chr(31)), install_mode, installed_from FROM duckdb_extensions() WHERE loaded OR installed ORDER BY extension_name", |row| Ok(Extension { name: row.get(0)?, loaded: row.get(1)?, installed: row.get(2)?, version: nonempty(row.get(3)?), description: row.get(4)?, aliases: split(row.get(5)?), install_mode: optional_semantic(nonempty(row.get(6)?), "install_mode", extension_install_mode)?, installed_from: nonempty(row.get(7)?) }))
}

fn load_secrets(
    connection: &Connection,
    source: &DuckDbSource,
) -> Result<Vec<Secret>, IntrospectionError> {
    // Intentionally do not select `secret_string`. Credential material must
    // never cross the acquisition boundary into dbmd's catalog or errors.
    map(connection, source, "secrets", "SELECT name, type, provider, persistent, storage, array_to_string(scope, chr(31)) FROM duckdb_secrets() ORDER BY name", |row| Ok(Secret { name: row.get(0)?, secret_type: row.get(1)?, provider: row.get(2)?, persistent: row.get(3)?, storage: row.get(4)?, scope: split(row.get(5)?) }))
}

fn map<T>(
    connection: &Connection,
    source: &DuckDbSource,
    operation: &'static str,
    sql: &str,
    mut convert: impl FnMut(&Row<'_>) -> duckdb::Result<T>,
) -> Result<Vec<T>, IntrospectionError> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| query_error(source, operation, error))?;
    let values = statement
        .query_map([], |row| convert(row))
        .map_err(|error| query_error(source, operation, error))?;
    values
        .collect::<duckdb::Result<Vec<_>>>()
        .map_err(|error| query_error(source, operation, error))
}
fn attach<T>(
    tables: &mut [Table],
    mut grouped: BTreeMap<(String, String, String), Vec<T>>,
    mut assign: impl FnMut(&mut Table, Vec<T>),
) {
    for table in tables {
        if let Some(values) = grouped.remove(&(
            table.database.clone(),
            table.schema.clone(),
            table.name.clone(),
        )) {
            assign(table, values);
        }
    }
}
fn split(value: Option<String>) -> Vec<String> {
    value
        .filter(|value| !value.is_empty())
        .map_or_else(Vec::new, |value| {
            value.split('\u{1f}').map(str::to_string).collect()
        })
}

fn string_map(row: &Row<'_>, index: usize) -> duckdb::Result<BTreeMap<String, String>> {
    let Value::Map(values) = row.get::<_, Value>(index)? else {
        return Err(duckdb::Error::FromSqlConversionFailure(
            index,
            SqlType::Map(Box::new(SqlType::Text), Box::new(SqlType::Text)),
            Box::new(InvalidStringMap),
        ));
    };
    values
        .iter()
        .map(|(key, value)| match (key, value) {
            (Value::Text(key), Value::Text(value)) => Ok((key.clone(), value.clone())),
            _ => Err(duckdb::Error::FromSqlConversionFailure(
                index,
                SqlType::Map(Box::new(SqlType::Text), Box::new(SqlType::Text)),
                Box::new(InvalidStringMap),
            )),
        })
        .collect()
}

#[derive(Debug, Error)]
#[error("DuckDB metadata map was not MAP(VARCHAR, VARCHAR)")]
struct InvalidStringMap;

fn nonempty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.is_empty())
}

fn semantic<T>(
    value: String,
    field: &'static str,
    decode: impl FnOnce(&str) -> Option<T>,
) -> duckdb::Result<T> {
    decode(&value).ok_or_else(|| {
        duckdb::Error::FromSqlConversionFailure(
            usize::MAX,
            SqlType::Text,
            Box::new(UnknownSemanticValue { field, value }),
        )
    })
}

fn optional_semantic<T>(
    value: Option<String>,
    field: &'static str,
    decode: impl FnOnce(&str) -> Option<T>,
) -> duckdb::Result<Option<T>> {
    value
        .map(|value| semantic(value, field, decode))
        .transpose()
}

#[derive(Debug, Error)]
#[error("unknown closed DuckDB `{field}` value `{value}`")]
struct UnknownSemanticValue {
    field: &'static str,
    value: String,
}

fn constraint_kind(value: &str) -> Option<ConstraintKind> {
    match value {
        "CHECK" => Some(ConstraintKind::Check),
        "FOREIGN KEY" => Some(ConstraintKind::ForeignKey),
        "PRIMARY KEY" => Some(ConstraintKind::PrimaryKey),
        "NOT NULL" => Some(ConstraintKind::NotNull),
        "UNIQUE" => Some(ConstraintKind::Unique),
        _ => None,
    }
}

fn function_kind(value: &str) -> Option<FunctionKind> {
    match value {
        "table" => Some(FunctionKind::Table),
        "scalar" => Some(FunctionKind::Scalar),
        "aggregate" => Some(FunctionKind::Aggregate),
        "pragma" => Some(FunctionKind::Pragma),
        "macro" => Some(FunctionKind::Macro),
        "table_macro" => Some(FunctionKind::TableMacro),
        _ => None,
    }
}

fn function_stability(value: &str) -> Option<FunctionStability> {
    match value {
        "CONSISTENT" => Some(FunctionStability::Consistent),
        "VOLATILE" => Some(FunctionStability::Volatile),
        "CONSISTENT_WITHIN_QUERY" => Some(FunctionStability::ConsistentWithinQuery),
        _ => None,
    }
}

fn extension_install_mode(value: &str) -> Option<ExtensionInstallMode> {
    match value {
        "UNKNOWN" => Some(ExtensionInstallMode::Unknown),
        "REPOSITORY" => Some(ExtensionInstallMode::Repository),
        "CUSTOM_PATH" => Some(ExtensionInstallMode::CustomPath),
        "STATICALLY_LINKED" => Some(ExtensionInstallMode::StaticallyLinked),
        "NOT_INSTALLED" => Some(ExtensionInstallMode::NotInstalled),
        _ => None,
    }
}

fn index_type(definition: &str) -> String {
    let uppercase = definition.to_ascii_uppercase();
    uppercase
        .split_whitespace()
        .skip_while(|part| *part != "USING")
        .nth(1)
        .map(|part| part.trim_end_matches(['(', ';']).to_string())
        .unwrap_or_else(|| "ART".to_string())
}

fn validate_attachment(
    name: &str,
    path: &Path,
    attachments: &[DuckDbAttachment],
) -> Result<(), DuckDbSourceError> {
    if name.is_empty() {
        return Err(DuckDbSourceError::EmptyAttachmentName);
    }
    if name.contains('\0') {
        return Err(DuckDbSourceError::NulAttachmentName);
    }
    if matches!(name.to_ascii_lowercase().as_str(), "system" | "temp") {
        return Err(DuckDbSourceError::ReservedAttachmentName(name.to_string()));
    }
    if attachments.iter().any(|attachment| attachment.name == name) {
        return Err(DuckDbSourceError::DuplicateAttachmentName(name.to_string()));
    }
    if path.as_os_str().is_empty() {
        return Err(DuckDbSourceError::EmptyAttachmentPath(name.to_string()));
    }
    if path.to_string_lossy().contains('\0') {
        return Err(DuckDbSourceError::NulAttachmentPath(name.to_string()));
    }
    Ok(())
}

fn validate_secret_directory(path: &Path) -> Result<(), DuckDbSourceError> {
    if path.as_os_str().is_empty() {
        return Err(DuckDbSourceError::EmptySecretDirectory);
    }
    if path.to_string_lossy().contains('\0') {
        return Err(DuckDbSourceError::NulSecretDirectory);
    }
    Ok(())
}

fn validate_extension_directory(path: &Path) -> Result<(), DuckDbSourceError> {
    if path.as_os_str().is_empty() {
        return Err(DuckDbSourceError::EmptyExtensionDirectory);
    }
    if path.to_string_lossy().contains('\0') {
        return Err(DuckDbSourceError::NulExtensionDirectory);
    }
    Ok(())
}
fn query_error(
    source: &DuckDbSource,
    operation: &'static str,
    error: duckdb::Error,
) -> IntrospectionError {
    IntrospectionError::Query {
        source_id: source.id.clone(),
        operation,
        source: error,
    }
}

#[derive(Debug, Error)]
pub enum DuckDbSourceError {
    #[error("DuckDB path cannot be empty")]
    EmptyPath,
    #[error("DuckDB attachment name cannot be empty")]
    EmptyAttachmentName,
    #[error("DuckDB attachment name cannot contain a NUL byte")]
    NulAttachmentName,
    #[error("DuckDB attachment name `{0}` is reserved")]
    ReservedAttachmentName(String),
    #[error("DuckDB attachment name `{0}` is duplicated")]
    DuplicateAttachmentName(String),
    #[error("DuckDB attachment `{0}` path cannot be empty")]
    EmptyAttachmentPath(String),
    #[error("DuckDB attachment `{0}` path cannot contain a NUL byte")]
    NulAttachmentPath(String),
    #[error("DuckDB secret directory cannot be empty")]
    EmptySecretDirectory,
    #[error("DuckDB secret directory cannot contain a NUL byte")]
    NulSecretDirectory,
    #[error("DuckDB extension directory cannot be empty")]
    EmptyExtensionDirectory,
    #[error("DuckDB extension directory cannot contain a NUL byte")]
    NulExtensionDirectory,
}
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IntrospectionError {
    #[error("could not open DuckDB source `{source_id}`")]
    Connect {
        source_id: SourceId,
        #[source]
        source: duckdb::Error,
    },
    #[error("could not attach DuckDB database `{database}` for source `{source_id}`")]
    Attach {
        source_id: SourceId,
        database: String,
        #[source]
        source: duckdb::Error,
    },
    #[error("could not introspect {operation} for DuckDB source `{source_id}`")]
    Query {
        source_id: SourceId,
        operation: &'static str,
        #[source]
        source: duckdb::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::{
        constraint_kind, extension_install_mode, function_kind, function_stability, index_type,
        is_generated_column,
    };
    use crate::{ConstraintKind, ExtensionInstallMode, FunctionKind, FunctionStability};

    #[test]
    fn identifies_only_the_matching_generated_column_declaration() {
        let definition = r#"CREATE TABLE items (
            source VARCHAR,
            note VARCHAR DEFAULT 'generated_col',
            "generated_col" VARCHAR GENERATED ALWAYS AS (concat(source, ',', note)) VIRTUAL
        )"#;
        assert!(!is_generated_column(definition, "source"));
        assert!(!is_generated_column(definition, "note"));
        assert!(is_generated_column(definition, "generated_col"));
    }

    #[test]
    fn derives_default_and_explicit_index_types_from_normalized_sql() {
        assert_eq!(index_type("CREATE INDEX idx ON items(value);"), "ART");
        assert_eq!(
            index_type("CREATE INDEX idx ON items USING HNSW (embedding);"),
            "HNSW"
        );
    }

    #[test]
    fn decodes_closed_metadata_values_semantically() {
        assert_eq!(
            constraint_kind("FOREIGN KEY"),
            Some(ConstraintKind::ForeignKey)
        );
        assert_eq!(function_kind("table_macro"), Some(FunctionKind::TableMacro));
        assert_eq!(
            function_stability("CONSISTENT_WITHIN_QUERY"),
            Some(FunctionStability::ConsistentWithinQuery)
        );
        assert_eq!(
            extension_install_mode("STATICALLY_LINKED"),
            Some(ExtensionInstallMode::StaticallyLinked)
        );
    }

    #[test]
    fn rejects_unknown_closed_metadata_values() {
        assert_eq!(constraint_kind("EXCLUSION"), None);
        assert_eq!(function_kind("window"), None);
        assert_eq!(function_stability("IMMUTABLE"), None);
        assert_eq!(extension_install_mode("REMOTE"), None);
    }
}
