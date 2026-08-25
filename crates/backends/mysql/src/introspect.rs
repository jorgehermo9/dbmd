use std::{collections::BTreeMap, fmt};

use dbmd_core::{SourceId, SourceSnapshot};
use dbmd_relational::{ForeignKeyAction, ForeignKeyMatch, IndexSortOrder};
use mysql::{prelude::Queryable, Conn, Opts, Row};
use thiserror::Error;

use super::{
    Account, AuthenticationFactor, Catalog, Column, Component, Constraint, ConstraintKind,
    DefaultRole, Event, Index, IndexTerm, JsonDualityColumn, JsonDualityLink, JsonDualityTable,
    JsonDualityView, JsonDualityViewStatus, Library, LoadableFunction, LoadableFunctionKind,
    LoadableFunctionReturnType, Parameter, ParameterMode, Partition, Plugin, PluginKind,
    PluginLicense, PluginLoadOption, PluginStatus, Privilege, PrivilegeObjectKind, ResourceGroup,
    ResourceGroupKind, RoleGrant, Routine, RoutineDataAccess, RoutineKind, RoutineLibrary,
    ScheduledEventCompletion, ScheduledEventKind, ScheduledEventStatus, Schema, ServerDefinition,
    Snapshot, SpatialReferenceSystem, SqlSecurity, Table, Tablespace, TlsRequirement, Trigger,
    TriggerEvent, TriggerOrientation, TriggerTiming, View, ViewCheckOption, ViewKind,
};

#[derive(Clone, PartialEq, Eq)]
pub struct MysqlSource {
    id: SourceId,
    display_name: Option<String>,
    connection_url: String,
    schema: Option<String>,
    include_global_objects: bool,
}

impl MysqlSource {
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
    pub const fn with_global_objects(mut self, include: bool) -> Self {
        self.include_global_objects = include;
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
            .field("include_global_objects", &self.include_global_objects)
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
    attach_routine_libraries(&mut connection, source, &mut routines)?;
    let mut views = load_views(&mut connection, source)?;
    attach_json_duality_metadata(&mut connection, source, &mut views)?;
    let mut accounts = if source.include_global_objects {
        load_accounts(&mut connection, source)?
    } else {
        Vec::new()
    };
    if source.include_global_objects {
        attach_authentication_factors(&mut connection, source, &mut accounts)?;
    }
    let catalog = Catalog {
        schemas: load_schemas(&mut connection, source)?,
        tables,
        views,
        routines,
        triggers: load_triggers(&mut connection, source)?,
        events: load_events(&mut connection, source)?,
        libraries: load_libraries(&mut connection, source)?,
        servers: if source.include_global_objects {
            load_servers(&mut connection, source)?
        } else {
            Vec::new()
        },
        spatial_reference_systems: if source.include_global_objects {
            load_spatial_reference_systems(&mut connection, source)?
        } else {
            Vec::new()
        },
        tablespaces: if source.include_global_objects {
            load_tablespaces(&mut connection, source)?
        } else {
            Vec::new()
        },
        resource_groups: if source.include_global_objects {
            load_resource_groups(&mut connection, source)?
        } else {
            Vec::new()
        },
        loadable_functions: if source.include_global_objects {
            load_loadable_functions(&mut connection, source)?
        } else {
            Vec::new()
        },
        plugins: if source.include_global_objects {
            load_plugins(&mut connection, source)?
        } else {
            Vec::new()
        },
        components: if source.include_global_objects {
            load_components(&mut connection, source)?
        } else {
            Vec::new()
        },
        accounts,
        role_grants: if source.include_global_objects {
            load_role_grants(&mut connection, source)?
        } else {
            Vec::new()
        },
        default_roles: if source.include_global_objects {
            load_default_roles(&mut connection, source)?
        } else {
            Vec::new()
        },
        privileges: if source.include_global_objects {
            load_privileges(&mut connection, source)?
        } else {
            Vec::new()
        },
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
        "SELECT schema_.SCHEMA_NAME, schema_.DEFAULT_CHARACTER_SET_NAME, schema_.DEFAULT_COLLATION_NAME, schema_.DEFAULT_ENCRYPTION, extension.OPTIONS FROM information_schema.SCHEMATA schema_ LEFT JOIN information_schema.SCHEMATA_EXTENSIONS extension ON extension.SCHEMA_NAME = schema_.SCHEMA_NAME WHERE {} ORDER BY schema_.SCHEMA_NAME",
        schema_filter(source, "schema_.SCHEMA_NAME")
    ))?;
    values
        .into_iter()
        .map(|row| {
            Ok(Schema {
                name: row.required("SCHEMA_NAME")?,
                default_character_set: row.required("DEFAULT_CHARACTER_SET_NAME")?,
                default_collation: row.required("DEFAULT_COLLATION_NAME")?,
                default_encryption: row.semantic("DEFAULT_ENCRYPTION", yes_no)?,
                read_only: row
                    .optional::<String>("OPTIONS")?
                    .is_some_and(|options| options.contains("READ ONLY=1")),
            })
        })
        .collect()
}

