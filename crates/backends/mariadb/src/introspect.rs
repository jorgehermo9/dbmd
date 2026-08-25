use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use dbmd_core::{SourceId, SourceSnapshot};
use dbmd_relational::{ForeignKeyAction, ForeignKeyMatch, IndexSortOrder};
use mysql::{prelude::Queryable, Conn, Opts, Row};
use thiserror::Error;

use super::{
    Account, AccountKind, ApplicationTimePeriod, Catalog, CheckConstraintLevel, Column, Constraint,
    ConstraintKind, Event, GeneratedColumnStorage, Index, IndexTerm, LoadableFunction,
    LoadableFunctionKind, LoadableFunctionReturnType, Package, Parameter, ParameterMode, Partition,
    PartitionMethod, Plugin, PluginKind, PluginLicense, PluginLoadOption, PluginMaturity,
    PluginStatus, Privilege, PrivilegeObjectKind, RoleMembership, Routine, RoutineDataAccess,
    RoutineKind, ScheduledEventCompletion, ScheduledEventKind, ScheduledEventStatus,
    ScheduledIntervalUnit, Schema, Sequence, ServerDefinition, ServerOption, Snapshot, SqlSecurity,
    StoredProgramDefinition, SystemTimePeriod, Table, TlsRequirement, Trigger, TriggerEvent,
    TriggerOrientation, TriggerTiming, VectorIndexOptions, View, ViewAlgorithm, ViewCheckOption,
};

#[derive(Clone, PartialEq, Eq)]
pub struct MariaDbSource {
    id: SourceId,
    display_name: Option<String>,
    connection_url: String,
    schema: Option<String>,
    include_global_objects: bool,
}

impl MariaDbSource {
    #[must_use]
    pub fn new(id: SourceId, connection_url: impl Into<String>) -> Self {
        Self {
            id,
            display_name: None,
            connection_url: connection_url.into(),
            schema: None,
            include_global_objects: false,
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
    pub fn with_global_objects(mut self, include: bool) -> Self {
        self.include_global_objects = include;
        self
    }
    #[must_use]
    pub fn id(&self) -> &SourceId {
        &self.id
    }
}

impl fmt::Debug for MariaDbSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MariaDbSource")
            .field("id", &self.id)
            .field("display_name", &self.display_name)
            .field("connection_url", &"[REDACTED]")
            .field("schema", &self.schema)
            .field("include_global_objects", &self.include_global_objects)
            .finish()
    }
}

