use std::{collections::BTreeMap, fmt};

use dbmd_core::{SourceId, SourceSnapshot};
use mysql::{prelude::Queryable, Conn, Opts, Row};
use thiserror::Error;

use super::{
    Catalog, Column, Constraint, ConstraintKind, Event, Index, IndexTerm, Parameter, Partition,
    Routine, Schema, Snapshot, Table, Trigger, View,
};

#[derive(Clone, PartialEq, Eq)]
pub struct MysqlSource {
    id: SourceId,
    display_name: Option<String>,
    connection_url: String,
    schema: Option<String>,
}

impl MysqlSource {
    #[must_use]
    pub fn new(id: SourceId, connection_url: impl Into<String>) -> Self {
        Self {
            id,
            display_name: None,
            connection_url: connection_url.into(),
            schema: None,
        }
    }

    #[must_use]
    pub fn with_schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
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
}

impl fmt::Debug for MysqlSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MysqlSource")
            .field("id", &self.id)
            .field("display_name", &self.display_name)
            .field("connection_url", &"[REDACTED]")
            .field("schema", &self.schema)
            .finish()
    }
}

pub fn introspect(source: &MysqlSource) -> Result<Snapshot, IntrospectionError> {
    let options =
        Opts::from_url(&source.connection_url).map_err(|error| IntrospectionError::Url {
            source_id: source.id.clone(),
            source: error,
        })?;
    let mut connection = Conn::new(options).map_err(|error| IntrospectionError::Connect {
        source_id: source.id.clone(),
        source: error,
    })?;
    let mut tables = load_tables(&mut connection, source)?;
    attach_columns(&mut connection, source, &mut tables)?;
    attach_constraints(&mut connection, source, &mut tables)?;
    attach_indexes(&mut connection, source, &mut tables)?;
    attach_partitions(&mut connection, source, &mut tables)?;
    attach_definitions(&mut connection, source, &mut tables)?;
    let mut routines = load_routines(&mut connection, source)?;
    attach_parameters(&mut connection, source, &mut routines)?;
    let catalog = Catalog {
        schemas: load_schemas(&mut connection, source)?,
        tables,
        views: load_views(&mut connection, source)?,
        routines,
        triggers: load_triggers(&mut connection, source)?,
        events: load_events(&mut connection, source)?,
    };
    let snapshot = SourceSnapshot::new(source.id.clone(), catalog);
    Ok(match &source.display_name {
        Some(name) => snapshot.with_display_name(name),
        None => snapshot,
    })
}