fn load_tables(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Table>, IntrospectionError> {
    let values = rows(connection, source, "tables", &format!(
        "SELECT table_.TABLE_SCHEMA, table_.TABLE_NAME, table_.ENGINE, table_.ROW_FORMAT, table_.TABLE_COLLATION, table_.TABLE_COMMENT, table_.CREATE_OPTIONS, extension.ENGINE_ATTRIBUTE, extension.SECONDARY_ENGINE_ATTRIBUTE FROM information_schema.TABLES table_ LEFT JOIN information_schema.TABLES_EXTENSIONS extension ON extension.TABLE_SCHEMA = table_.TABLE_SCHEMA AND extension.TABLE_NAME = table_.TABLE_NAME WHERE table_.TABLE_TYPE = 'BASE TABLE' AND {} ORDER BY table_.TABLE_SCHEMA, table_.TABLE_NAME",
        schema_filter(source, "table_.TABLE_SCHEMA")
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
                engine_attribute: nonempty(row.optional("ENGINE_ATTRIBUTE")?),
                secondary_engine_attribute: nonempty(row.optional("SECONDARY_ENGINE_ATTRIBUTE")?),
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
        "SELECT column_.TABLE_SCHEMA, column_.TABLE_NAME, column_.COLUMN_NAME, column_.ORDINAL_POSITION, column_.DATA_TYPE, column_.COLUMN_TYPE, column_.IS_NULLABLE, column_.COLUMN_DEFAULT, column_.EXTRA, column_.GENERATION_EXPRESSION, column_.CHARACTER_SET_NAME, column_.COLLATION_NAME, column_.COLUMN_COMMENT, column_.SRS_ID, extension.ENGINE_ATTRIBUTE, extension.SECONDARY_ENGINE_ATTRIBUTE FROM information_schema.COLUMNS column_ LEFT JOIN information_schema.COLUMNS_EXTENSIONS extension ON extension.TABLE_SCHEMA = column_.TABLE_SCHEMA AND extension.TABLE_NAME = column_.TABLE_NAME AND extension.COLUMN_NAME = column_.COLUMN_NAME WHERE {} ORDER BY column_.TABLE_SCHEMA, column_.TABLE_NAME, column_.ORDINAL_POSITION",
        schema_filter(source, "column_.TABLE_SCHEMA")
    ))?;
    let mut grouped = BTreeMap::new();
    for row in values {
        let extra = row.required::<String>("EXTRA")?;
        let masking_policy_configured = extra.to_ascii_uppercase().contains("MASKING POLICY");
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
                visible: Some(!extra.to_ascii_uppercase().contains("INVISIBLE")),
                extra,
                generation_expression: nonempty(row.optional("GENERATION_EXPRESSION")?),
                character_set: row.optional("CHARACTER_SET_NAME")?,
                collation: row.optional("COLLATION_NAME")?,
                comment: nonempty(row.optional("COLUMN_COMMENT")?),
                srs_id: row.optional("SRS_ID")?,
                engine_attribute: nonempty(row.optional("ENGINE_ATTRIBUTE")?),
                secondary_engine_attribute: nonempty(row.optional("SECONDARY_ENGINE_ATTRIBUTE")?),
                masking_policy_configured,
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
        "SELECT tc.TABLE_SCHEMA, tc.TABLE_NAME, tc.CONSTRAINT_NAME, tc.CONSTRAINT_TYPE, kcu.COLUMN_NAME, kcu.ORDINAL_POSITION, kcu.REFERENCED_TABLE_SCHEMA, kcu.REFERENCED_TABLE_NAME, kcu.REFERENCED_COLUMN_NAME, rc.MATCH_OPTION, rc.UPDATE_RULE, rc.DELETE_RULE, cc.CHECK_CLAUSE, tc.ENFORCED, extension.ENGINE_ATTRIBUTE, extension.SECONDARY_ENGINE_ATTRIBUTE FROM information_schema.TABLE_CONSTRAINTS tc LEFT JOIN information_schema.KEY_COLUMN_USAGE kcu ON kcu.CONSTRAINT_SCHEMA=tc.CONSTRAINT_SCHEMA AND kcu.TABLE_NAME=tc.TABLE_NAME AND kcu.CONSTRAINT_NAME=tc.CONSTRAINT_NAME LEFT JOIN information_schema.REFERENTIAL_CONSTRAINTS rc ON rc.CONSTRAINT_SCHEMA=tc.CONSTRAINT_SCHEMA AND rc.TABLE_NAME=tc.TABLE_NAME AND rc.CONSTRAINT_NAME=tc.CONSTRAINT_NAME LEFT JOIN information_schema.CHECK_CONSTRAINTS cc ON cc.CONSTRAINT_SCHEMA=tc.CONSTRAINT_SCHEMA AND cc.CONSTRAINT_NAME=tc.CONSTRAINT_NAME LEFT JOIN information_schema.TABLE_CONSTRAINTS_EXTENSIONS extension ON extension.CONSTRAINT_SCHEMA=tc.CONSTRAINT_SCHEMA AND extension.TABLE_NAME=tc.TABLE_NAME AND extension.CONSTRAINT_NAME=tc.CONSTRAINT_NAME WHERE {} ORDER BY tc.TABLE_SCHEMA, tc.TABLE_NAME, tc.CONSTRAINT_NAME, kcu.ORDINAL_POSITION",
        schema_filter(source, "tc.TABLE_SCHEMA")
    ))?;
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
        let enforced = row
            .optional::<String>("ENFORCED")?
            .map(|value| semantic_value(&row, "ENFORCED", &value, yes_no))
            .transpose()?;
        let engine_attribute = nonempty(row.optional("ENGINE_ATTRIBUTE")?);
        let secondary_engine_attribute = nonempty(row.optional("SECONDARY_ENGINE_ATTRIBUTE")?);
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
                enforced,
                engine_attribute,
                secondary_engine_attribute,
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
        "SELECT TABLE_SCHEMA, TABLE_NAME, INDEX_NAME, NON_UNIQUE, SEQ_IN_INDEX, COLUMN_NAME, COLLATION, SUB_PART, INDEX_TYPE, INDEX_COMMENT, COMMENT, IS_VISIBLE, EXPRESSION FROM information_schema.STATISTICS WHERE {} ORDER BY TABLE_SCHEMA, TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX",
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
        let visible = row.optional_semantic("IS_VISIBLE", yes_no)?;
        let comment = nonempty(row.optional("INDEX_COMMENT")?);
        let disabled_reason = nonempty(row.optional("COMMENT")?);
        let item = indexes.entry(key.clone()).or_insert_with(|| Index {
            name: key.2.clone(),
            unique,
            index_type,
            visible,
            comment,
            disabled_reason,
            terms: Vec::new(),
        });
        item.terms.push(IndexTerm {
            position: row.required("SEQ_IN_INDEX")?,
            column: row.optional("COLUMN_NAME")?,
            expression: row.optional("EXPRESSION")?,
            prefix_length: row.optional("SUB_PART")?,
            sort_order: row.optional_semantic("COLLATION", index_sort_order)?,
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
        "SELECT TABLE_SCHEMA, TABLE_NAME, PARTITION_NAME, SUBPARTITION_NAME, PARTITION_METHOD, SUBPARTITION_METHOD, PARTITION_EXPRESSION, SUBPARTITION_EXPRESSION, PARTITION_DESCRIPTION, PARTITION_ORDINAL_POSITION, SUBPARTITION_ORDINAL_POSITION, TABLESPACE_NAME, PARTITION_COMMENT, NODEGROUP FROM information_schema.PARTITIONS WHERE PARTITION_NAME IS NOT NULL AND {} ORDER BY TABLE_SCHEMA, TABLE_NAME, PARTITION_ORDINAL_POSITION, SUBPARTITION_ORDINAL_POSITION",
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
                subpartition_method: row.optional("SUBPARTITION_METHOD")?,
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
            let create: String = rows(connection, source, "view definitions", &sql)?
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
                kind: if create
                    .to_ascii_uppercase()
                    .contains("JSON RELATIONAL DUALITY VIEW")
                {
                    ViewKind::JsonRelationalDuality
                } else {
                    ViewKind::Sql
                },
                definition: row.required("VIEW_DEFINITION")?,
                check_option: row.semantic("CHECK_OPTION", view_check_option)?,
                updatable: row.semantic("IS_UPDATABLE", yes_no)?,
                security: row.semantic("SECURITY_TYPE", sql_security)?,
                definer: row.required("DEFINER")?,
                character_set: row.required("CHARACTER_SET_CLIENT")?,
                collation: row.required("COLLATION_CONNECTION")?,
                create_statement: create,
                duality: None,
            })
        })
        .collect()
}