pub fn introspect(source: &MariaDbSource) -> Result<Snapshot, IntrospectionError> {
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
    attach_application_time_periods(&mut connection, source, &mut tables)?;
    attach_period_usage(&mut connection, source, &mut tables)?;
    attach_partitions(&mut connection, source, &mut tables)?;
    attach_definitions(&mut connection, source, &mut tables)?;
    let mut routines = load_routines(&mut connection, source)?;
    attach_parameters(&mut connection, source, &mut routines)?;
    let mut triggers = load_triggers(&mut connection, source)?;
    attach_trigger_update_columns(&mut connection, source, &mut triggers)?;
    let (servers, loadable_functions, plugins, accounts, role_memberships, privileges) =
        if source.include_global_objects {
            let mut servers = load_servers(&mut connection, source)?;
            attach_server_options(&mut connection, source, &mut servers)?;
            let mut accounts = load_accounts(&mut connection, source)?;
            attach_authentication_plugins(&mut connection, source, &mut accounts)?;
            let mut privileges = load_privileges(&mut connection, source)?;
            privileges.sort_by(|left, right| {
                (
                    &left.grantee,
                    left.object_kind.display_name(),
                    &left.schema,
                    &left.object,
                    &left.column,
                    &left.privilege,
                )
                    .cmp(&(
                        &right.grantee,
                        right.object_kind.display_name(),
                        &right.schema,
                        &right.object,
                        &right.column,
                        &right.privilege,
                    ))
            });
            privileges.dedup();
            (
                servers,
                load_loadable_functions(&mut connection, source)?,
                load_plugins(&mut connection, source)?,
                accounts,
                load_role_memberships(&mut connection, source)?,
                privileges,
            )
        } else {
            (
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
        };
    let catalog = Catalog {
        schemas: load_schemas(&mut connection, source)?,
        tables,
        views: load_views(&mut connection, source)?,
        sequences: load_sequences(&mut connection, source)?,
        routines,
        packages: load_packages(&mut connection, source)?,
        triggers,
        events: load_events(&mut connection, source)?,
        servers,
        loadable_functions,
        plugins,
        accounts,
        role_memberships,
        privileges,
    };
    let snapshot = SourceSnapshot::new(source.id.clone(), catalog);
    Ok(match &source.display_name {
        Some(name) => snapshot.with_display_name(name),
        None => snapshot,
    })
}

fn load_schemas(
    connection: &mut Conn,
    source: &MariaDbSource,
) -> Result<Vec<Schema>, IntrospectionError> {
    let values = rows(connection, source, "schemas", &format!("SELECT SCHEMA_NAME, DEFAULT_CHARACTER_SET_NAME, DEFAULT_COLLATION_NAME, SCHEMA_COMMENT FROM information_schema.SCHEMATA WHERE {} ORDER BY SCHEMA_NAME", schema_filter(source, "SCHEMA_NAME")))?;
    values
        .into_iter()
        .map(|row| {
            let name: String = row.required("SCHEMA_NAME")?;
            let sql = format!("SHOW CREATE DATABASE `{}`", escape_identifier(&name));
            let definition = rows(connection, source, "schema definitions", &sql)?
                .into_iter()
                .next()
                .map(|value| value.required_at(1, "create statement"))
                .transpose()?
                .ok_or_else(|| IntrospectionError::MissingDefinition {
                    source_id: source.id.clone(),
                    object: name.clone(),
                })?;
            Ok(Schema {
                name,
                default_character_set: row.required("DEFAULT_CHARACTER_SET_NAME")?,
                default_collation: row.required("DEFAULT_COLLATION_NAME")?,
                comment: nonempty(row.optional("SCHEMA_COMMENT")?),
                definition,
            })
        })
        .collect()
}

fn load_tables(
    connection: &mut Conn,
    source: &MariaDbSource,
) -> Result<Vec<Table>, IntrospectionError> {
    let values = rows(connection, source, "tables", &format!("SELECT TABLE_SCHEMA, TABLE_NAME, ENGINE, ROW_FORMAT, TABLE_COLLATION, TABLE_COMMENT, CREATE_OPTIONS FROM information_schema.TABLES WHERE TABLE_TYPE NOT IN ('VIEW', 'SEQUENCE') AND {} ORDER BY TABLE_SCHEMA, TABLE_NAME", schema_filter(source, "TABLE_SCHEMA")))?;
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
                system_versioned: false,
                system_time_period: None,
                application_time_periods: Vec::new(),
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
    source: &MariaDbSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let values = rows(connection, source, "columns", &format!("SELECT TABLE_SCHEMA, TABLE_NAME, COLUMN_NAME, ORDINAL_POSITION, DATA_TYPE, COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT, EXTRA, GENERATION_EXPRESSION, CHARACTER_SET_NAME, COLLATION_NAME, COLUMN_COMMENT, IS_SYSTEM_TIME_PERIOD_START, IS_SYSTEM_TIME_PERIOD_END FROM information_schema.COLUMNS WHERE {} ORDER BY TABLE_SCHEMA, TABLE_NAME, ORDINAL_POSITION", schema_filter(source, "TABLE_SCHEMA")))?;
    let mut grouped = BTreeMap::new();
    for row in values {
        let extra = row.required::<String>("EXTRA")?;
        let generation_expression = nonempty(row.optional("GENERATION_EXPRESSION")?);
        let generated_storage = generated_column_storage(&extra);
        if generation_expression.is_some() && generated_storage.is_none() {
            return Err(row.unknown_value("EXTRA", &extra));
        }
        grouped
            .entry((row.required("TABLE_SCHEMA")?, row.required("TABLE_NAME")?))
            .or_insert_with(Vec::new)
            .push(Column {
                name: row.required("COLUMN_NAME")?,
                position: row.required("ORDINAL_POSITION")?,
                data_type: row.required("DATA_TYPE")?,
                column_type: row.required("COLUMN_TYPE")?,
                nullable: row.semantic("IS_NULLABLE", yes_no)?,
                default: row.optional("COLUMN_DEFAULT")?,
                visible: !extra.to_ascii_uppercase().contains("INVISIBLE"),
                extra,
                generation_expression,
                generated_storage,
                character_set: row.optional("CHARACTER_SET_NAME")?,
                collation: row.optional("COLLATION_NAME")?,
                comment: nonempty(row.optional("COLUMN_COMMENT")?),
                system_time_period_start: row.semantic("IS_SYSTEM_TIME_PERIOD_START", yes_no)?,
                system_time_period_end: row.semantic("IS_SYSTEM_TIME_PERIOD_END", yes_no)?,
            });
    }
    attach(tables, grouped, |table, value| table.columns = value);
    Ok(())
}

fn attach_constraints(
    connection: &mut Conn,
    source: &MariaDbSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let values = rows(connection, source, "constraints", &format!("SELECT tc.TABLE_SCHEMA, tc.TABLE_NAME, tc.CONSTRAINT_NAME, tc.CONSTRAINT_TYPE, kcu.COLUMN_NAME, kcu.ORDINAL_POSITION, kcu.REFERENCED_TABLE_SCHEMA, kcu.REFERENCED_TABLE_NAME, kcu.REFERENCED_COLUMN_NAME, rc.MATCH_OPTION, rc.UPDATE_RULE, rc.DELETE_RULE, cc.CHECK_CLAUSE, cc.LEVEL AS CHECK_LEVEL FROM information_schema.TABLE_CONSTRAINTS tc LEFT JOIN information_schema.KEY_COLUMN_USAGE kcu ON kcu.CONSTRAINT_SCHEMA=tc.CONSTRAINT_SCHEMA AND kcu.TABLE_NAME=tc.TABLE_NAME AND kcu.CONSTRAINT_NAME=tc.CONSTRAINT_NAME LEFT JOIN information_schema.REFERENTIAL_CONSTRAINTS rc ON rc.CONSTRAINT_SCHEMA=tc.CONSTRAINT_SCHEMA AND rc.TABLE_NAME=tc.TABLE_NAME AND rc.CONSTRAINT_NAME=tc.CONSTRAINT_NAME LEFT JOIN information_schema.CHECK_CONSTRAINTS cc ON cc.CONSTRAINT_SCHEMA=tc.CONSTRAINT_SCHEMA AND cc.TABLE_NAME=tc.TABLE_NAME AND cc.CONSTRAINT_NAME=tc.CONSTRAINT_NAME WHERE {} ORDER BY tc.TABLE_SCHEMA, tc.TABLE_NAME, tc.CONSTRAINT_NAME, kcu.ORDINAL_POSITION", schema_filter(source, "tc.TABLE_SCHEMA")))?;
    let mut constraints = BTreeMap::<(String, String, String), Constraint>::new();
    for row in values {
        let key = (
            row.required("TABLE_SCHEMA")?,
            row.required("TABLE_NAME")?,
            row.required("CONSTRAINT_NAME")?,
        );
        let kind = row.semantic("CONSTRAINT_TYPE", constraint_kind)?;
        let referenced_schema = row.optional("REFERENCED_TABLE_SCHEMA")?;
        let referenced_table = row.optional("REFERENCED_TABLE_NAME")?;
        let match_type = optional_foreign_key_match(&row)?;
        let on_update = row.optional_semantic("UPDATE_RULE", foreign_key_action)?;
        let on_delete = row.optional_semantic("DELETE_RULE", foreign_key_action)?;
        let expression = row.optional("CHECK_CLAUSE")?;
        let check_level = row.optional_semantic("CHECK_LEVEL", check_constraint_level)?;
        let item = constraints
            .entry(key.clone())
            .or_insert_with(|| Constraint {
                name: key.2.clone(),
                kind,
                columns: Vec::new(),
                referenced_schema,
                referenced_table,
                referenced_columns: Vec::new(),
                match_type,
                on_update,
                on_delete,
                expression,
                check_level,
                period: None,
            });
        if let Some(column) = row.optional("COLUMN_NAME")? {
            item.columns.push(column);
        }
        if let Some(column) = row.optional("REFERENCED_COLUMN_NAME")? {
            item.referenced_columns.push(column);
        }
    }
    let mut grouped = BTreeMap::new();
    for ((schema, table, _), value) in constraints {
        grouped
            .entry((schema, table))
            .or_insert_with(Vec::new)
            .push(value);
    }
    attach(tables, grouped, |table, value| table.constraints = value);
    Ok(())
}

fn attach_indexes(
    connection: &mut Conn,
    source: &MariaDbSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let values = rows(connection, source, "indexes", &format!("SELECT TABLE_SCHEMA, TABLE_NAME, INDEX_NAME, NON_UNIQUE, SEQ_IN_INDEX, COLUMN_NAME, COLLATION, SUB_PART, INDEX_TYPE, COMMENT, INDEX_COMMENT, IGNORED FROM information_schema.STATISTICS WHERE {} ORDER BY TABLE_SCHEMA, TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX", schema_filter(source, "TABLE_SCHEMA")))?;
    let mut indexes = BTreeMap::<(String, String, String), Index>::new();
    for row in values {
        let key = (
            row.required("TABLE_SCHEMA")?,
            row.required("TABLE_NAME")?,
            row.required("INDEX_NAME")?,
        );
        let unique = !row.semantic_u64("NON_UNIQUE", binary_flag)?;
        let index_type = row.required("INDEX_TYPE")?;
        let ignored = row.optional_semantic("IGNORED", yes_no)?;
        let comment = nonempty(row.optional("INDEX_COMMENT")?);
        let catalog_comment = nonempty(row.optional("COMMENT")?);
        let item = indexes.entry(key.clone()).or_insert_with(|| Index {
            name: key.2.clone(),
            unique,
            index_type,
            ignored,
            comment,
            catalog_comment,
            period: None,
            vector_options: None,
            terms: Vec::new(),
        });
        item.terms.push(IndexTerm {
            position: row.required("SEQ_IN_INDEX")?,
            column: row.required("COLUMN_NAME")?,
            prefix_length: row.optional("SUB_PART")?,
            sort_order: row.optional_semantic("COLLATION", index_sort_order)?,
        });
    }
    let mut grouped = BTreeMap::new();
    for ((schema, table, _), value) in indexes {
        grouped
            .entry((schema, table))
            .or_insert_with(Vec::new)
            .push(value);
    }
    attach(tables, grouped, |table, value| table.indexes = value);
    Ok(())
}

fn attach_application_time_periods(
    connection: &mut Conn,
    source: &MariaDbSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let values = rows(
        connection,
        source,
        "application-time periods",
        &format!(
            "SELECT TABLE_SCHEMA, TABLE_NAME, PERIOD, START_COLUMN_NAME, END_COLUMN_NAME FROM information_schema.PERIODS WHERE UPPER(PERIOD) <> 'SYSTEM_TIME' AND {} ORDER BY TABLE_SCHEMA, TABLE_NAME, PERIOD",
            schema_filter(source, "TABLE_SCHEMA")
        ),
    )?;
    let mut grouped = BTreeMap::new();
    for row in values {
        grouped
            .entry((row.required("TABLE_SCHEMA")?, row.required("TABLE_NAME")?))
            .or_insert_with(Vec::new)
            .push(ApplicationTimePeriod {
                name: row.required("PERIOD")?,
                start_column: row.required("START_COLUMN_NAME")?,
                end_column: row.required("END_COLUMN_NAME")?,
            });
    }
    attach(tables, grouped, |table, value| {
        table.application_time_periods = value;
    });
    Ok(())
}

fn attach_period_usage(
    connection: &mut Conn,
    source: &MariaDbSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let values = rows(
        connection,
        source,
        "period key usage",
        &format!(
            "SELECT CONSTRAINT_SCHEMA, CONSTRAINT_NAME, TABLE_SCHEMA, TABLE_NAME, PERIOD_NAME FROM information_schema.KEY_PERIOD_USAGE WHERE {} ORDER BY TABLE_SCHEMA, TABLE_NAME, CONSTRAINT_NAME",
            schema_filter(source, "TABLE_SCHEMA")
        ),
    )?;
    let mut periods = BTreeMap::new();
    for row in values {
        let table_schema: String = row.required("TABLE_SCHEMA")?;
        let constraint_schema: String = row.required("CONSTRAINT_SCHEMA")?;
        if table_schema != constraint_schema {
            continue;
        }
        periods.insert(
            (
                table_schema,
                row.required("TABLE_NAME")?,
                row.required("CONSTRAINT_NAME")?,
            ),
            row.required("PERIOD_NAME")?,
        );
    }
    for table in tables {
        for constraint in &mut table.constraints {
            constraint.period = periods
                .get(&(
                    table.schema.clone(),
                    table.name.clone(),
                    constraint.name.clone(),
                ))
                .cloned();
        }
        for index in &mut table.indexes {
            index.period = periods
                .get(&(table.schema.clone(), table.name.clone(), index.name.clone()))
                .cloned();
        }
    }
    Ok(())
}

fn attach_partitions(
    connection: &mut Conn,
    source: &MariaDbSource,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let values = rows(connection, source, "partitions", &format!("SELECT TABLE_SCHEMA, TABLE_NAME, PARTITION_NAME, SUBPARTITION_NAME, PARTITION_METHOD, SUBPARTITION_METHOD, PARTITION_EXPRESSION, SUBPARTITION_EXPRESSION, PARTITION_DESCRIPTION, PARTITION_ORDINAL_POSITION, SUBPARTITION_ORDINAL_POSITION, TABLESPACE_NAME, PARTITION_COMMENT, NODEGROUP FROM information_schema.PARTITIONS WHERE PARTITION_NAME IS NOT NULL AND {} ORDER BY TABLE_SCHEMA, TABLE_NAME, PARTITION_ORDINAL_POSITION, SUBPARTITION_ORDINAL_POSITION", schema_filter(source, "TABLE_SCHEMA")))?;
    let mut grouped = BTreeMap::new();
    for row in values {
        grouped
            .entry((row.required("TABLE_SCHEMA")?, row.required("TABLE_NAME")?))
            .or_insert_with(Vec::new)
            .push(Partition {
                name: row.required("PARTITION_NAME")?,
                subpartition: row.optional("SUBPARTITION_NAME")?,
                method: row.optional_semantic("PARTITION_METHOD", partition_method)?,
                subpartition_method: row
                    .optional_semantic("SUBPARTITION_METHOD", partition_method)?,
                expression: row.optional("PARTITION_EXPRESSION")?,
                subpartition_expression: row.optional("SUBPARTITION_EXPRESSION")?,
                description: row.optional("PARTITION_DESCRIPTION")?,
                ordinal: row.required("PARTITION_ORDINAL_POSITION")?,
                subpartition_ordinal: row.optional("SUBPARTITION_ORDINAL_POSITION")?,
                tablespace: nonempty(row.optional("TABLESPACE_NAME")?),
                comment: nonempty(row.optional("PARTITION_COMMENT")?),
                nodegroup: nonempty(row.optional("NODEGROUP")?),
            });
    }
    attach(tables, grouped, |table, value| table.partitions = value);
    Ok(())
}

fn attach_definitions(
    connection: &mut Conn,
    source: &MariaDbSource,
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
        table.system_versioned = table
            .definition
            .to_ascii_uppercase()
            .contains("WITH SYSTEM VERSIONING");
        table.system_time_period = system_time_period(&table.definition);
        for index in &mut table.indexes {
            if index.index_type.eq_ignore_ascii_case("VECTOR") {
                index.vector_options = vector_index_options(&table.definition, &index.name);
            }
        }
    }
    Ok(())
}