fn load_schemas(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Schema>, IntrospectionError> {
    let values = rows(connection, source, "schemas", &format!(
        "SELECT SCHEMA_NAME, DEFAULT_CHARACTER_SET_NAME, DEFAULT_COLLATION_NAME FROM information_schema.SCHEMATA WHERE {} ORDER BY SCHEMA_NAME",
        schema_filter(source, "SCHEMA_NAME")
    ))?;
    values
        .into_iter()
        .map(|row| {
            Ok(Schema {
                name: row.required("SCHEMA_NAME")?,
                default_character_set: row.required("DEFAULT_CHARACTER_SET_NAME")?,
                default_collation: row.required("DEFAULT_COLLATION_NAME")?,
            })
        })
        .collect()
}

fn load_tables(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Table>, IntrospectionError> {
    let values = rows(connection, source, "tables", &format!(
        "SELECT TABLE_SCHEMA, TABLE_NAME, ENGINE, ROW_FORMAT, TABLE_COLLATION, TABLE_COMMENT, CREATE_OPTIONS FROM information_schema.TABLES WHERE TABLE_TYPE = 'BASE TABLE' AND {} ORDER BY TABLE_SCHEMA, TABLE_NAME",
        schema_filter(source, "TABLE_SCHEMA")
    ))?;
    values
        .into_iter()
        .map(|row| {
            Ok(Table {
                schema: row.required("TABLE_SCHEMA")?,
                name: row.required("TABLE_NAME")?,
                engine: row.optional("ENGINE")?,
                row_format: row.optional("ROW_FORMAT")?,
                collation: row.optional("TABLE_COLLATION")?,
                comment: nonempty(row.optional("TABLE_COMMENT")?),
                create_options: nonempty(row.optional("CREATE_OPTIONS")?),
                columns: Vec::new(),
                constraints: Vec::new(),
                indexes: Vec::new(),
                partitions: Vec::new(),
                definition: String::new(),
            })
        })
        .collect()
}

fn attach_columns(
    connection: &mut Conn,
    source: &MysqlSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let values = rows(connection, source, "columns", &format!(
        "SELECT TABLE_SCHEMA, TABLE_NAME, COLUMN_NAME, ORDINAL_POSITION, DATA_TYPE, COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT, EXTRA, GENERATION_EXPRESSION, CHARACTER_SET_NAME, COLLATION_NAME, COLUMN_COMMENT FROM information_schema.COLUMNS WHERE {} ORDER BY TABLE_SCHEMA, TABLE_NAME, ORDINAL_POSITION",
        schema_filter(source, "TABLE_SCHEMA")
    ))?;
    let mut grouped = BTreeMap::new();
    for row in values {
        let extra = row.required::<String>("EXTRA")?;
        grouped
            .entry((row.required("TABLE_SCHEMA")?, row.required("TABLE_NAME")?))
            .or_insert_with(Vec::new)
            .push(Column {
                name: row.required("COLUMN_NAME")?,
                position: row.required("ORDINAL_POSITION")?,
                data_type: row.required("DATA_TYPE")?,
                column_type: row.required("COLUMN_TYPE")?,
                nullable: row.required::<String>("IS_NULLABLE")? == "YES",
                default: row.optional("COLUMN_DEFAULT")?,
                visible: Some(!extra.to_ascii_uppercase().contains("INVISIBLE")),
                extra,
                generation_expression: nonempty(row.optional("GENERATION_EXPRESSION")?),
                character_set: row.optional("CHARACTER_SET_NAME")?,
                collation: row.optional("COLLATION_NAME")?,
                comment: nonempty(row.optional("COLUMN_COMMENT")?),
            });
    }
    attach(tables, grouped, |table, value| table.columns = value);
    Ok(())
}

fn attach_constraints(
    connection: &mut Conn,
    source: &MysqlSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let values = rows(connection, source, "constraints", &format!(
        "SELECT tc.TABLE_SCHEMA, tc.TABLE_NAME, tc.CONSTRAINT_NAME, tc.CONSTRAINT_TYPE, kcu.COLUMN_NAME, kcu.ORDINAL_POSITION, kcu.REFERENCED_TABLE_SCHEMA, kcu.REFERENCED_TABLE_NAME, kcu.REFERENCED_COLUMN_NAME, rc.MATCH_OPTION, rc.UPDATE_RULE, rc.DELETE_RULE, cc.CHECK_CLAUSE, tc.ENFORCED FROM information_schema.TABLE_CONSTRAINTS tc LEFT JOIN information_schema.KEY_COLUMN_USAGE kcu ON kcu.CONSTRAINT_SCHEMA=tc.CONSTRAINT_SCHEMA AND kcu.TABLE_NAME=tc.TABLE_NAME AND kcu.CONSTRAINT_NAME=tc.CONSTRAINT_NAME LEFT JOIN information_schema.REFERENTIAL_CONSTRAINTS rc ON rc.CONSTRAINT_SCHEMA=tc.CONSTRAINT_SCHEMA AND rc.TABLE_NAME=tc.TABLE_NAME AND rc.CONSTRAINT_NAME=tc.CONSTRAINT_NAME LEFT JOIN information_schema.CHECK_CONSTRAINTS cc ON cc.CONSTRAINT_SCHEMA=tc.CONSTRAINT_SCHEMA AND cc.CONSTRAINT_NAME=tc.CONSTRAINT_NAME WHERE {} ORDER BY tc.TABLE_SCHEMA, tc.TABLE_NAME, tc.CONSTRAINT_NAME, kcu.ORDINAL_POSITION",
        schema_filter(source, "tc.TABLE_SCHEMA")
    ))?;
    let mut constraints = BTreeMap::<(String, String, String), Constraint>::new();
    for row in values {
        let key = (
            row.required("TABLE_SCHEMA")?,
            row.required("TABLE_NAME")?,
            row.required("CONSTRAINT_NAME")?,
        );
        let kind = constraint_kind(&row.required::<String>("CONSTRAINT_TYPE")?);
        let referenced_schema = row.optional("REFERENCED_TABLE_SCHEMA")?;
        let referenced_table = row.optional("REFERENCED_TABLE_NAME")?;
        let match_option = row.optional("MATCH_OPTION")?;
        let update_rule = row.optional("UPDATE_RULE")?;
        let delete_rule = row.optional("DELETE_RULE")?;
        let expression = row.optional("CHECK_CLAUSE")?;
        let enforced = row
            .optional::<String>("ENFORCED")?
            .map(|value| value == "YES");
        let item = constraints
            .entry(key.clone())
            .or_insert_with(|| Constraint {
                name: key.2.clone(),
                kind,
                columns: Vec::new(),
                referenced_schema,
                referenced_table,
                referenced_columns: Vec::new(),
                match_option,
                update_rule,
                delete_rule,
                expression,
                enforced,
            });
        if let Some(column) = row.optional("COLUMN_NAME")? {
            item.columns.push(column);
        }
        if let Some(column) = row.optional("REFERENCED_COLUMN_NAME")? {
            item.referenced_columns.push(column);
        }
    }
    let mut grouped = BTreeMap::new();
    for ((schema, table, _), constraint) in constraints {
        grouped
            .entry((schema, table))
            .or_insert_with(Vec::new)
            .push(constraint);
    }
    attach(tables, grouped, |table, value| table.constraints = value);
    Ok(())
}

fn attach_indexes(
    connection: &mut Conn,
    source: &MysqlSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let values = rows(connection, source, "indexes", &format!(
        "SELECT TABLE_SCHEMA, TABLE_NAME, INDEX_NAME, NON_UNIQUE, SEQ_IN_INDEX, COLUMN_NAME, COLLATION, SUB_PART, INDEX_TYPE, COMMENT, IS_VISIBLE, EXPRESSION FROM information_schema.STATISTICS WHERE {} ORDER BY TABLE_SCHEMA, TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX",
        schema_filter(source, "TABLE_SCHEMA")
    ))?;
    let mut indexes = BTreeMap::<(String, String, String), Index>::new();
    for row in values {
        let key = (
            row.required("TABLE_SCHEMA")?,
            row.required("TABLE_NAME")?,
            row.required("INDEX_NAME")?,
        );
        let unique = row.required::<u64>("NON_UNIQUE")? == 0;
        let index_type = row.required("INDEX_TYPE")?;
        let visible = row
            .optional::<String>("IS_VISIBLE")?
            .map(|value| value == "YES");
        let comment = nonempty(row.optional("COMMENT")?);
        let item = indexes.entry(key.clone()).or_insert_with(|| Index {
            name: key.2.clone(),
            unique,
            index_type,
            visible,
            comment,
            terms: Vec::new(),
        });
        item.terms.push(IndexTerm {
            position: row.required("SEQ_IN_INDEX")?,
            column: row.optional("COLUMN_NAME")?,
            expression: row.optional("EXPRESSION")?,
            prefix_length: row.optional("SUB_PART")?,
            descending: row
                .optional::<String>("COLLATION")?
                .map(|value| value == "D"),
        });
    }
    let mut grouped = BTreeMap::new();
    for ((schema, table, _), index) in indexes {
        grouped
            .entry((schema, table))
            .or_insert_with(Vec::new)
            .push(index);
    }
    attach(tables, grouped, |table, value| table.indexes = value);
    Ok(())
}

fn attach_partitions(
    connection: &mut Conn,
    source: &MysqlSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let values = rows(connection, source, "partitions", &format!(
        "SELECT TABLE_SCHEMA, TABLE_NAME, PARTITION_NAME, SUBPARTITION_NAME, PARTITION_METHOD, PARTITION_EXPRESSION, PARTITION_DESCRIPTION, PARTITION_ORDINAL_POSITION FROM information_schema.PARTITIONS WHERE PARTITION_NAME IS NOT NULL AND {} ORDER BY TABLE_SCHEMA, TABLE_NAME, PARTITION_ORDINAL_POSITION, SUBPARTITION_ORDINAL_POSITION",
        schema_filter(source, "TABLE_SCHEMA")
    ))?;
    let mut grouped = BTreeMap::new();
    for row in values {
        grouped
            .entry((row.required("TABLE_SCHEMA")?, row.required("TABLE_NAME")?))
            .or_insert_with(Vec::new)
            .push(Partition {
                name: row.required("PARTITION_NAME")?,
                subpartition: row.optional("SUBPARTITION_NAME")?,
                method: row.optional("PARTITION_METHOD")?,
                expression: row.optional("PARTITION_EXPRESSION")?,
                description: row.optional("PARTITION_DESCRIPTION")?,
                ordinal: row.required("PARTITION_ORDINAL_POSITION")?,
            });
    }
    attach(tables, grouped, |table, value| table.partitions = value);
    Ok(())
}

fn attach_definitions(
    connection: &mut Conn,
    source: &MysqlSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    for table in tables {
        let sql = format!(
            "SHOW CREATE TABLE `{}`.`{}`",
            escape_identifier(&table.schema),
            escape_identifier(&table.name)
        );
        let row = rows(connection, source, "table definitions", &sql)?
            .into_iter()
            .next()
            .ok_or_else(|| IntrospectionError::MissingDefinition {
                source_id: source.id.clone(),
                object: table.qualified_name(),
            })?;
        let definition: String = row.required_at(1, "create statement")?;
        table.definition = stable_create_statement(&definition);
    }
    Ok(())
}

fn load_views(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<View>, IntrospectionError> {
    let values = rows(connection, source, "views", &format!(
        "SELECT TABLE_SCHEMA, TABLE_NAME, VIEW_DEFINITION, CHECK_OPTION, IS_UPDATABLE, SECURITY_TYPE, DEFINER, CHARACTER_SET_CLIENT, COLLATION_CONNECTION FROM information_schema.VIEWS WHERE {} ORDER BY TABLE_SCHEMA, TABLE_NAME",
        schema_filter(source, "TABLE_SCHEMA")
    ))?;
    values
        .into_iter()
        .map(|row| {
            let schema: String = row.required("TABLE_SCHEMA")?;
            let name: String = row.required("TABLE_NAME")?;
            let sql = format!(
                "SHOW CREATE VIEW `{}`.`{}`",
                escape_identifier(&schema),
                escape_identifier(&name)
            );
            let create = rows(connection, source, "view definitions", &sql)?
                .into_iter()
                .next()
                .map(|value| value.required_at(1, "create statement"))
                .transpose()?
                .ok_or_else(|| IntrospectionError::MissingDefinition {
                    source_id: source.id.clone(),
                    object: format!("{schema}.{name}"),
                })?;
            Ok(View {
                schema,
                name,
                definition: row.required("VIEW_DEFINITION")?,
                check_option: row.required("CHECK_OPTION")?,
                updatable: row.required::<String>("IS_UPDATABLE")? == "YES",
                security_type: row.required("SECURITY_TYPE")?,
                definer: row.required("DEFINER")?,
                character_set: row.required("CHARACTER_SET_CLIENT")?,
                collation: row.required("COLLATION_CONNECTION")?,
                create_statement: create,
            })
        })
        .collect()
}

fn load_routines(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Routine>, IntrospectionError> {
    let values = rows(connection, source, "routines", &format!(
        "SELECT ROUTINE_SCHEMA, ROUTINE_NAME, ROUTINE_TYPE, DTD_IDENTIFIER, ROUTINE_BODY, ROUTINE_DEFINITION, IS_DETERMINISTIC, SQL_DATA_ACCESS, SECURITY_TYPE, DEFINER, ROUTINE_COMMENT FROM information_schema.ROUTINES WHERE {} ORDER BY ROUTINE_SCHEMA, ROUTINE_NAME, ROUTINE_TYPE",
        schema_filter(source, "ROUTINE_SCHEMA")
    ))?;
    values
        .into_iter()
        .map(|row| {
            Ok(Routine {
                schema: row.required("ROUTINE_SCHEMA")?,
                name: row.required("ROUTINE_NAME")?,
                kind: row.required("ROUTINE_TYPE")?,
                return_type: row.optional("DTD_IDENTIFIER")?,
                body: row.required("ROUTINE_BODY")?,
                definition: row.optional("ROUTINE_DEFINITION")?,
                deterministic: row.required::<String>("IS_DETERMINISTIC")? == "YES",
                sql_data_access: row.required("SQL_DATA_ACCESS")?,
                security_type: row.required("SECURITY_TYPE")?,
                definer: row.required("DEFINER")?,
                comment: nonempty(row.optional("ROUTINE_COMMENT")?),
                parameters: Vec::new(),
            })
        })
        .collect()
}

fn attach_parameters(
    connection: &mut Conn,
    source: &MysqlSource,
    routines: &mut [Routine],
) -> Result<(), IntrospectionError> {
    let values = rows(connection, source, "routine parameters", &format!(
        "SELECT SPECIFIC_SCHEMA, SPECIFIC_NAME, ORDINAL_POSITION, PARAMETER_MODE, PARAMETER_NAME, DATA_TYPE, DTD_IDENTIFIER FROM information_schema.PARAMETERS WHERE {} ORDER BY SPECIFIC_SCHEMA, SPECIFIC_NAME, ORDINAL_POSITION",
        schema_filter(source, "SPECIFIC_SCHEMA")
    ))?;
    let mut grouped = BTreeMap::new();
    for row in values {
        grouped
            .entry((
                row.required("SPECIFIC_SCHEMA")?,
                row.required("SPECIFIC_NAME")?,
            ))
            .or_insert_with(Vec::new)
            .push(Parameter {
                position: row.required("ORDINAL_POSITION")?,
                mode: row.optional("PARAMETER_MODE")?,
                name: row.optional("PARAMETER_NAME")?,
                data_type: row.required("DATA_TYPE")?,
                dtd_identifier: row.required("DTD_IDENTIFIER")?,
            });
    }
    for routine in routines {
        if let Some(values) = grouped.remove(&(routine.schema.clone(), routine.name.clone())) {
            routine.parameters = values;
        }
    }
    Ok(())
}

fn load_triggers(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Trigger>, IntrospectionError> {
    let values = rows(connection, source, "triggers", &format!(
        "SELECT TRIGGER_SCHEMA, TRIGGER_NAME, EVENT_OBJECT_TABLE, EVENT_MANIPULATION, ACTION_TIMING, ACTION_ORIENTATION, ACTION_STATEMENT, ACTION_ORDER, SQL_MODE, DEFINER, CHARACTER_SET_CLIENT, COLLATION_CONNECTION FROM information_schema.TRIGGERS WHERE {} ORDER BY TRIGGER_SCHEMA, TRIGGER_NAME",
        schema_filter(source, "TRIGGER_SCHEMA")
    ))?;
    values
        .into_iter()
        .map(|row| {
            Ok(Trigger {
                schema: row.required("TRIGGER_SCHEMA")?,
                name: row.required("TRIGGER_NAME")?,
                table: row.required("EVENT_OBJECT_TABLE")?,
                event: row.required("EVENT_MANIPULATION")?,
                timing: row.required("ACTION_TIMING")?,
                orientation: row.required("ACTION_ORIENTATION")?,
                statement: row.required("ACTION_STATEMENT")?,
                action_order: row.required("ACTION_ORDER")?,
                sql_mode: row.required("SQL_MODE")?,
                definer: row.required("DEFINER")?,
                character_set: row.required("CHARACTER_SET_CLIENT")?,
                collation: row.required("COLLATION_CONNECTION")?,
            })
        })
        .collect()
}

fn load_events(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Event>, IntrospectionError> {
    let values = rows(connection, source, "events", &format!(
        "SELECT EVENT_SCHEMA, EVENT_NAME, DEFINER, TIME_ZONE, EVENT_TYPE, CAST(EXECUTE_AT AS CHAR) AS EXECUTE_AT, INTERVAL_VALUE, INTERVAL_FIELD, CAST(STARTS AS CHAR) AS STARTS, CAST(ENDS AS CHAR) AS ENDS, STATUS, ON_COMPLETION, EVENT_COMMENT, EVENT_DEFINITION FROM information_schema.EVENTS WHERE {} ORDER BY EVENT_SCHEMA, EVENT_NAME",
        schema_filter(source, "EVENT_SCHEMA")
    ))?;
    values
        .into_iter()
        .map(|row| {
            Ok(Event {
                schema: row.required("EVENT_SCHEMA")?,
                name: row.required("EVENT_NAME")?,
                definer: row.required("DEFINER")?,
                time_zone: row.required("TIME_ZONE")?,
                event_type: row.required("EVENT_TYPE")?,
                execute_at: row.optional("EXECUTE_AT")?,
                interval_value: row.optional("INTERVAL_VALUE")?,
                interval_field: row.optional("INTERVAL_FIELD")?,
                starts: row.optional("STARTS")?,
                ends: row.optional("ENDS")?,
                status: row.required("STATUS")?,
                on_completion: row.required("ON_COMPLETION")?,
                comment: nonempty(row.optional("EVENT_COMMENT")?),
                definition: row.required("EVENT_DEFINITION")?,
            })
        })
        .collect()
}

fn rows(
    connection: &mut Conn,
    source: &MysqlSource,
    operation: &'static str,
    sql: &str,
) -> Result<Vec<CatalogRow>, IntrospectionError> {
    connection
        .query::<Row, _>(sql)
        .map(|rows| {
            rows.into_iter()
                .map(|row| CatalogRow {
                    row,
                    source_id: source.id.clone(),
                    operation,
                })
                .collect()
        })
        .map_err(|error| IntrospectionError::Query {
            source_id: source.id.clone(),
            operation,
            source: error,
        })
}

fn schema_filter(source: &MysqlSource, column: &str) -> String {
    source.schema.as_ref().map_or_else(
        || format!("{column} NOT IN ('information_schema','mysql','performance_schema','sys')"),
        |schema| format!("{column} = '{}'", escape_literal(schema)),
    )
}

fn attach<T>(
    tables: &mut [Table],
    mut grouped: BTreeMap<(String, String), Vec<T>>,
    mut assign: impl FnMut(&mut Table, Vec<T>),
) {
    for table in tables {
        if let Some(values) = grouped.remove(&(table.schema.clone(), table.name.clone())) {
            assign(table, values);
        }
    }
}

fn constraint_kind(value: &str) -> ConstraintKind {
    match value {
        "PRIMARY KEY" => ConstraintKind::PrimaryKey,
        "UNIQUE" => ConstraintKind::Unique,
        "FOREIGN KEY" => ConstraintKind::ForeignKey,
        "CHECK" => ConstraintKind::Check,
        _ => ConstraintKind::Unknown,
    }
}
struct CatalogRow {
    row: Row,
    source_id: SourceId,
    operation: &'static str,
}

impl CatalogRow {
    fn required<T: mysql::prelude::FromValue>(&self, name: &str) -> Result<T, IntrospectionError> {
        self.convert(name, self.row.get_opt(name))
    }

    fn required_at<T: mysql::prelude::FromValue>(
        &self,
        index: usize,
        name: &str,
    ) -> Result<T, IntrospectionError> {
        self.convert(name, self.row.get_opt(index))
    }

    fn optional<T: mysql::prelude::FromValue>(
        &self,
        name: &str,
    ) -> Result<Option<T>, IntrospectionError> {
        self.convert(name, self.row.get_opt::<Option<T>, _>(name))
    }

    fn convert<T>(
        &self,
        column: &str,
        value: Option<Result<T, mysql::FromValueError>>,
    ) -> Result<T, IntrospectionError> {
        match value {
            Some(Ok(value)) => Ok(value),
            Some(Err(error)) => Err(IntrospectionError::Decode {
                source_id: self.source_id.clone(),
                operation: self.operation,
                column: column.to_string(),
                reason: error.to_string(),
            }),
            None => Err(IntrospectionError::Decode {
                source_id: self.source_id.clone(),
                operation: self.operation,
                column: column.to_string(),
                reason: "column is missing".to_string(),
            }),
        }
    }
}
fn nonempty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.is_empty())
}
fn escape_literal(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "''")
}
fn escape_identifier(value: &str) -> String {
    value.replace('`', "``")
}

fn stable_create_statement(definition: &str) -> String {
    let uppercase = definition.to_ascii_uppercase();
    let Some(start) = uppercase.find(" AUTO_INCREMENT=") else {
        return definition.to_string();
    };
    let value_start = start + " AUTO_INCREMENT=".len();
    let value_end = uppercase[value_start..]
        .find(|character: char| !character.is_ascii_digit())
        .map_or(uppercase.len(), |offset| value_start + offset);
    format!("{}{}", &definition[..start], &definition[value_end..])
}

#[cfg(test)]
mod tests {
    use super::stable_create_statement;

    #[test]
    fn removes_only_the_volatile_auto_increment_counter() {
        assert_eq!(
            stable_create_statement(
                "CREATE TABLE `items` (`id` bigint NOT NULL AUTO_INCREMENT) ENGINE=InnoDB AUTO_INCREMENT=42 DEFAULT CHARSET=utf8mb4"
            ),
            "CREATE TABLE `items` (`id` bigint NOT NULL AUTO_INCREMENT) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"
        );
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IntrospectionError {
    #[error("invalid MySQL URL for source `{source_id}`")]
    Url {
        source_id: SourceId,
        #[source]
        source: mysql::UrlError,
    },
    #[error("could not connect to MySQL source `{source_id}`")]
    Connect {
        source_id: SourceId,
        #[source]
        source: mysql::Error,
    },
    #[error("could not introspect {operation} for MySQL source `{source_id}`")]
    Query {
        source_id: SourceId,
        operation: &'static str,
        #[source]
        source: mysql::Error,
    },
    #[error(
        "could not decode {operation} column `{column}` for MySQL source `{source_id}`: {reason}"
    )]
    Decode {
        source_id: SourceId,
        operation: &'static str,
        column: String,
        reason: String,
    },
    #[error("MySQL did not return a definition for `{object}` in source `{source_id}`")]
    MissingDefinition { source_id: SourceId, object: String },
}