fn attach_json_duality_metadata(
    connection: &mut Conn,
    source: &MysqlSource,
    views: &mut [View],
) -> Result<(), IntrospectionError> {
    let values = rows(connection, source, "JSON duality views", &format!(
        "SELECT TABLE_SCHEMA, TABLE_NAME, JSON_COLUMN_NAME, ROOT_TABLE_SCHEMA, ROOT_TABLE_NAME, ALLOW_INSERT, ALLOW_UPDATE, ALLOW_DELETE, READ_ONLY, STATUS FROM information_schema.JSON_DUALITY_VIEWS WHERE {} ORDER BY TABLE_SCHEMA, TABLE_NAME",
        schema_filter(source, "TABLE_SCHEMA")
    ))?;
    let mut metadata = BTreeMap::new();
    for row in values {
        metadata.insert(
            (row.required("TABLE_SCHEMA")?, row.required("TABLE_NAME")?),
            JsonDualityView {
                json_column_name: row.required("JSON_COLUMN_NAME")?,
                root_table_schema: row.required("ROOT_TABLE_SCHEMA")?,
                root_table_name: row.required("ROOT_TABLE_NAME")?,
                allow_insert: row.required::<u64>("ALLOW_INSERT")? != 0,
                allow_update: row.required::<u64>("ALLOW_UPDATE")? != 0,
                allow_delete: row.required::<u64>("ALLOW_DELETE")? != 0,
                read_only: row.required::<u64>("READ_ONLY")? != 0,
                status: row.semantic("STATUS", json_duality_view_status)?,
                tables: Vec::new(),
                columns: Vec::new(),
                links: Vec::new(),
            },
        );
    }

    let values = rows(connection, source, "JSON duality tables", &format!(
        "SELECT TABLE_SCHEMA, TABLE_NAME, REFERENCED_TABLE_SCHEMA, REFERENCED_TABLE_NAME, WHERE_CLAUSE, ALLOW_INSERT, ALLOW_UPDATE, ALLOW_DELETE, READ_ONLY, IS_ROOT_TABLE, REFERENCED_TABLE_ID, REFERENCED_TABLE_PARENT_ID, REFERENCED_TABLE_PARENT_RELATIONSHIP FROM information_schema.JSON_DUALITY_VIEW_TABLES WHERE {} ORDER BY TABLE_SCHEMA, TABLE_NAME, REFERENCED_TABLE_ID",
        schema_filter(source, "TABLE_SCHEMA")
    ))?;
    for row in values {
        let key = (row.required("TABLE_SCHEMA")?, row.required("TABLE_NAME")?);
        if let Some(view) = metadata.get_mut(&key) {
            view.tables.push(JsonDualityTable {
                schema: row.required("REFERENCED_TABLE_SCHEMA")?,
                name: row.required("REFERENCED_TABLE_NAME")?,
                where_clause: nonempty(row.optional("WHERE_CLAUSE")?),
                allow_insert: row.required::<u64>("ALLOW_INSERT")? != 0,
                allow_update: row.required::<u64>("ALLOW_UPDATE")? != 0,
                allow_delete: row.required::<u64>("ALLOW_DELETE")? != 0,
                read_only: row.required::<u64>("READ_ONLY")? != 0,
                root: row.required::<u64>("IS_ROOT_TABLE")? != 0,
                id: row.required("REFERENCED_TABLE_ID")?,
                parent_id: row.optional("REFERENCED_TABLE_PARENT_ID")?,
                parent_relationship: nonempty(
                    row.optional("REFERENCED_TABLE_PARENT_RELATIONSHIP")?,
                ),
            });
        }
    }

    let values = rows(connection, source, "JSON duality columns", &format!(
        "SELECT TABLE_SCHEMA, TABLE_NAME, REFERENCED_TABLE_SCHEMA, REFERENCED_TABLE_NAME, IS_ROOT_TABLE, REFERENCED_TABLE_ID, REFERENCED_COLUMN_NAME, JSON_KEY_NAME, ALLOW_INSERT, ALLOW_UPDATE, ALLOW_DELETE, READ_ONLY FROM information_schema.JSON_DUALITY_VIEW_COLUMNS WHERE {} ORDER BY TABLE_SCHEMA, TABLE_NAME, REFERENCED_TABLE_ID, JSON_KEY_NAME, REFERENCED_COLUMN_NAME",
        schema_filter(source, "TABLE_SCHEMA")
    ))?;
    for row in values {
        let key = (row.required("TABLE_SCHEMA")?, row.required("TABLE_NAME")?);
        if let Some(view) = metadata.get_mut(&key) {
            view.columns.push(JsonDualityColumn {
                table_schema: row.required("REFERENCED_TABLE_SCHEMA")?,
                table_name: row.required("REFERENCED_TABLE_NAME")?,
                root_table: row.required::<u64>("IS_ROOT_TABLE")? != 0,
                table_id: row.required("REFERENCED_TABLE_ID")?,
                column_name: row.required("REFERENCED_COLUMN_NAME")?,
                json_key_name: row.required("JSON_KEY_NAME")?,
                allow_insert: row.required::<u64>("ALLOW_INSERT")? != 0,
                allow_update: row.required::<u64>("ALLOW_UPDATE")? != 0,
                allow_delete: row.required::<u64>("ALLOW_DELETE")? != 0,
                read_only: row.required::<u64>("READ_ONLY")? != 0,
            });
        }
    }

    let values = rows(connection, source, "JSON duality links", &format!(
        "SELECT TABLE_SCHEMA, TABLE_NAME, PARENT_TABLE_SCHEMA, PARENT_TABLE_NAME, CHILD_TABLE_SCHEMA, CHILD_TABLE_NAME, PARENT_COLUMN_NAME, CHILD_COLUMN_NAME, JOIN_TYPE, JSON_KEY_NAME FROM information_schema.JSON_DUALITY_VIEW_LINKS WHERE {} ORDER BY TABLE_SCHEMA, TABLE_NAME, PARENT_TABLE_SCHEMA, PARENT_TABLE_NAME, CHILD_TABLE_SCHEMA, CHILD_TABLE_NAME, PARENT_COLUMN_NAME, CHILD_COLUMN_NAME",
        schema_filter(source, "TABLE_SCHEMA")
    ))?;
    for row in values {
        let key = (row.required("TABLE_SCHEMA")?, row.required("TABLE_NAME")?);
        if let Some(view) = metadata.get_mut(&key) {
            view.links.push(JsonDualityLink {
                parent_schema: row.required("PARENT_TABLE_SCHEMA")?,
                parent_table: row.required("PARENT_TABLE_NAME")?,
                child_schema: row.required("CHILD_TABLE_SCHEMA")?,
                child_table: row.required("CHILD_TABLE_NAME")?,
                parent_column: row.required("PARENT_COLUMN_NAME")?,
                child_column: row.required("CHILD_COLUMN_NAME")?,
                join_type: row.required("JOIN_TYPE")?,
                json_key_name: nonempty(row.optional("JSON_KEY_NAME")?),
            });
        }
    }

    for view in views {
        view.duality = metadata.remove(&(view.schema.clone(), view.name.clone()));
    }
    Ok(())
}