fn load_views(
    connection: &mut Conn,
    source: &MariaDbSource,
) -> Result<Vec<View>, IntrospectionError> {
    let values = rows(connection, source, "views", &format!("SELECT TABLE_SCHEMA, TABLE_NAME, VIEW_DEFINITION, CHECK_OPTION, IS_UPDATABLE, SECURITY_TYPE, DEFINER, CHARACTER_SET_CLIENT, COLLATION_CONNECTION, ALGORITHM FROM information_schema.VIEWS WHERE {} ORDER BY TABLE_SCHEMA, TABLE_NAME", schema_filter(source, "TABLE_SCHEMA")))?;
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
            let create_statement: String = rows(connection, source, "view definitions", &sql)?
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
                check_option: row.semantic("CHECK_OPTION", view_check_option)?,
                updatable: row.semantic("IS_UPDATABLE", yes_no)?,
                security: row.semantic("SECURITY_TYPE", sql_security)?,
                definer: row.required("DEFINER")?,
                character_set: row.required("CHARACTER_SET_CLIENT")?,
                collation: row.required("COLLATION_CONNECTION")?,
                algorithm: row.semantic("ALGORITHM", view_algorithm)?,
                create_statement: stable_create_statement(&create_statement),
            })
        })
        .collect()
}

fn load_sequences(
    connection: &mut Conn,
    source: &MariaDbSource,
) -> Result<Vec<Sequence>, IntrospectionError> {
    let values = rows(connection, source, "sequences", &format!("SELECT s.SEQUENCE_SCHEMA, s.SEQUENCE_NAME, s.DATA_TYPE, s.NUMERIC_PRECISION, s.NUMERIC_PRECISION_RADIX, s.NUMERIC_SCALE, s.START_VALUE, s.MINIMUM_VALUE, s.MAXIMUM_VALUE, s.INCREMENT, s.CYCLE_OPTION, t.ENGINE, t.TABLE_COMMENT FROM information_schema.SEQUENCES s LEFT JOIN information_schema.TABLES t ON t.TABLE_SCHEMA=s.SEQUENCE_SCHEMA AND t.TABLE_NAME=s.SEQUENCE_NAME WHERE {} ORDER BY s.SEQUENCE_SCHEMA, s.SEQUENCE_NAME", schema_filter(source, "s.SEQUENCE_SCHEMA")))?;
    values
        .into_iter()
        .map(|row| {
            let schema: String = row.required("SEQUENCE_SCHEMA")?;
            let name: String = row.required("SEQUENCE_NAME")?;
            let sql = format!(
                "SHOW CREATE SEQUENCE `{}`.`{}`",
                escape_identifier(&schema),
                escape_identifier(&name)
            );
            let definition: String = rows(connection, source, "sequence definitions", &sql)?
                .into_iter()
                .next()
                .map(|value| value.required_at(1, "create statement"))
                .transpose()?
                .ok_or_else(|| IntrospectionError::MissingDefinition {
                    source_id: source.id.clone(),
                    object: format!("{schema}.{name}"),
                })?;
            Ok(Sequence {
                schema,
                name,
                data_type: row.required("DATA_TYPE")?,
                numeric_precision: row.required("NUMERIC_PRECISION")?,
                numeric_precision_radix: row.required("NUMERIC_PRECISION_RADIX")?,
                numeric_scale: row.required("NUMERIC_SCALE")?,
                start_value: row.required("START_VALUE")?,
                minimum_value: row.required("MINIMUM_VALUE")?,
                maximum_value: row.required("MAXIMUM_VALUE")?,
                increment: row.required("INCREMENT")?,
                cache: sequence_cache(&definition),
                cycle: row.semantic_u64("CYCLE_OPTION", binary_flag)?,
                engine: row.optional("ENGINE")?,
                comment: nonempty(row.optional("TABLE_COMMENT")?),
                definition,
            })
        })
        .collect()
}

fn sequence_cache(definition: &str) -> Option<u64> {
    let mut tokens = definition.split_ascii_whitespace();
    while let Some(token) = tokens.next() {
        let token = token.trim_matches(|character: char| character == ',' || character == ';');
        if token.eq_ignore_ascii_case("nocache") {
            return Some(0);
        }
        if token.eq_ignore_ascii_case("cache") {
            return tokens.next()?.trim_matches(',').parse().ok();
        }
    }
    None
}

fn load_routines(
    connection: &mut Conn,
    source: &MariaDbSource,
) -> Result<Vec<Routine>, IntrospectionError> {
    let values = rows(connection, source, "routines", &format!("SELECT ROUTINE_SCHEMA, ROUTINE_NAME, ROUTINE_TYPE, DTD_IDENTIFIER, ROUTINE_DEFINITION, IS_DETERMINISTIC, SQL_DATA_ACCESS, SECURITY_TYPE, DEFINER, ROUTINE_COMMENT, SQL_MODE, CHARACTER_SET_CLIENT, COLLATION_CONNECTION, DATABASE_COLLATION FROM information_schema.ROUTINES WHERE ROUTINE_TYPE IN ('FUNCTION', 'PROCEDURE') AND {} ORDER BY ROUTINE_SCHEMA, ROUTINE_NAME, ROUTINE_TYPE", schema_filter(source, "ROUTINE_SCHEMA")))?;
    values
        .into_iter()
        .map(|row| {
            let schema: String = row.required("ROUTINE_SCHEMA")?;
            let name: String = row.required("ROUTINE_NAME")?;
            let native_kind: String = row.required("ROUTINE_TYPE")?;
            let kind = semantic_value(&row, "ROUTINE_TYPE", &native_kind, routine_kind)?;
            let sql = format!(
                "SHOW CREATE {native_kind} `{}`.`{}`",
                escape_identifier(&schema),
                escape_identifier(&name)
            );
            let create_statement: String = rows(connection, source, "routine definitions", &sql)?
                .into_iter()
                .next()
                .map(|value| value.required_at(2, "create statement"))
                .transpose()?
                .ok_or_else(|| IntrospectionError::MissingDefinition {
                    source_id: source.id.clone(),
                    object: format!("{schema}.{name}"),
                })?;
            Ok(Routine {
                schema,
                name,
                kind,
                return_type: row.optional("DTD_IDENTIFIER")?,
                definition: row.optional("ROUTINE_DEFINITION")?,
                deterministic: row.semantic("IS_DETERMINISTIC", yes_no)?,
                data_access: row.semantic("SQL_DATA_ACCESS", routine_data_access)?,
                security: row.semantic("SECURITY_TYPE", sql_security)?,
                definer: row.required("DEFINER")?,
                comment: nonempty(row.optional("ROUTINE_COMMENT")?),
                parameters: Vec::new(),
                sql_mode: row.required("SQL_MODE")?,
                character_set_client: row.required("CHARACTER_SET_CLIENT")?,
                collation_connection: row.required("COLLATION_CONNECTION")?,
                database_collation: row.required("DATABASE_COLLATION")?,
                create_statement: stable_create_statement(&create_statement),
            })
        })
        .collect()
}

fn load_packages(
    connection: &mut Conn,
    source: &MariaDbSource,
) -> Result<Vec<Package>, IntrospectionError> {
    let body_rows = rows(connection, source, "package bodies", &format!("SELECT ROUTINE_SCHEMA, ROUTINE_NAME FROM information_schema.ROUTINES WHERE ROUTINE_TYPE = 'PACKAGE BODY' AND {} ORDER BY ROUTINE_SCHEMA, ROUTINE_NAME", schema_filter(source, "ROUTINE_SCHEMA")))?;
    let mut bodies = BTreeSet::new();
    for row in body_rows {
        bodies.insert((
            row.required("ROUTINE_SCHEMA")?,
            row.required("ROUTINE_NAME")?,
        ));
    }
    let values = rows(connection, source, "packages", &format!("SELECT ROUTINE_SCHEMA, ROUTINE_NAME, DEFINER, SECURITY_TYPE, ROUTINE_COMMENT FROM information_schema.ROUTINES WHERE ROUTINE_TYPE = 'PACKAGE' AND {} ORDER BY ROUTINE_SCHEMA, ROUTINE_NAME", schema_filter(source, "ROUTINE_SCHEMA")))?;
    values
        .into_iter()
        .map(|row| {
            let schema: String = row.required("ROUTINE_SCHEMA")?;
            let name: String = row.required("ROUTINE_NAME")?;
            let body = bodies
                .contains(&(schema.clone(), name.clone()))
                .then(|| show_package_definition(connection, source, &schema, &name, true))
                .transpose()?;
            Ok(Package {
                specification: show_package_definition(connection, source, &schema, &name, false)?,
                schema,
                name,
                definer: row.required("DEFINER")?,
                security: row.semantic("SECURITY_TYPE", sql_security)?,
                comment: nonempty(row.optional("ROUTINE_COMMENT")?),
                body,
            })
        })
        .collect()
}

fn show_package_definition(
    connection: &mut Conn,
    source: &MariaDbSource,
    schema: &str,
    name: &str,
    body: bool,
) -> Result<StoredProgramDefinition, IntrospectionError> {
    let kind = if body { "PACKAGE BODY" } else { "PACKAGE" };
    let sql = format!(
        "SHOW CREATE {kind} `{}`.`{}`",
        escape_identifier(schema),
        escape_identifier(name)
    );
    let row = rows(connection, source, "package definitions", &sql)?
        .into_iter()
        .next()
        .ok_or_else(|| IntrospectionError::MissingDefinition {
            source_id: source.id.clone(),
            object: format!("{schema}.{name} {kind}"),
        })?;
    Ok(StoredProgramDefinition {
        sql_mode: row.required_at(1, "sql mode")?,
        definition: row.required_at(2, "create statement")?,
        character_set_client: row.required_at(3, "character set client")?,
        collation_connection: row.required_at(4, "collation connection")?,
        database_collation: row.required_at(5, "database collation")?,
    })
}

fn attach_parameters(
    connection: &mut Conn,
    source: &MariaDbSource,
    routines: &mut [Routine],
) -> Result<(), IntrospectionError> {
    let values = rows(connection, source, "routine parameters", &format!("SELECT SPECIFIC_SCHEMA, SPECIFIC_NAME, ORDINAL_POSITION, PARAMETER_MODE, PARAMETER_NAME, DATA_TYPE, DTD_IDENTIFIER, PARAMETER_DEFAULT FROM information_schema.PARAMETERS WHERE {} ORDER BY SPECIFIC_SCHEMA, SPECIFIC_NAME, ORDINAL_POSITION", schema_filter(source, "SPECIFIC_SCHEMA")))?;
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
                mode: row.optional_semantic("PARAMETER_MODE", parameter_mode)?,
                name: row.optional("PARAMETER_NAME")?,
                data_type: row.required("DATA_TYPE")?,
                dtd_identifier: row.required("DTD_IDENTIFIER")?,
                default: row.optional("PARAMETER_DEFAULT")?,
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
    source: &MariaDbSource,
) -> Result<Vec<Trigger>, IntrospectionError> {
    let values = rows(connection, source, "triggers", &format!("SELECT TRIGGER_SCHEMA, TRIGGER_NAME, EVENT_OBJECT_TABLE, EVENT_MANIPULATION, ACTION_TIMING, ACTION_ORIENTATION, ACTION_STATEMENT, ACTION_ORDER, SQL_MODE, DEFINER, CHARACTER_SET_CLIENT, COLLATION_CONNECTION, DATABASE_COLLATION FROM information_schema.TRIGGERS WHERE {} ORDER BY TRIGGER_SCHEMA, TRIGGER_NAME", schema_filter(source, "TRIGGER_SCHEMA")))?;
    values
        .into_iter()
        .map(|row| {
            let schema: String = row.required("TRIGGER_SCHEMA")?;
            let name: String = row.required("TRIGGER_NAME")?;
            let sql = format!(
                "SHOW CREATE TRIGGER `{}`.`{}`",
                escape_identifier(&schema),
                escape_identifier(&name)
            );
            let create_statement: String = rows(connection, source, "trigger definitions", &sql)?
                .into_iter()
                .next()
                .map(|value| value.required_at(2, "create statement"))
                .transpose()?
                .ok_or_else(|| IntrospectionError::MissingDefinition {
                    source_id: source.id.clone(),
                    object: format!("{schema}.{name}"),
                })?;
            Ok(Trigger {
                schema,
                name,
                table: row.required("EVENT_OBJECT_TABLE")?,
                events: row.semantic("EVENT_MANIPULATION", trigger_events)?,
                update_columns: Vec::new(),
                timing: row.semantic("ACTION_TIMING", trigger_timing)?,
                orientation: row.semantic("ACTION_ORIENTATION", trigger_orientation)?,
                statement: row.required("ACTION_STATEMENT")?,
                action_order: row.required("ACTION_ORDER")?,
                sql_mode: row.required("SQL_MODE")?,
                definer: row.required("DEFINER")?,
                character_set_client: row.required("CHARACTER_SET_CLIENT")?,
                collation_connection: row.required("COLLATION_CONNECTION")?,
                database_collation: row.required("DATABASE_COLLATION")?,
                create_statement: stable_create_statement(&create_statement),
            })
        })
        .collect()
}

fn attach_trigger_update_columns(
    connection: &mut Conn,
    source: &MariaDbSource,
    triggers: &mut [Trigger],
) -> Result<(), IntrospectionError> {
    let values = rows(
        connection,
        source,
        "trigger update columns",
        &format!(
            "SELECT TRIGGER_SCHEMA, TRIGGER_NAME, EVENT_OBJECT_COLUMN FROM information_schema.TRIGGERED_UPDATE_COLUMNS WHERE {} ORDER BY TRIGGER_SCHEMA, TRIGGER_NAME, EVENT_OBJECT_COLUMN",
            schema_filter(source, "TRIGGER_SCHEMA")
        ),
    )?;
    let mut grouped = BTreeMap::<(String, String), Vec<String>>::new();
    for row in values {
        grouped
            .entry((
                row.required("TRIGGER_SCHEMA")?,
                row.required("TRIGGER_NAME")?,
            ))
            .or_default()
            .push(row.required("EVENT_OBJECT_COLUMN")?);
    }
    for trigger in triggers {
        trigger.update_columns = grouped
            .remove(&(trigger.schema.clone(), trigger.name.clone()))
            .unwrap_or_default();
    }
    Ok(())
}