fn load_routines(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Routine>, IntrospectionError> {
    let values = rows(connection, source, "routines", &format!(
        "SELECT ROUTINE_SCHEMA, ROUTINE_NAME, ROUTINE_TYPE, DTD_IDENTIFIER, ROUTINE_BODY, ROUTINE_DEFINITION, EXTERNAL_LANGUAGE, IS_DETERMINISTIC, SQL_DATA_ACCESS, SECURITY_TYPE, DEFINER, ROUTINE_COMMENT, SQL_MODE, CHARACTER_SET_CLIENT, COLLATION_CONNECTION, DATABASE_COLLATION FROM information_schema.ROUTINES WHERE {} ORDER BY ROUTINE_SCHEMA, ROUTINE_NAME, ROUTINE_TYPE",
        schema_filter(source, "ROUTINE_SCHEMA")
    ))?;
    values
        .into_iter()
        .map(|row| {
            let schema: String = row.required("ROUTINE_SCHEMA")?;
            let name: String = row.required("ROUTINE_NAME")?;
            let native_kind: String = row.required("ROUTINE_TYPE")?;
            let kind = semantic_value(&row, "ROUTINE_TYPE", &native_kind, routine_kind)?;
            let sql = format!(
                "SHOW CREATE {} `{}`.`{}`",
                native_kind,
                escape_identifier(&schema),
                escape_identifier(&name)
            );
            let create_statement = rows(connection, source, "routine definitions", &sql)?
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
                body: row.required("ROUTINE_BODY")?,
                definition: row.optional("ROUTINE_DEFINITION")?,
                create_statement,
                external_language: row.optional("EXTERNAL_LANGUAGE")?,
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
                libraries: Vec::new(),
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
                mode: row.optional_semantic("PARAMETER_MODE", parameter_mode)?,
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

fn attach_routine_libraries(
    connection: &mut Conn,
    source: &MysqlSource,
    routines: &mut [Routine],
) -> Result<(), IntrospectionError> {
    let values = rows(connection, source, "routine libraries", &format!(
        "SELECT ROUTINE_SCHEMA, ROUTINE_NAME, LIBRARY_SCHEMA, LIBRARY_NAME, LIBRARY_VERSION FROM information_schema.ROUTINE_LIBRARIES WHERE {} ORDER BY ROUTINE_SCHEMA, ROUTINE_NAME, LIBRARY_SCHEMA, LIBRARY_NAME, LIBRARY_VERSION",
        schema_filter(source, "ROUTINE_SCHEMA")
    ))?;
    let mut grouped = BTreeMap::new();
    for row in values {
        grouped
            .entry((
                row.required("ROUTINE_SCHEMA")?,
                row.required("ROUTINE_NAME")?,
            ))
            .or_insert_with(Vec::new)
            .push(RoutineLibrary {
                schema: row.required("LIBRARY_SCHEMA")?,
                name: row.required("LIBRARY_NAME")?,
                version: row.optional("LIBRARY_VERSION")?,
            });
    }
    for routine in routines {
        if let Some(values) = grouped.remove(&(routine.schema.clone(), routine.name.clone())) {
            routine.libraries = values;
        }
    }
    Ok(())
}

fn load_libraries(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Library>, IntrospectionError> {
    rows(connection, source, "libraries", &format!(
        "SELECT LIBRARY_SCHEMA, LIBRARY_NAME, LIBRARY_DEFINITION, LANGUAGE, SQL_MODE, LIBRARY_COMMENT, CREATOR FROM information_schema.LIBRARIES WHERE {} ORDER BY LIBRARY_SCHEMA, LIBRARY_NAME",
        schema_filter(source, "LIBRARY_SCHEMA")
    ))?
    .into_iter()
    .map(|row| {
        Ok(Library {
            schema: row.required("LIBRARY_SCHEMA")?,
            name: row.required("LIBRARY_NAME")?,
            definition: row.required("LIBRARY_DEFINITION")?,
            language: row.required("LANGUAGE")?,
            sql_mode: row.required("SQL_MODE")?,
            comment: nonempty(row.optional("LIBRARY_COMMENT")?),
            creator: row.required("CREATOR")?,
        })
    })
    .collect()
}

fn load_triggers(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Trigger>, IntrospectionError> {
    let values = rows(connection, source, "triggers", &format!(
        "SELECT TRIGGER_SCHEMA, TRIGGER_NAME, EVENT_OBJECT_TABLE, EVENT_MANIPULATION, ACTION_TIMING, ACTION_ORIENTATION, ACTION_STATEMENT, ACTION_ORDER, SQL_MODE, DEFINER, CHARACTER_SET_CLIENT, COLLATION_CONNECTION, DATABASE_COLLATION FROM information_schema.TRIGGERS WHERE {} ORDER BY TRIGGER_SCHEMA, TRIGGER_NAME",
        schema_filter(source, "TRIGGER_SCHEMA")
    ))?;
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
            let create_statement = rows(connection, source, "trigger definitions", &sql)?
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
                event: row.semantic("EVENT_MANIPULATION", trigger_event)?,
                timing: row.semantic("ACTION_TIMING", trigger_timing)?,
                orientation: row.semantic("ACTION_ORIENTATION", trigger_orientation)?,
                statement: row.required("ACTION_STATEMENT")?,
                action_order: row.required("ACTION_ORDER")?,
                sql_mode: row.required("SQL_MODE")?,
                definer: row.required("DEFINER")?,
                character_set: row.required("CHARACTER_SET_CLIENT")?,
                collation: row.required("COLLATION_CONNECTION")?,
                database_collation: row.required("DATABASE_COLLATION")?,
                create_statement,
            })
        })
        .collect()
}

fn load_events(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Event>, IntrospectionError> {
    let values = rows(connection, source, "events", &format!(
        "SELECT EVENT_SCHEMA, EVENT_NAME, DEFINER, TIME_ZONE, EVENT_TYPE, CAST(EXECUTE_AT AS CHAR) AS EXECUTE_AT, INTERVAL_VALUE, INTERVAL_FIELD, CAST(STARTS AS CHAR) AS STARTS, CAST(ENDS AS CHAR) AS ENDS, STATUS, ON_COMPLETION, EVENT_COMMENT, EVENT_DEFINITION, SQL_MODE, ORIGINATOR, CHARACTER_SET_CLIENT, COLLATION_CONNECTION, DATABASE_COLLATION FROM information_schema.EVENTS WHERE {} ORDER BY EVENT_SCHEMA, EVENT_NAME",
        schema_filter(source, "EVENT_SCHEMA")
    ))?;
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
            let create_statement = rows(connection, source, "event definitions", &sql)?
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
                interval_field: row.optional("INTERVAL_FIELD")?,
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
                create_statement,
            })
        })
        .collect()
}