fn load_events(
    connection: &mut Conn,
    source: &MariaDbSource,
) -> Result<Vec<Event>, IntrospectionError> {
    let values = rows(connection, source, "events", &format!("SELECT EVENT_SCHEMA, EVENT_NAME, DEFINER, TIME_ZONE, EVENT_TYPE, CAST(EXECUTE_AT AS CHAR) AS EXECUTE_AT, INTERVAL_VALUE, INTERVAL_FIELD, CAST(STARTS AS CHAR) AS STARTS, CAST(ENDS AS CHAR) AS ENDS, STATUS, ON_COMPLETION, EVENT_COMMENT, EVENT_DEFINITION, SQL_MODE, ORIGINATOR, CHARACTER_SET_CLIENT, COLLATION_CONNECTION, DATABASE_COLLATION FROM information_schema.EVENTS WHERE {} ORDER BY EVENT_SCHEMA, EVENT_NAME", schema_filter(source, "EVENT_SCHEMA")))?;
    values
        .into_iter()
        .map(|row| {
            let schema: String = row.required("EVENT_SCHEMA")?;
            let name: String = row.required("EVENT_NAME")?;
            let sql = format!(
                "SHOW CREATE EVENT `{}`.`{}`",
                escape_identifier(&schema),
                escape_identifier(&name)
            );
            let create_statement: String = rows(connection, source, "event definitions", &sql)?
                .into_iter()
                .next()
                .map(|value| value.required_at(3, "create statement"))
                .transpose()?
                .ok_or_else(|| IntrospectionError::MissingDefinition {
                    source_id: source.id.clone(),
                    object: format!("{schema}.{name}"),
                })?;
            Ok(Event {
                schema,
                name,
                definer: row.required("DEFINER")?,
                time_zone: row.required("TIME_ZONE")?,
                kind: row.semantic("EVENT_TYPE", scheduled_event_kind)?,
                execute_at: row.optional("EXECUTE_AT")?,
                interval_value: row.optional("INTERVAL_VALUE")?,
                interval_unit: row.optional_semantic("INTERVAL_FIELD", scheduled_interval_unit)?,
                starts: row.optional("STARTS")?,
                ends: row.optional("ENDS")?,
                status: row.semantic("STATUS", scheduled_event_status)?,
                completion: row.semantic("ON_COMPLETION", scheduled_event_completion)?,
                comment: nonempty(row.optional("EVENT_COMMENT")?),
                definition: row.required("EVENT_DEFINITION")?,
                sql_mode: row.required("SQL_MODE")?,
                originator: row.required("ORIGINATOR")?,
                character_set_client: row.required("CHARACTER_SET_CLIENT")?,
                collation_connection: row.required("COLLATION_CONNECTION")?,
                database_collation: row.required("DATABASE_COLLATION")?,
                create_statement: stable_create_statement(&create_statement),
            })
        })
        .collect()
}

fn load_servers(
    connection: &mut Conn,
    source: &MariaDbSource,
) -> Result<Vec<ServerDefinition>, IntrospectionError> {
    let values = rows(
        connection,
        source,
        "server definitions",
        "SELECT Server_name, Host, Db, Username, Port, Socket, Wrapper, Owner FROM mysql.servers ORDER BY LOWER(Server_name), Server_name",
    )?;
    values
        .into_iter()
        .map(|row| {
            let port = row.required::<u16>("Port")?;
            Ok(ServerDefinition {
                name: row.required("Server_name")?,
                wrapper: row.required("Wrapper")?,
                host: nonempty(row.optional("Host")?),
                database: nonempty(row.optional("Db")?),
                username: nonempty(row.optional("Username")?),
                port: (port != 0).then_some(port),
                socket: nonempty(row.optional("Socket")?),
                owner: nonempty(row.optional("Owner")?),
                options: Vec::new(),
            })
        })
        .collect()
}

fn attach_server_options(
    connection: &mut Conn,
    source: &MariaDbSource,
    servers: &mut [ServerDefinition],
) -> Result<(), IntrospectionError> {
    let values = rows(
        connection,
        source,
        "server definition options",
        "SELECT s.Server_name, jt.option_name AS OPTION_NAME, LOWER(jt.option_name) REGEXP '(password|passwd|secret|token|credential|private|key)' AS IS_SENSITIVE, CASE WHEN LOWER(jt.option_name) REGEXP '(password|passwd|secret|token|credential|private|key)' THEN NULL ELSE JSON_UNQUOTE(JSON_EXTRACT(s.Options, CONCAT('$.', jt.option_name))) END AS OPTION_VALUE FROM mysql.servers s JOIN JSON_TABLE(JSON_KEYS(s.Options), '$[*]' COLUMNS(option_name VARCHAR(512) PATH '$')) AS jt ORDER BY LOWER(s.Server_name), s.Server_name, LOWER(jt.option_name), jt.option_name",
    )?;
    let mut grouped = BTreeMap::<String, Vec<ServerOption>>::new();
    for row in values {
        grouped
            .entry(row.required("Server_name")?)
            .or_default()
            .push(ServerOption {
                name: row.required("OPTION_NAME")?,
                value: row.optional("OPTION_VALUE")?,
                sensitive: row.required::<u64>("IS_SENSITIVE")? != 0,
            });
    }
    for server in servers {
        server.options = grouped.remove(&server.name).unwrap_or_default();
    }
    Ok(())
}

fn load_loadable_functions(
    connection: &mut Conn,
    source: &MariaDbSource,
) -> Result<Vec<LoadableFunction>, IntrospectionError> {
    rows(
        connection,
        source,
        "loadable functions",
        "SELECT name, ret, dl, type FROM mysql.func ORDER BY LOWER(name), name",
    )?
    .into_iter()
    .map(|row| {
        let return_type_code = row.required::<u8>("ret")?;
        Ok(LoadableFunction {
            name: row.required("name")?,
            return_type: loadable_function_return_type(return_type_code)
                .ok_or_else(|| row.unknown_value("ret", &return_type_code.to_string()))?,
            library: row.required("dl")?,
            kind: row.semantic("type", loadable_function_kind)?,
        })
    })
    .collect()
}

fn load_plugins(
    connection: &mut Conn,
    source: &MariaDbSource,
) -> Result<Vec<Plugin>, IntrospectionError> {
    rows(
        connection,
        source,
        "plugins",
        "SELECT PLUGIN_NAME, PLUGIN_VERSION, PLUGIN_STATUS, PLUGIN_TYPE, PLUGIN_TYPE_VERSION, PLUGIN_LIBRARY, PLUGIN_LIBRARY_VERSION, PLUGIN_AUTHOR, PLUGIN_DESCRIPTION, PLUGIN_LICENSE, LOAD_OPTION, PLUGIN_MATURITY, PLUGIN_AUTH_VERSION FROM information_schema.PLUGINS ORDER BY LOWER(PLUGIN_NAME), PLUGIN_NAME",
    )?
    .into_iter()
    .map(|row| {
        Ok(Plugin {
            name: row.required("PLUGIN_NAME")?,
            version: row.required("PLUGIN_VERSION")?,
            status: row.semantic("PLUGIN_STATUS", plugin_status)?,
            kind: row.semantic("PLUGIN_TYPE", plugin_kind)?,
            type_version: row.required("PLUGIN_TYPE_VERSION")?,
            library: nonempty(row.optional("PLUGIN_LIBRARY")?),
            library_version: nonempty(row.optional("PLUGIN_LIBRARY_VERSION")?),
            author: trimmed_nonempty(row.optional("PLUGIN_AUTHOR")?),
            description: trimmed_nonempty(row.optional("PLUGIN_DESCRIPTION")?),
            license: row.semantic("PLUGIN_LICENSE", plugin_license)?,
            load_option: row.semantic("LOAD_OPTION", plugin_load_option)?,
            maturity: row.semantic("PLUGIN_MATURITY", plugin_maturity)?,
            authentication_version: nonempty(row.optional("PLUGIN_AUTH_VERSION")?),
        })
    })
    .collect()
}

fn load_accounts(
    connection: &mut Conn,
    source: &MariaDbSource,
) -> Result<Vec<Account>, IntrospectionError> {
    let values = rows(
        connection,
        source,
        "accounts",
        "SELECT u.Host, u.User, u.plugin, u.password_expired, u.is_role, u.default_role, u.ssl_type, u.ssl_cipher, u.x509_issuer, u.x509_subject, u.max_questions, u.max_updates, u.max_connections, u.max_user_connections, CASE WHEN u.max_statement_time = 0 THEN NULL ELSE CAST(u.max_statement_time AS CHAR) END AS MAX_STATEMENT_TIME, CAST(JSON_VALUE(g.Priv, '$.password_lifetime') AS UNSIGNED) AS PASSWORD_LIFETIME, COALESCE(JSON_VALUE(g.Priv, '$.account_locked'), 0) <> 0 AS ACCOUNT_LOCKED FROM mysql.user u JOIN mysql.global_priv g ON g.Host=u.Host AND g.User=u.User ORDER BY LOWER(u.User), u.User, LOWER(u.Host), u.Host",
    )?;
    values
        .into_iter()
        .map(|row| {
            let plugin = nonempty(row.optional("plugin")?);
            Ok(Account {
                name: row.required("User")?,
                host: row.required("Host")?,
                kind: if row.semantic("is_role", y_n)? {
                    AccountKind::Role
                } else {
                    AccountKind::User
                },
                authentication_plugins: plugin.into_iter().collect(),
                password_expired: row.semantic("password_expired", y_n)?,
                password_lifetime_days: row.optional("PASSWORD_LIFETIME")?,
                account_locked: row.semantic_u64("ACCOUNT_LOCKED", binary_flag)?,
                default_role: nonempty(row.optional("default_role")?),
                tls_requirement: row.semantic("ssl_type", tls_requirement)?,
                tls_cipher: nonempty(row.optional("ssl_cipher")?),
                x509_issuer: nonempty(row.optional("x509_issuer")?),
                x509_subject: nonempty(row.optional("x509_subject")?),
                max_queries_per_hour: nonzero(row.required("max_questions")?),
                max_updates_per_hour: nonzero(row.required("max_updates")?),
                max_connections_per_hour: nonzero(row.required("max_connections")?),
                max_user_connections: nonzero(row.required("max_user_connections")?),
                max_statement_time: row.optional("MAX_STATEMENT_TIME")?,
            })
        })
        .collect()
}

fn attach_authentication_plugins(
    connection: &mut Conn,
    source: &MariaDbSource,
    accounts: &mut [Account],
) -> Result<(), IntrospectionError> {
    let values = rows(
        connection,
        source,
        "account authentication plugins",
        "SELECT g.Host, g.User, jt.plugin AS PLUGIN FROM mysql.global_priv g JOIN JSON_TABLE(COALESCE(JSON_QUERY(g.Priv, '$.auth_or'), '[]'), '$[*]' COLUMNS(plugin VARCHAR(128) PATH '$.plugin')) AS jt WHERE jt.plugin IS NOT NULL AND jt.plugin <> '' ORDER BY LOWER(g.User), g.User, LOWER(g.Host), g.Host, LOWER(jt.plugin), jt.plugin",
    )?;
    let mut grouped = BTreeMap::<(String, String), Vec<String>>::new();
    for row in values {
        grouped
            .entry((row.required("User")?, row.required("Host")?))
            .or_default()
            .push(row.required("PLUGIN")?);
    }
    for account in accounts {
        account.authentication_plugins.extend(
            grouped
                .remove(&(account.name.clone(), account.host.clone()))
                .unwrap_or_default(),
        );
        account
            .authentication_plugins
            .sort_by_key(|value| value.to_ascii_lowercase());
        account.authentication_plugins.dedup();
    }
    Ok(())
}

fn load_role_memberships(
    connection: &mut Conn,
    source: &MariaDbSource,
) -> Result<Vec<RoleMembership>, IntrospectionError> {
    rows(
        connection,
        source,
        "role memberships",
        "SELECT User, Host, Role, Admin_option FROM mysql.roles_mapping ORDER BY LOWER(User), User, LOWER(Host), Host, LOWER(Role), Role",
    )?
    .into_iter()
    .map(|row| {
        Ok(RoleMembership {
            user: row.required("User")?,
            host: row.required("Host")?,
            role: row.required("Role")?,
            admin_option: row.semantic("Admin_option", y_n)?,
        })
    })
    .collect()
}

fn load_privileges(
    connection: &mut Conn,
    source: &MariaDbSource,
) -> Result<Vec<Privilege>, IntrospectionError> {
    let schema_privileges = schema_filter(source, "TABLE_SCHEMA");
    let routine_privileges = schema_filter(source, "Db");
    let sql = format!(
        "SELECT GRANTEE, SCOPE_KIND, OBJECT_SCHEMA, OBJECT_NAME, COLUMN_NAME, PRIVILEGE_TYPE, IS_GRANTABLE FROM (\
         SELECT GRANTEE, 'global' AS SCOPE_KIND, CAST(NULL AS CHAR) AS OBJECT_SCHEMA, CAST(NULL AS CHAR) AS OBJECT_NAME, CAST(NULL AS CHAR) AS COLUMN_NAME, PRIVILEGE_TYPE, IS_GRANTABLE FROM information_schema.USER_PRIVILEGES \
         UNION ALL SELECT GRANTEE, 'schema', TABLE_SCHEMA, CAST(NULL AS CHAR), CAST(NULL AS CHAR), PRIVILEGE_TYPE, IS_GRANTABLE FROM information_schema.SCHEMA_PRIVILEGES WHERE {schema_privileges} \
         UNION ALL SELECT GRANTEE, 'table', TABLE_SCHEMA, TABLE_NAME, CAST(NULL AS CHAR), PRIVILEGE_TYPE, IS_GRANTABLE FROM information_schema.TABLE_PRIVILEGES WHERE {schema_privileges} \
         UNION ALL SELECT GRANTEE, 'column', TABLE_SCHEMA, TABLE_NAME, COLUMN_NAME, PRIVILEGE_TYPE, IS_GRANTABLE FROM information_schema.COLUMN_PRIVILEGES WHERE {schema_privileges} \
         UNION ALL SELECT CONCAT(QUOTE(User), '@', QUOTE(Host)), 'schema', Db, CAST(NULL AS CHAR), CAST(NULL AS CHAR), 'SHOW CREATE ROUTINE', IF(Grant_priv = 'Y', 'YES', 'NO') FROM mysql.db WHERE Show_create_routine_priv = 'Y' AND {routine_privileges} \
         UNION ALL SELECT CONCAT(QUOTE(User), '@', QUOTE(Host)), LOWER(REPLACE(Routine_type, ' ', '_')), Db, Routine_name, CAST(NULL AS CHAR), 'EXECUTE', CASE WHEN FIND_IN_SET('Grant', Proc_priv) > 0 THEN 'YES' ELSE 'NO' END FROM mysql.procs_priv WHERE FIND_IN_SET('Execute', Proc_priv) > 0 AND {routine_privileges} \
         UNION ALL SELECT CONCAT(QUOTE(User), '@', QUOTE(Host)), LOWER(REPLACE(Routine_type, ' ', '_')), Db, Routine_name, CAST(NULL AS CHAR), 'ALTER ROUTINE', CASE WHEN FIND_IN_SET('Grant', Proc_priv) > 0 THEN 'YES' ELSE 'NO' END FROM mysql.procs_priv WHERE FIND_IN_SET('Alter Routine', Proc_priv) > 0 AND {routine_privileges} \
         UNION ALL SELECT CONCAT(QUOTE(User), '@', QUOTE(Host)), 'proxy', CAST(NULL AS CHAR), CONCAT(QUOTE(Proxied_user), '@', QUOTE(Proxied_host)), CAST(NULL AS CHAR), 'PROXY', IF(With_grant <> 0, 'YES', 'NO') FROM mysql.proxies_priv\
         ) privileges ORDER BY LOWER(GRANTEE), GRANTEE, SCOPE_KIND, OBJECT_SCHEMA, OBJECT_NAME, COLUMN_NAME, PRIVILEGE_TYPE"
    );
    rows(connection, source, "privileges", &sql)?
        .into_iter()
        .map(|row| {
            let object_kind = row.semantic("SCOPE_KIND", privilege_object_kind)?;
            Ok(Privilege {
                grantee: row.required("GRANTEE")?,
                object_kind,
                schema: row.optional("OBJECT_SCHEMA")?,
                object: row.optional("OBJECT_NAME")?,
                column: row.optional("COLUMN_NAME")?,
                privilege: row.required("PRIVILEGE_TYPE")?,
                grantable: row.semantic("IS_GRANTABLE", yes_no)?,
            })
        })
        .collect()
}

fn nonzero(value: u64) -> Option<u64> {
    (value != 0).then_some(value)
}

fn system_time_period(definition: &str) -> Option<SystemTimePeriod> {
    let uppercase = definition.to_ascii_uppercase();
    let start = uppercase.find("PERIOD FOR SYSTEM_TIME")?;
    let remainder = &definition[start..];
    let open = remainder.find('(')?;
    let close = remainder[open + 1..].find(')')? + open + 1;
    let mut columns = remainder[open + 1..close]
        .split(',')
        .map(|value| value.trim().trim_matches('`').to_string());
    Some(SystemTimePeriod {
        start_column: columns.next()?,
        end_column: columns.next()?,
    })
}

fn vector_index_options(definition: &str, index_name: &str) -> Option<VectorIndexOptions> {
    let quoted_name = format!("`{}`", index_name.replace('`', "``"));
    let line = definition.lines().find(|line| {
        line.to_ascii_uppercase().contains("VECTOR KEY") && line.contains(&quoted_name)
    })?;
    Some(VectorIndexOptions {
        m: ddl_option(line, "`M`=").and_then(|value| value.parse().ok()),
        distance: ddl_option(line, "`DISTANCE`="),
    })
}