fn load_servers(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<ServerDefinition>, IntrospectionError> {
    rows(
        connection,
        source,
        "server definitions",
        "SELECT Server_name, Wrapper, Host, Db, Username, Port, Socket, Owner, Password <> '' AS PASSWORD_CONFIGURED FROM mysql.servers ORDER BY Server_name",
    )?
    .into_iter()
    .map(|row| {
        Ok(ServerDefinition {
            name: row.required("Server_name")?,
            wrapper: row.required("Wrapper")?,
            host: row.required("Host")?,
            database: row.required("Db")?,
            username: row.required("Username")?,
            port: row.required("Port")?,
            socket: row.required("Socket")?,
            owner: row.required("Owner")?,
            password_configured: row.required::<u64>("PASSWORD_CONFIGURED")? != 0,
        })
    })
    .collect()
}

fn load_spatial_reference_systems(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<SpatialReferenceSystem>, IntrospectionError> {
    rows(
        connection,
        source,
        "spatial reference systems",
        "SELECT SRS_ID, SRS_NAME, ORGANIZATION, ORGANIZATION_COORDSYS_ID, DEFINITION, DESCRIPTION FROM information_schema.ST_SPATIAL_REFERENCE_SYSTEMS WHERE SRS_ID >= 32768 ORDER BY SRS_ID",
    )?
    .into_iter()
    .map(|row| {
        Ok(SpatialReferenceSystem {
            id: row.required("SRS_ID")?,
            name: row.required("SRS_NAME")?,
            organization: row.optional("ORGANIZATION")?,
            organization_id: row.optional("ORGANIZATION_COORDSYS_ID")?,
            definition: row.required("DEFINITION")?,
            description: nonempty(row.optional("DESCRIPTION")?),
        })
    })
    .collect()
}

fn load_tablespaces(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Tablespace>, IntrospectionError> {
    rows(
        connection,
        source,
        "tablespaces",
        "SELECT space.NAME AS TABLESPACE_NAME, 'InnoDB' AS ENGINE, space.ROW_FORMAT, space.PAGE_SIZE, space.AUTOEXTEND_SIZE, space.SPACE_TYPE, space.ENCRYPTION, extension.ENGINE_ATTRIBUTE FROM information_schema.INNODB_TABLESPACES space LEFT JOIN information_schema.TABLESPACES_EXTENSIONS extension ON extension.TABLESPACE_NAME = space.NAME WHERE space.SPACE_TYPE = 'General' ORDER BY space.NAME",
    )?
    .into_iter()
    .map(|row| {
        Ok(Tablespace {
            name: row.required("TABLESPACE_NAME")?,
            engine: row.required("ENGINE")?,
            row_format: nonempty(row.optional("ROW_FORMAT")?),
            page_size: row.optional("PAGE_SIZE")?,
            autoextend_size: row.required("AUTOEXTEND_SIZE")?,
            space_type: row.required("SPACE_TYPE")?,
            encryption: nonempty(row.optional("ENCRYPTION")?),
            engine_attribute: nonempty(row.optional("ENGINE_ATTRIBUTE")?),
            file_locations_redacted: true,
        })
    })
    .collect()
}

fn load_resource_groups(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<ResourceGroup>, IntrospectionError> {
    rows(
        connection,
        source,
        "resource groups",
        "SELECT RESOURCE_GROUP_NAME, RESOURCE_GROUP_TYPE, RESOURCE_GROUP_ENABLED, VCPU_IDS, THREAD_PRIORITY FROM information_schema.RESOURCE_GROUPS WHERE RESOURCE_GROUP_NAME NOT IN ('USR_default','SYS_default') ORDER BY RESOURCE_GROUP_NAME",
    )?
    .into_iter()
    .map(|row| {
        Ok(ResourceGroup {
            name: row.required("RESOURCE_GROUP_NAME")?,
            kind: row.semantic("RESOURCE_GROUP_TYPE", resource_group_kind)?,
            enabled: row.required::<u64>("RESOURCE_GROUP_ENABLED")? != 0,
            virtual_cpus: row.required("VCPU_IDS")?,
            thread_priority: row.required("THREAD_PRIORITY")?,
        })
    })
    .collect()
}

fn load_loadable_functions(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<LoadableFunction>, IntrospectionError> {
    rows(
        connection,
        source,
        "loadable functions",
        "SELECT UDF_NAME AS FUNCTION_NAME, UDF_RETURN_TYPE AS RETURN_TYPE, UDF_LIBRARY AS LIBRARY_NAME, UDF_TYPE AS FUNCTION_KIND FROM performance_schema.user_defined_functions WHERE UDF_NAME IS NOT NULL ORDER BY UDF_NAME",
    )?
    .into_iter()
    .map(|row| {
        Ok(LoadableFunction {
            name: row.required("FUNCTION_NAME")?,
            return_type: row.semantic("RETURN_TYPE", loadable_function_return_type)?,
            library: row.optional("LIBRARY_NAME")?,
            kind: row.semantic("FUNCTION_KIND", loadable_function_kind)?,
        })
    })
    .collect()
}

fn load_plugins(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Plugin>, IntrospectionError> {
    rows(
        connection,
        source,
        "plugins",
        "SELECT PLUGIN_NAME, PLUGIN_VERSION, PLUGIN_STATUS, PLUGIN_TYPE, PLUGIN_TYPE_VERSION, PLUGIN_LIBRARY, PLUGIN_LIBRARY_VERSION, PLUGIN_AUTHOR, PLUGIN_DESCRIPTION, PLUGIN_LICENSE, LOAD_OPTION FROM information_schema.PLUGINS ORDER BY PLUGIN_NAME",
    )?
    .into_iter()
    .map(|row| {
        Ok(Plugin {
            name: row.required("PLUGIN_NAME")?,
            version: row.required("PLUGIN_VERSION")?,
            status: row.semantic("PLUGIN_STATUS", plugin_status)?,
            kind: row.semantic("PLUGIN_TYPE", plugin_kind)?,
            type_version: row.required("PLUGIN_TYPE_VERSION")?,
            library: row.optional("PLUGIN_LIBRARY")?,
            library_version: row.optional("PLUGIN_LIBRARY_VERSION")?,
            author: trimmed_nonempty(row.optional("PLUGIN_AUTHOR")?),
            description: trimmed_nonempty(row.optional("PLUGIN_DESCRIPTION")?),
            license: row.semantic("PLUGIN_LICENSE", plugin_license)?,
            load_option: row.semantic("LOAD_OPTION", plugin_load_option)?,
        })
    })
    .collect()
}

fn load_components(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Component>, IntrospectionError> {
    rows(
        connection,
        source,
        "components",
        "SELECT component_urn FROM mysql.component ORDER BY component_urn",
    )?
    .into_iter()
    .map(|row| {
        Ok(Component {
            urn: row.required("component_urn")?,
        })
    })
    .collect()
}

fn load_accounts(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Account>, IntrospectionError> {
    rows(
        connection,
        source,
        "accounts",
        "SELECT account.User, account.Host, account.account_locked, account.password_expired, account.password_lifetime, account.Password_reuse_history, account.Password_reuse_time, account.password_require_current, COALESCE(JSON_CONTAINS_PATH(account.User_attributes, 'one', '$.additional_password'), 0) AS DUAL_PASSWORD_CONFIGURED, account.ssl_type, NULLIF(account.ssl_cipher, '') AS SSL_CIPHER, NULLIF(account.x509_issuer, '') AS X509_ISSUER, NULLIF(account.x509_subject, '') AS X509_SUBJECT, account.max_questions, account.max_updates, account.max_connections, account.max_user_connections, JSON_UNQUOTE(JSON_EXTRACT(attributes.ATTRIBUTE, '$.comment')) AS USER_COMMENT, COALESCE(JSON_LENGTH(attributes.ATTRIBUTE) > IF(JSON_CONTAINS_PATH(attributes.ATTRIBUTE, 'one', '$.comment'), 1, 0), 0) AS ATTRIBUTES_CONFIGURED FROM mysql.user account LEFT JOIN information_schema.USER_ATTRIBUTES attributes ON attributes.USER=account.User AND attributes.HOST=account.Host WHERE account.User NOT IN ('mysql.infoschema','mysql.session','mysql.sys') ORDER BY account.User, account.Host",
    )?
    .into_iter()
    .map(|row| {
        let require_current = row.optional_semantic("password_require_current", y_n)?;
        Ok(Account {
            user: row.required("User")?,
            host: row.required("Host")?,
            authentication_factors: Vec::new(),
            locked: row.semantic("account_locked", y_n)?,
            password_expired: row.semantic("password_expired", y_n)?,
            password_lifetime_days: row.optional("password_lifetime")?,
            password_reuse_history: row.optional("Password_reuse_history")?,
            password_reuse_interval_days: row.optional("Password_reuse_time")?,
            require_current_password: require_current,
            dual_password_configured: row.required::<u64>("DUAL_PASSWORD_CONFIGURED")? != 0,
            tls_requirement: row.semantic("ssl_type", tls_requirement)?,
            tls_cipher: row.optional("SSL_CIPHER")?,
            x509_issuer: row.optional("X509_ISSUER")?,
            x509_subject: row.optional("X509_SUBJECT")?,
            max_queries_per_hour: row.required("max_questions")?,
            max_updates_per_hour: row.required("max_updates")?,
            max_connections_per_hour: row.required("max_connections")?,
            max_user_connections: row.required("max_user_connections")?,
            comment: nonempty(row.optional("USER_COMMENT")?),
            attributes_configured: row.required::<u64>("ATTRIBUTES_CONFIGURED")? != 0,
        })
    })
    .collect()
}

fn attach_authentication_factors(
    connection: &mut Conn,
    source: &MysqlSource,
    accounts: &mut [Account],
) -> Result<(), IntrospectionError> {
    let values = rows(
        connection,
        source,
        "authentication factors",
        r#"
SELECT User, Host, 1 AS FACTOR_POSITION, plugin AS PLUGIN,
       authentication_string <> '' AS CREDENTIAL_CONFIGURED,
       0 AS PASSWORDLESS, 0 AS REGISTRATION_REQUIRED
FROM mysql.user
WHERE User NOT IN ('mysql.infoschema','mysql.session','mysql.sys')
UNION ALL
SELECT account.User, account.Host, factor.factor_position + 1,
       JSON_UNQUOTE(JSON_EXTRACT(factor.definition, '$.plugin')),
       COALESCE(JSON_UNQUOTE(JSON_EXTRACT(factor.definition, '$.authentication_string')) <> '', 0),
       COALESCE(JSON_EXTRACT(factor.definition, '$.passwordless') = CAST(1 AS JSON), 0),
       COALESCE(JSON_EXTRACT(factor.definition, '$.requires_registration') = CAST(1 AS JSON), 0)
FROM mysql.user AS account
JOIN JSON_TABLE(
    account.User_attributes,
    '$.multi_factor_authentication[*]' COLUMNS(
        factor_position FOR ORDINALITY,
        definition JSON PATH '$'
    )
) AS factor
ORDER BY User, Host, FACTOR_POSITION
"#,
    )?;
    let mut grouped = BTreeMap::new();
    for row in values {
        grouped
            .entry((row.required("User")?, row.required("Host")?))
            .or_insert_with(Vec::new)
            .push(AuthenticationFactor {
                position: row.required("FACTOR_POSITION")?,
                plugin: row.required("PLUGIN")?,
                credential_configured: row.required::<u64>("CREDENTIAL_CONFIGURED")? != 0,
                passwordless: row.required::<u64>("PASSWORDLESS")? != 0,
                registration_required: row.required::<u64>("REGISTRATION_REQUIRED")? != 0,
            });
    }
    for account in accounts {
        account.authentication_factors = grouped
            .remove(&(account.user.clone(), account.host.clone()))
            .unwrap_or_default();
    }
    Ok(())
}

fn load_role_grants(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<RoleGrant>, IntrospectionError> {
    rows(
        connection,
        source,
        "role grants",
        "SELECT FROM_USER, FROM_HOST, TO_USER, TO_HOST, WITH_ADMIN_OPTION FROM mysql.role_edges ORDER BY FROM_USER, FROM_HOST, TO_USER, TO_HOST",
    )?
    .into_iter()
    .map(|row| {
        Ok(RoleGrant {
            role_user: row.required("FROM_USER")?,
            role_host: row.required("FROM_HOST")?,
            member_user: row.required("TO_USER")?,
            member_host: row.required("TO_HOST")?,
            admin_option: row.semantic("WITH_ADMIN_OPTION", y_n)?,
        })
    })
    .collect()
}

fn load_default_roles(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<DefaultRole>, IntrospectionError> {
    rows(
        connection,
        source,
        "default roles",
        "SELECT USER, HOST, DEFAULT_ROLE_USER, DEFAULT_ROLE_HOST FROM mysql.default_roles ORDER BY USER, HOST, DEFAULT_ROLE_USER, DEFAULT_ROLE_HOST",
    )?
    .into_iter()
    .map(|row| {
        Ok(DefaultRole {
            user: row.required("USER")?,
            host: row.required("HOST")?,
            role_user: row.required("DEFAULT_ROLE_USER")?,
            role_host: row.required("DEFAULT_ROLE_HOST")?,
        })
    })
    .collect()
}

fn load_privileges(
    connection: &mut Conn,
    source: &MysqlSource,
) -> Result<Vec<Privilege>, IntrospectionError> {
    rows(
        connection,
        source,
        "privileges",
        r#"
SELECT GRANTEE, OBJECT_TYPE, OBJECT_IDENTITY, PRIVILEGE_TYPE, IS_GRANTABLE
FROM (
    SELECT GRANTEE, 'global' AS OBJECT_TYPE, '*.*' AS OBJECT_IDENTITY,
           PRIVILEGE_TYPE, IS_GRANTABLE
    FROM information_schema.USER_PRIVILEGES
    UNION ALL
    SELECT GRANTEE, 'schema', TABLE_SCHEMA,
           PRIVILEGE_TYPE, IS_GRANTABLE
    FROM information_schema.SCHEMA_PRIVILEGES
    UNION ALL
    SELECT GRANTEE, 'table', CONCAT(TABLE_SCHEMA, '.', TABLE_NAME),
           PRIVILEGE_TYPE, IS_GRANTABLE
    FROM information_schema.TABLE_PRIVILEGES
    UNION ALL
    SELECT GRANTEE, 'column', CONCAT(TABLE_SCHEMA, '.', TABLE_NAME, '.', COLUMN_NAME),
           PRIVILEGE_TYPE, IS_GRANTABLE
    FROM information_schema.COLUMN_PRIVILEGES
    UNION ALL
    SELECT CONCAT('\'', routine.User, '\'@\'', routine.Host, '\''),
           LOWER(routine.Routine_type), CONCAT(routine.Db, '.', routine.Routine_name),
           privileges.PRIVILEGE_TYPE,
           IF(FIND_IN_SET('Grant', routine.Proc_priv) > 0, 'YES', 'NO')
    FROM mysql.procs_priv AS routine
    JOIN JSON_TABLE(
        CONCAT('["', REPLACE(routine.Proc_priv, ',', '","'), '"]'),
        '$[*]' COLUMNS(PRIVILEGE_TYPE VARCHAR(64) PATH '$')
    ) AS privileges
    WHERE privileges.PRIVILEGE_TYPE <> 'Grant'
    UNION ALL
    SELECT CONCAT('\'', User, '\'@\'', Host, '\''), 'proxy',
           CONCAT('\'', Proxied_user, '\'@\'', Proxied_host, '\''),
           'PROXY', IF(With_grant, 'YES', 'NO')
    FROM mysql.proxies_priv
) AS grants
ORDER BY GRANTEE, OBJECT_TYPE, OBJECT_IDENTITY, PRIVILEGE_TYPE, IS_GRANTABLE
"#,
    )?
    .into_iter()
    .map(|row| {
        Ok(Privilege {
            grantee: row.required("GRANTEE")?,
            object_kind: row.semantic("OBJECT_TYPE", privilege_object_kind)?,
            object_identity: row.required("OBJECT_IDENTITY")?,
            privilege: row.required("PRIVILEGE_TYPE")?,
            grantable: row.semantic("IS_GRANTABLE", yes_no)?,
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

fn constraint_kind(value: &str) -> Option<ConstraintKind> {
    match value {
        "PRIMARY KEY" => Some(ConstraintKind::PrimaryKey),
        "UNIQUE" => Some(ConstraintKind::Unique),
        "FOREIGN KEY" => Some(ConstraintKind::ForeignKey),
        "CHECK" => Some(ConstraintKind::Check),
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

fn view_check_option(value: &str) -> Option<ViewCheckOption> {
    match value {
        "NONE" => Some(ViewCheckOption::None),
        "CASCADED" => Some(ViewCheckOption::Cascaded),
        "LOCAL" => Some(ViewCheckOption::Local),
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

fn trigger_event(value: &str) -> Option<TriggerEvent> {
    match value {
        "INSERT" => Some(TriggerEvent::Insert),
        "UPDATE" => Some(TriggerEvent::Update),
        "DELETE" => Some(TriggerEvent::Delete),
        _ => None,
    }
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
        "REPLICA_SIDE_DISABLED" | "SLAVESIDE_DISABLED" => {
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

fn resource_group_kind(value: &str) -> Option<ResourceGroupKind> {
    match value {
        "USER" => Some(ResourceGroupKind::User),
        "SYSTEM" => Some(ResourceGroupKind::System),
        _ => None,
    }
}

fn json_duality_view_status(value: &str) -> Option<JsonDualityViewStatus> {
    match value {
        "valid" => Some(JsonDualityViewStatus::Valid),
        "invalid" => Some(JsonDualityViewStatus::Invalid),
        _ => None,
    }
}

fn loadable_function_return_type(value: &str) -> Option<LoadableFunctionReturnType> {
    match value {
        "int" | "integer" => Some(LoadableFunctionReturnType::Integer),
        "decimal" => Some(LoadableFunctionReturnType::Decimal),
        "real" => Some(LoadableFunctionReturnType::Real),
        "char" | "character" | "string" => Some(LoadableFunctionReturnType::Character),
        "row" => Some(LoadableFunctionReturnType::Row),
        _ => None,
    }
}

fn loadable_function_kind(value: &str) -> Option<LoadableFunctionKind> {
    match value {
        "function" => Some(LoadableFunctionKind::Scalar),
        "aggregate" => Some(LoadableFunctionKind::Aggregate),
        _ => None,
    }
}

fn plugin_kind(value: &str) -> Option<PluginKind> {
    match value {
        "UDF" => Some(PluginKind::LoadableFunction),
        "STORAGE ENGINE" => Some(PluginKind::StorageEngine),
        "FTPARSER" => Some(PluginKind::FullTextParser),
        "DAEMON" => Some(PluginKind::Daemon),
        "INFORMATION SCHEMA" => Some(PluginKind::InformationSchema),
        "AUDIT" => Some(PluginKind::Audit),
        "REPLICATION" => Some(PluginKind::Replication),
        "AUTHENTICATION" => Some(PluginKind::Authentication),
        "VALIDATE PASSWORD" => Some(PluginKind::PasswordValidation),
        "GROUP REPLICATION" => Some(PluginKind::GroupReplication),
        "KEYRING" => Some(PluginKind::Keyring),
        "CLONE" => Some(PluginKind::Clone),
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

fn plugin_status(value: &str) -> Option<PluginStatus> {
    match value {
        "ACTIVE" => Some(PluginStatus::Active),
        "INACTIVE" => Some(PluginStatus::Inactive),
        "DISABLED" => Some(PluginStatus::Disabled),
        "DELETING" => Some(PluginStatus::Deleting),
        "DELETED" => Some(PluginStatus::Deleted),
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
        json_duality_view_status, loadable_function_kind, loadable_function_return_type,
        plugin_kind, plugin_license, stable_create_statement,
    };
    use crate::{
        JsonDualityViewStatus, LoadableFunctionKind, LoadableFunctionReturnType, PluginKind,
        PluginLicense,
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
    fn decodes_closed_extension_abi_values_semantically() {
        assert_eq!(
            loadable_function_return_type("char"),
            Some(LoadableFunctionReturnType::Character)
        );
        assert_eq!(
            loadable_function_kind("function"),
            Some(LoadableFunctionKind::Scalar)
        );
        assert_eq!(plugin_kind("FTPARSER"), Some(PluginKind::FullTextParser));
        assert_eq!(plugin_license("GPL"), Some(PluginLicense::Gpl));
        assert_eq!(
            json_duality_view_status("valid"),
            Some(JsonDualityViewStatus::Valid)
        );
    }

    #[test]
    fn rejects_unknown_closed_extension_abi_values() {
        assert_eq!(loadable_function_return_type("json"), None);
        assert_eq!(loadable_function_kind("window"), None);
        assert_eq!(plugin_kind("QUERY REWRITER"), None);
        assert_eq!(plugin_license("MIT"), None);
        assert_eq!(json_duality_view_status("stale"), None);
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