fn ddl_option(line: &str, option: &str) -> Option<String> {
    let uppercase = line.to_ascii_uppercase();
    let start = uppercase.find(option)? + option.len();
    let value = line[start..]
        .trim_start()
        .split(|character: char| character == ',' || character.is_ascii_whitespace())
        .next()?
        .trim_matches('`')
        .to_string();
    (!value.is_empty()).then_some(value)
}
fn rows(
    connection: &mut Conn,
    source: &MariaDbSource,
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
fn schema_filter(source: &MariaDbSource, column: &str) -> String {
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
fn constraint_kind(value: &str) -> Option<ConstraintKind> {
    match value {
        "PRIMARY KEY" => Some(ConstraintKind::PrimaryKey),
        "UNIQUE" => Some(ConstraintKind::Unique),
        "FOREIGN KEY" => Some(ConstraintKind::ForeignKey),
        "CHECK" => Some(ConstraintKind::Check),
        _ => None,
    }
}

fn check_constraint_level(value: &str) -> Option<CheckConstraintLevel> {
    match value.to_ascii_uppercase().as_str() {
        "COLUMN" => Some(CheckConstraintLevel::Column),
        "TABLE" => Some(CheckConstraintLevel::Table),
        _ => None,
    }
}

fn optional_foreign_key_match(
    row: &CatalogRow,
) -> Result<Option<ForeignKeyMatch>, IntrospectionError> {
    let Some(value) = row.optional::<String>("MATCH_OPTION")? else {
        return Ok(None);
    };
    match value.as_str() {
        "NONE" => Ok(None),
        "SIMPLE" => Ok(Some(ForeignKeyMatch::Simple)),
        "PARTIAL" => Ok(Some(ForeignKeyMatch::Partial)),
        "FULL" => Ok(Some(ForeignKeyMatch::Full)),
        _ => Err(row.unknown_value("MATCH_OPTION", &value)),
    }
}

fn foreign_key_action(value: &str) -> Option<ForeignKeyAction> {
    match value {
        "NO ACTION" => Some(ForeignKeyAction::NoAction),
        "RESTRICT" => Some(ForeignKeyAction::Restrict),
        "SET NULL" => Some(ForeignKeyAction::SetNull),
        "SET DEFAULT" => Some(ForeignKeyAction::SetDefault),
        "CASCADE" => Some(ForeignKeyAction::Cascade),
        _ => None,
    }
}

fn index_sort_order(value: &str) -> Option<IndexSortOrder> {
    match value {
        "A" => Some(IndexSortOrder::Ascending),
        "D" => Some(IndexSortOrder::Descending),
        _ => None,
    }
}

fn yes_no(value: &str) -> Option<bool> {
    match value {
        "YES" => Some(true),
        "NO" => Some(false),
        _ => None,
    }
}

fn y_n(value: &str) -> Option<bool> {
    match value {
        "Y" => Some(true),
        "N" => Some(false),
        _ => None,
    }
}

fn binary_flag(value: u64) -> Option<bool> {
    match value {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

fn generated_column_storage(extra: &str) -> Option<GeneratedColumnStorage> {
    let extra = extra.to_ascii_uppercase();
    if extra.contains("VIRTUAL GENERATED") {
        Some(GeneratedColumnStorage::Virtual)
    } else if extra.contains("STORED GENERATED") || extra.contains("PERSISTENT GENERATED") {
        Some(GeneratedColumnStorage::Stored)
    } else {
        None
    }
}

fn partition_method(value: &str) -> Option<PartitionMethod> {
    match value {
        "RANGE" => Some(PartitionMethod::Range),
        "RANGE COLUMNS" => Some(PartitionMethod::RangeColumns),
        "LIST" => Some(PartitionMethod::List),
        "LIST COLUMNS" => Some(PartitionMethod::ListColumns),
        "HASH" => Some(PartitionMethod::Hash),
        "LINEAR HASH" => Some(PartitionMethod::LinearHash),
        "KEY" => Some(PartitionMethod::Key),
        "LINEAR KEY" => Some(PartitionMethod::LinearKey),
        "SYSTEM_TIME" => Some(PartitionMethod::SystemTime),
        _ => None,
    }
}

fn view_check_option(value: &str) -> Option<ViewCheckOption> {
    match value {
        "NONE" => Some(ViewCheckOption::None),
        "CASCADED" => Some(ViewCheckOption::Cascaded),
        "LOCAL" => Some(ViewCheckOption::Local),
        _ => None,
    }
}

fn view_algorithm(value: &str) -> Option<ViewAlgorithm> {
    match value {
        "UNDEFINED" => Some(ViewAlgorithm::Undefined),
        "MERGE" => Some(ViewAlgorithm::Merge),
        "TEMPTABLE" => Some(ViewAlgorithm::TemporaryTable),
        _ => None,
    }
}

fn sql_security(value: &str) -> Option<SqlSecurity> {
    match value {
        "DEFINER" => Some(SqlSecurity::Definer),
        "INVOKER" => Some(SqlSecurity::Invoker),
        _ => None,
    }
}

fn routine_kind(value: &str) -> Option<RoutineKind> {
    match value {
        "FUNCTION" => Some(RoutineKind::Function),
        "PROCEDURE" => Some(RoutineKind::Procedure),
        _ => None,
    }
}

fn routine_data_access(value: &str) -> Option<RoutineDataAccess> {
    match value {
        "CONTAINS SQL" => Some(RoutineDataAccess::ContainsSql),
        "NO SQL" => Some(RoutineDataAccess::NoSql),
        "READS SQL DATA" => Some(RoutineDataAccess::ReadsSqlData),
        "MODIFIES SQL DATA" => Some(RoutineDataAccess::ModifiesSqlData),
        _ => None,
    }
}

fn parameter_mode(value: &str) -> Option<ParameterMode> {
    match value {
        "IN" => Some(ParameterMode::In),
        "OUT" => Some(ParameterMode::Out),
        "INOUT" => Some(ParameterMode::InOut),
        _ => None,
    }
}

fn trigger_events(value: &str) -> Option<Vec<TriggerEvent>> {
    let events = value
        .split(',')
        .map(|event| match event.trim() {
            "INSERT" => Some(TriggerEvent::Insert),
            "UPDATE" => Some(TriggerEvent::Update),
            "DELETE" => Some(TriggerEvent::Delete),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    (!events.is_empty()).then_some(events)
}

fn trigger_timing(value: &str) -> Option<TriggerTiming> {
    match value {
        "BEFORE" => Some(TriggerTiming::Before),
        "AFTER" => Some(TriggerTiming::After),
        _ => None,
    }
}

fn trigger_orientation(value: &str) -> Option<TriggerOrientation> {
    match value {
        "ROW" => Some(TriggerOrientation::Row),
        _ => None,
    }
}

fn scheduled_event_kind(value: &str) -> Option<ScheduledEventKind> {
    match value {
        "ONE TIME" => Some(ScheduledEventKind::OneTime),
        "RECURRING" => Some(ScheduledEventKind::Recurring),
        _ => None,
    }
}

fn scheduled_event_status(value: &str) -> Option<ScheduledEventStatus> {
    match value {
        "ENABLED" => Some(ScheduledEventStatus::Enabled),
        "DISABLED" => Some(ScheduledEventStatus::Disabled),
        "SLAVESIDE_DISABLED" | "REPLICA_SIDE_DISABLED" => {
            Some(ScheduledEventStatus::ReplicaSideDisabled)
        }
        _ => None,
    }
}

fn scheduled_event_completion(value: &str) -> Option<ScheduledEventCompletion> {
    match value {
        "PRESERVE" => Some(ScheduledEventCompletion::Preserve),
        "NOT PRESERVE" => Some(ScheduledEventCompletion::Drop),
        _ => None,
    }
}

fn scheduled_interval_unit(value: &str) -> Option<ScheduledIntervalUnit> {
    match value {
        "YEAR" => Some(ScheduledIntervalUnit::Year),
        "QUARTER" => Some(ScheduledIntervalUnit::Quarter),
        "MONTH" => Some(ScheduledIntervalUnit::Month),
        "WEEK" => Some(ScheduledIntervalUnit::Week),
        "DAY" => Some(ScheduledIntervalUnit::Day),
        "HOUR" => Some(ScheduledIntervalUnit::Hour),
        "MINUTE" => Some(ScheduledIntervalUnit::Minute),
        "SECOND" => Some(ScheduledIntervalUnit::Second),
        "MICROSECOND" => Some(ScheduledIntervalUnit::Microsecond),
        "YEAR_MONTH" => Some(ScheduledIntervalUnit::YearMonth),
        "DAY_HOUR" => Some(ScheduledIntervalUnit::DayHour),
        "DAY_MINUTE" => Some(ScheduledIntervalUnit::DayMinute),
        "DAY_SECOND" => Some(ScheduledIntervalUnit::DaySecond),
        "HOUR_MINUTE" => Some(ScheduledIntervalUnit::HourMinute),
        "HOUR_SECOND" => Some(ScheduledIntervalUnit::HourSecond),
        "MINUTE_SECOND" => Some(ScheduledIntervalUnit::MinuteSecond),
        "DAY_MICROSECOND" => Some(ScheduledIntervalUnit::DayMicrosecond),
        "HOUR_MICROSECOND" => Some(ScheduledIntervalUnit::HourMicrosecond),
        "MINUTE_MICROSECOND" => Some(ScheduledIntervalUnit::MinuteMicrosecond),
        "SECOND_MICROSECOND" => Some(ScheduledIntervalUnit::SecondMicrosecond),
        _ => None,
    }
}

fn loadable_function_return_type(value: u8) -> Option<LoadableFunctionReturnType> {
    match value {
        0 => Some(LoadableFunctionReturnType::String),
        1 => Some(LoadableFunctionReturnType::Real),
        2 => Some(LoadableFunctionReturnType::Integer),
        3 => Some(LoadableFunctionReturnType::Row),
        4 => Some(LoadableFunctionReturnType::Decimal),
        5 => Some(LoadableFunctionReturnType::Temporal),
        _ => None,
    }
}

fn loadable_function_kind(value: &str) -> Option<LoadableFunctionKind> {
    match value.to_ascii_lowercase().as_str() {
        "function" => Some(LoadableFunctionKind::Scalar),
        "aggregate" => Some(LoadableFunctionKind::Aggregate),
        _ => None,
    }
}

fn plugin_status(value: &str) -> Option<PluginStatus> {
    match value {
        "ACTIVE" => Some(PluginStatus::Active),
        "INACTIVE" => Some(PluginStatus::Inactive),
        "DISABLED" => Some(PluginStatus::Disabled),
        "DELETED" => Some(PluginStatus::Deleted),
        _ => None,
    }
}

fn plugin_kind(value: &str) -> Option<PluginKind> {
    match value {
        "UDF" => Some(PluginKind::Udf),
        "STORAGE ENGINE" => Some(PluginKind::StorageEngine),
        "FTPARSER" => Some(PluginKind::FullTextParser),
        "DAEMON" => Some(PluginKind::Daemon),
        "INFORMATION SCHEMA" => Some(PluginKind::InformationSchema),
        "AUDIT" => Some(PluginKind::Audit),
        "REPLICATION" => Some(PluginKind::Replication),
        "AUTHENTICATION" => Some(PluginKind::Authentication),
        "PASSWORD VALIDATION" => Some(PluginKind::PasswordValidation),
        "ENCRYPTION" => Some(PluginKind::Encryption),
        "DATA TYPE" => Some(PluginKind::DataType),
        "FUNCTION" => Some(PluginKind::Function),
        _ => None,
    }
}

fn plugin_license(value: &str) -> Option<PluginLicense> {
    match value {
        "PROPRIETARY" => Some(PluginLicense::Proprietary),
        "GPL" => Some(PluginLicense::Gpl),
        "BSD" => Some(PluginLicense::Bsd),
        _ => None,
    }
}

fn plugin_load_option(value: &str) -> Option<PluginLoadOption> {
    match value {
        "OFF" => Some(PluginLoadOption::Off),
        "ON" => Some(PluginLoadOption::On),
        "FORCE" => Some(PluginLoadOption::Force),
        "FORCE_PLUS_PERMANENT" => Some(PluginLoadOption::ForcePlusPermanent),
        _ => None,
    }
}

fn plugin_maturity(value: &str) -> Option<PluginMaturity> {
    match value.to_ascii_lowercase().as_str() {
        "unknown" => Some(PluginMaturity::Unknown),
        "experimental" => Some(PluginMaturity::Experimental),
        "alpha" => Some(PluginMaturity::Alpha),
        "beta" => Some(PluginMaturity::Beta),
        "gamma" => Some(PluginMaturity::Gamma),
        "stable" => Some(PluginMaturity::Stable),
        _ => None,
    }
}

fn tls_requirement(value: &str) -> Option<TlsRequirement> {
    match value {
        "" => Some(TlsRequirement::None),
        "ANY" => Some(TlsRequirement::Any),
        "X509" => Some(TlsRequirement::X509),
        "SPECIFIED" => Some(TlsRequirement::Specified),
        _ => None,
    }
}

fn privilege_object_kind(value: &str) -> Option<PrivilegeObjectKind> {
    match value {
        "global" => Some(PrivilegeObjectKind::Global),
        "schema" => Some(PrivilegeObjectKind::Schema),
        "table" => Some(PrivilegeObjectKind::Table),
        "column" => Some(PrivilegeObjectKind::Column),
        "function" => Some(PrivilegeObjectKind::Function),
        "procedure" => Some(PrivilegeObjectKind::Procedure),
        "package" => Some(PrivilegeObjectKind::Package),
        "package_body" => Some(PrivilegeObjectKind::PackageBody),
        "proxy" => Some(PrivilegeObjectKind::Proxy),
        _ => None,
    }
}

fn semantic_value<T>(
    row: &CatalogRow,
    column: &str,
    value: &str,
    decode: impl FnOnce(&str) -> Option<T>,
) -> Result<T, IntrospectionError> {
    decode(value).ok_or_else(|| row.unknown_value(column, value))
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

    fn semantic<T>(
        &self,
        name: &str,
        decode: impl FnOnce(&str) -> Option<T>,
    ) -> Result<T, IntrospectionError> {
        let value = self.required::<String>(name)?;
        semantic_value(self, name, &value, decode)
    }

    fn semantic_u64<T>(
        &self,
        name: &str,
        decode: impl FnOnce(u64) -> Option<T>,
    ) -> Result<T, IntrospectionError> {
        let value = self.required::<u64>(name)?;
        decode(value).ok_or_else(|| self.unknown_value(name, &value.to_string()))
    }

    fn optional_semantic<T>(
        &self,
        name: &str,
        decode: impl FnOnce(&str) -> Option<T>,
    ) -> Result<Option<T>, IntrospectionError> {
        self.optional::<String>(name)?
            .map(|value| semantic_value(self, name, &value, decode))
            .transpose()
    }

    fn unknown_value(&self, column: &str, value: &str) -> IntrospectionError {
        IntrospectionError::Decode {
            source_id: self.source_id.clone(),
            operation: self.operation,
            column: column.to_string(),
            reason: format!("unknown native catalog value `{value}`"),
        }
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

fn trimmed_nonempty(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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
    use super::{
        generated_column_storage, loadable_function_return_type, partition_method, plugin_kind,
        plugin_license, plugin_status, scheduled_interval_unit, stable_create_statement,
        trigger_events,
    };
    use crate::{
        GeneratedColumnStorage, LoadableFunctionReturnType, PartitionMethod, PluginKind,
        PluginLicense, ScheduledIntervalUnit, TriggerEvent,
    };

    #[test]
    fn removes_only_the_volatile_auto_increment_counter() {
        assert_eq!(
            stable_create_statement(
                "CREATE TABLE `items` (`id` bigint NOT NULL AUTO_INCREMENT) ENGINE=InnoDB AUTO_INCREMENT=42 DEFAULT CHARSET=utf8mb4"
            ),
            "CREATE TABLE `items` (`id` bigint NOT NULL AUTO_INCREMENT) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"
        );
    }

    #[test]
    fn decodes_closed_native_catalog_values_semantically() {
        assert_eq!(
            trigger_events("INSERT,UPDATE,DELETE"),
            Some(vec![
                TriggerEvent::Insert,
                TriggerEvent::Update,
                TriggerEvent::Delete
            ])
        );
        assert_eq!(
            generated_column_storage("STORED GENERATED"),
            Some(GeneratedColumnStorage::Stored)
        );
        assert_eq!(
            partition_method("SYSTEM_TIME"),
            Some(PartitionMethod::SystemTime)
        );
        assert_eq!(
            scheduled_interval_unit("SECOND_MICROSECOND"),
            Some(ScheduledIntervalUnit::SecondMicrosecond)
        );
        assert_eq!(plugin_kind("FUNCTION"), Some(PluginKind::Function));
        assert_eq!(plugin_license("BSD"), Some(PluginLicense::Bsd));
        assert_eq!(
            (0..=5)
                .map(loadable_function_return_type)
                .collect::<Vec<_>>(),
            vec![
                Some(LoadableFunctionReturnType::String),
                Some(LoadableFunctionReturnType::Real),
                Some(LoadableFunctionReturnType::Integer),
                Some(LoadableFunctionReturnType::Row),
                Some(LoadableFunctionReturnType::Decimal),
                Some(LoadableFunctionReturnType::Temporal),
            ]
        );
    }

    #[test]
    fn rejects_unknown_closed_native_catalog_values() {
        assert_eq!(trigger_events("TRUNCATE"), None);
        assert_eq!(partition_method("MAGIC"), None);
        assert_eq!(scheduled_interval_unit("FORTNIGHT"), None);
        assert_eq!(plugin_kind("MYSTERY"), None);
        assert_eq!(plugin_license("UNKNOWN"), None);
        assert_eq!(plugin_status("STARTING"), None);
        assert_eq!(loadable_function_return_type(6), None);
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IntrospectionError {
    #[error("invalid MariaDB URL for source `{source_id}`")]
    Url {
        source_id: SourceId,
        #[source]
        source: mysql::UrlError,
    },
    #[error("could not connect to MariaDB source `{source_id}`")]
    Connect {
        source_id: SourceId,
        #[source]
        source: mysql::Error,
    },
    #[error("could not introspect {operation} for MariaDB source `{source_id}`")]
    Query {
        source_id: SourceId,
        operation: &'static str,
        #[source]
        source: mysql::Error,
    },
    #[error(
        "could not decode {operation} column `{column}` for MariaDB source `{source_id}`: {reason}"
    )]
    Decode {
        source_id: SourceId,
        operation: &'static str,
        column: String,
        reason: String,
    },
    #[error("MariaDB did not return a definition for `{object}` in source `{source_id}`")]
    MissingDefinition { source_id: SourceId, object: String },
}
