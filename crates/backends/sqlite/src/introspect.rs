//! SQLite catalog introspection and normalization.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use bumpalo::Bump;
use dbmd_core::{SourceId, SourceSnapshot};
use fallible_iterator::FallibleIterator;
use rusqlite::types::Type;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use sqlite3_parser::{
    ast::{
        Cmd, ColumnConstraint, CreateTableBody, DeferSubclause, Expr, ForeignKeyClause,
        InitDeferredPred, RefAct, RefArg, ResolveType, SortOrder, Stmt, TableConstraint,
        TriggerEvent as AstTriggerEvent, TriggerTime as AstTriggerTime,
    },
    lexer::sql::Parser,
};
use thiserror::Error;

use super::catalog::{
    Catalog, Column, ColumnKind, ConflictResolution, Constraint, ConstraintKind, Index,
    IndexOrigin, IndexTarget, IndexTerm, Snapshot, Table, TableKind, Trigger,
    TriggerEvent as CoreTriggerEvent, TriggerTiming as CoreTriggerTiming, View,
};
use dbmd_relational::{
    ForeignKeyAction, ForeignKeyDeferrability, ForeignKeyInitialTiming, ForeignKeyMatch,
    ForeignKeyReference, IndexSortOrder, Namespace,
};

/// A resolved SQLite source ready for introspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqliteSource {
    id: SourceId,
    display_name: Option<String>,
    path: PathBuf,
    attachments: Vec<SqliteAttachment>,
}

impl SqliteSource {
    /// Creates a SQLite source with stable identity and a database path.
    pub fn new(id: SourceId, path: impl Into<PathBuf>) -> Self {
        Self {
            id,
            display_name: None,
            path: path.into(),
            attachments: Vec::new(),
        }
    }

    /// Sets the presentation-only source name.
    #[must_use]
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Returns the stable source identifier.
    #[must_use]
    pub fn id(&self) -> &SourceId {
        &self.id
    }

    /// Returns the SQLite database path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Adds a persistent database attached under `namespace` during introspection.
    ///
    /// # Errors
    ///
    /// Returns [`SqliteSourceError`] for an empty, reserved, duplicated, or
    /// NUL-containing namespace.
    pub fn with_attached_database(
        mut self,
        namespace: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Result<Self, SqliteSourceError> {
        let namespace = namespace.into();
        validate_attachment_namespace(&namespace, &self.attachments)?;
        self.attachments.push(SqliteAttachment {
            namespace,
            path: path.into(),
        });
        Ok(self)
    }
}

/// Introspects one SQLite source into a normalized structural snapshot.
///
/// The database is opened read-only. User tables and their columns are returned
/// in deterministic name and ordinal order.
///
/// # Errors
///
/// Returns [`IntrospectionError::Open`] when the database cannot be opened and
/// [`IntrospectionError::Query`] when required SQLite catalog metadata cannot be read.
pub fn introspect(source: &SqliteSource) -> Result<Snapshot, IntrospectionError> {
    let connection = Connection::open_with_flags(&source.path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| IntrospectionError::Open {
            source_id: source.id.clone(),
            source: error,
        })?;
    connection
        .execute_batch("PRAGMA query_only = ON")
        .map_err(|error| IntrospectionError::Query {
            source_id: source.id.clone(),
            source: error,
        })?;
    for attachment in &source.attachments {
        connection
            .execute(
                "ATTACH DATABASE ?1 AS ?2",
                params![attachment.path.to_string_lossy(), attachment.namespace],
            )
            .map_err(|error| IntrospectionError::Attach {
                source_id: source.id.clone(),
                namespace: attachment.namespace.clone(),
                source: error,
            })?;
    }

    let namespaces = std::iter::once("main".to_string())
        .chain(
            source
                .attachments
                .iter()
                .map(|attachment| attachment.namespace.clone()),
        )
        .collect::<Vec<_>>();
    let mut tables = Vec::new();
    let mut views = Vec::new();
    let mut triggers = Vec::new();
    for namespace in &namespaces {
        let table_entries = read_table_entries(&connection, namespace).map_err(|error| {
            IntrospectionError::Query {
                source_id: source.id.clone(),
                source: error,
            }
        })?;
        tables.extend(
            table_entries
                .iter()
                .map(|entry| read_table(&connection, entry))
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|error| IntrospectionError::Query {
                    source_id: source.id.clone(),
                    source: error,
                })?,
        );
        views.extend(read_views(&connection, namespace).map_err(|error| {
            IntrospectionError::Query {
                source_id: source.id.clone(),
                source: error,
            }
        })?);
        triggers.extend(read_triggers(&connection, namespace).map_err(|error| {
            IntrospectionError::Query {
                source_id: source.id.clone(),
                source: error,
            }
        })?);
    }

    let snapshot = SourceSnapshot::new(
        source.id.clone(),
        Catalog {
            namespaces: namespaces
                .into_iter()
                .map(|name| Namespace::new(name, None))
                .collect(),
            tables,
            views,
            triggers,
        },
    );
    Ok(match &source.display_name {
        Some(name) => snapshot.with_display_name(name),
        None => snapshot,
    })
}

fn read_table_entries(
    connection: &Connection,
    namespace: &str,
) -> rusqlite::Result<Vec<CatalogTable>> {
    let mut statement = connection.prepare(
        "SELECT name, type
         FROM pragma_table_list
         WHERE schema = ?1
           AND type IN ('table', 'virtual', 'shadow')
           AND name NOT GLOB 'sqlite_*'
         ORDER BY name COLLATE BINARY",
    )?;
    let rows = statement
        .query_map([namespace], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let virtual_tables = rows
        .iter()
        .filter(|(_, kind)| kind == "virtual")
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();

    rows.into_iter()
        .map(|(name, kind)| {
            let kind = match kind.as_str() {
                "table" => CatalogTableKind::Ordinary,
                "virtual" => CatalogTableKind::Virtual,
                "shadow" => CatalogTableKind::Shadow {
                    virtual_table: virtual_tables
                        .iter()
                        .filter(|virtual_table| name.starts_with(&format!("{virtual_table}_")))
                        .max_by_key(|virtual_table| virtual_table.len())
                        .cloned(),
                },
                _ => {
                    return Err(metadata_conversion_error(
                        1,
                        Type::Text,
                        UnsupportedTableKind(kind),
                    ));
                }
            };
            Ok(CatalogTable {
                namespace: namespace.to_string(),
                name,
                kind,
            })
        })
        .collect()
}

fn read_table(connection: &Connection, entry: &CatalogTable) -> rusqlite::Result<Table> {
    let table_name = &entry.name;
    let namespace = &entry.namespace;
    let schema = quoted_identifier(namespace);
    let definition = connection.query_row(
        &format!("SELECT sql FROM {schema}.sqlite_schema WHERE type = 'table' AND name = ?1"),
        [table_name],
        |row| row.get::<_, Option<String>>(0),
    )?;
    let parsed_definition = definition
        .as_deref()
        .map(|definition| parse_table_definition(definition, namespace))
        .transpose()?;
    let (without_rowid, strict) = connection.query_row(
        "SELECT wr, strict
             FROM pragma_table_list
             WHERE schema = ?1 AND name = ?2",
        [namespace, table_name],
        |row| Ok((row.get::<_, bool>(0)?, row.get::<_, bool>(1)?)),
    )?;

    let mut statement = connection.prepare(
        "SELECT name, type, \"notnull\", dflt_value, pk, hidden
         FROM pragma_table_xinfo(?1, ?2)
         ORDER BY cid",
    )?;
    let column_rows = statement
        .query_map([table_name, namespace], |row| {
            let name = row.get::<_, String>(0)?;
            let primary_key_ordinal = row.get::<_, u32>(4)?;
            let parsed_column = parsed_definition
                .as_ref()
                .and_then(|definition| definition.column(&name));
            Ok((
                Column {
                    name,
                    data_type: row.get(1)?,
                    nullable: Some(!row.get::<_, bool>(2)?),
                    default: row.get(3)?,
                    comment: None,
                    kind: sqlite_column_kind(row.get(5)?)?,
                    collation: parsed_column
                        .and_then(|column| column.collation.clone())
                        .unwrap_or_else(|| "BINARY".to_string()),
                    generated_expression: parsed_column
                        .and_then(|column| column.generated_expression.clone()),
                },
                primary_key_ordinal,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut primary_key_columns = column_rows
        .iter()
        .filter(|(_, ordinal)| *ordinal != 0)
        .map(|(column, ordinal)| (*ordinal, column.name.clone()))
        .collect::<Vec<_>>();
    primary_key_columns.sort_by_key(|(ordinal, _)| *ordinal);

    let catalog_primary_key = if primary_key_columns.is_empty() {
        Vec::new()
    } else {
        vec![Constraint {
            name: None,
            kind: ConstraintKind::PrimaryKey,
            columns: primary_key_columns
                .into_iter()
                .map(|(_, name)| name)
                .collect(),
            expression: None,
            references: None,
            conflict_resolution: None,
            auto_increment: false,
            declared_on_column: false,
        }]
    };
    let catalog_foreign_keys = read_foreign_keys(connection, namespace, table_name)?;
    let mut constraints = parsed_definition
        .as_ref()
        .map(|definition| definition.constraints.clone())
        .unwrap_or_default();
    merge_catalog_constraints(&mut constraints, catalog_primary_key, catalog_foreign_keys);
    let indexes = read_indexes(connection, namespace, table_name)?;

    let mut columns = column_rows
        .into_iter()
        .map(|(column, _)| column)
        .collect::<Vec<_>>();
    if without_rowid || strict {
        for constraint in constraints
            .iter()
            .filter(|constraint| constraint.kind == ConstraintKind::PrimaryKey)
        {
            for primary_key_column in &constraint.columns {
                if let Some(column) = columns
                    .iter_mut()
                    .find(|column| column.name.eq_ignore_ascii_case(primary_key_column))
                {
                    column.nullable = Some(false);
                }
            }
        }
    }
    if let Some(rowid_alias) = parsed_definition
        .as_ref()
        .and_then(|definition| definition.rowid_alias.as_deref())
    {
        if let Some(column) = columns
            .iter_mut()
            .find(|column| column.name.eq_ignore_ascii_case(rowid_alias))
        {
            column.nullable = Some(false);
        }
    }

    Ok(Table {
        namespace: namespace.clone(),
        name: table_name.to_string(),
        comment: None,
        columns,
        constraints,
        indexes,
        without_rowid,
        strict,
        definition,
        kind: match &entry.kind {
            CatalogTableKind::Ordinary => TableKind::Ordinary,
            CatalogTableKind::Virtual => TableKind::Virtual {
                module: parsed_definition
                    .as_ref()
                    .and_then(|definition| definition.virtual_module.clone())
                    .ok_or_else(|| {
                        metadata_conversion_error(
                            0,
                            Type::Text,
                            MissingVirtualTableModule(table_name.clone()),
                        )
                    })?,
                arguments: parsed_definition
                    .as_ref()
                    .map(|definition| definition.virtual_arguments.clone())
                    .unwrap_or_default(),
            },
            CatalogTableKind::Shadow { virtual_table } => TableKind::Shadow {
                virtual_table: virtual_table.clone(),
            },
        },
    })
}

fn read_views(connection: &Connection, namespace: &str) -> rusqlite::Result<Vec<View>> {
    let schema = quoted_identifier(namespace);
    let mut statement = connection.prepare(&format!(
        "SELECT name, sql
         FROM {schema}.sqlite_schema
         WHERE type = 'view' AND name NOT GLOB 'sqlite_*'
         ORDER BY name COLLATE BINARY"
    ))?;
    let definitions = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    definitions
        .into_iter()
        .map(|(name, definition)| {
            Ok(View {
                namespace: namespace.to_string(),
                columns: read_view_columns(connection, namespace, &name)?,
                name,
                definition,
            })
        })
        .collect()
}

fn read_view_columns(
    connection: &Connection,
    namespace: &str,
    view_name: &str,
) -> rusqlite::Result<Vec<Column>> {
    let mut statement = connection.prepare(
        "SELECT name, type, hidden
         FROM pragma_table_xinfo(?1, ?2)
         ORDER BY cid",
    )?;
    let columns = statement
        .query_map([view_name, namespace], |row| {
            Ok(Column {
                name: row.get(0)?,
                data_type: row.get(1)?,
                nullable: None,
                default: None,
                comment: None,
                kind: sqlite_column_kind(row.get(2)?)?,
                collation: "BINARY".to_string(),
                generated_expression: None,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(columns)
}

fn read_triggers(connection: &Connection, namespace: &str) -> rusqlite::Result<Vec<Trigger>> {
    let schema = quoted_identifier(namespace);
    let mut statement = connection.prepare(&format!(
        "SELECT name, sql
         FROM {schema}.sqlite_schema
         WHERE type = 'trigger' AND name NOT GLOB 'sqlite_*'
         ORDER BY name COLLATE BINARY"
    ))?;
    let triggers = statement
        .query_map([], |row| {
            let name = row.get::<_, String>(0)?;
            let definition = row.get::<_, String>(1)?;
            parse_trigger_definition(namespace, name, definition)
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(triggers)
}

fn parse_trigger_definition(
    namespace: &str,
    name: String,
    definition: String,
) -> rusqlite::Result<Trigger> {
    let (target_namespace, target, timing, event, when_expression) = {
        let bump = Bump::new();
        let mut parser = Parser::new(&bump, definition.as_bytes());
        let command = parser
            .next()
            .map_err(|error| metadata_conversion_error(0, Type::Text, error))?
            .ok_or_else(|| metadata_conversion_error(0, Type::Text, MissingTriggerStatement))?;
        if parser
            .next()
            .map_err(|error| metadata_conversion_error(0, Type::Text, error))?
            .is_some()
        {
            return Err(metadata_conversion_error(
                0,
                Type::Text,
                MultipleTriggerStatements,
            ));
        }
        let Cmd::Stmt(Stmt::CreateTrigger {
            time,
            event,
            tbl_name,
            when_clause,
            ..
        }) = command
        else {
            return Err(metadata_conversion_error(
                0,
                Type::Text,
                ExpectedCreateTrigger,
            ));
        };
        let event = match event {
            AstTriggerEvent::Delete => CoreTriggerEvent::Delete,
            AstTriggerEvent::Insert => CoreTriggerEvent::Insert,
            AstTriggerEvent::Update => CoreTriggerEvent::Update {
                columns: Vec::new(),
            },
            AstTriggerEvent::UpdateOf(columns) => CoreTriggerEvent::Update {
                columns: columns
                    .iter()
                    .map(|column| sqlite_identifier(column.0))
                    .collect(),
            },
        };
        (
            tbl_name
                .db_name
                .as_ref()
                .map(|namespace| sqlite_identifier(namespace.0))
                .unwrap_or_else(|| namespace.to_string()),
            sqlite_identifier(tbl_name.name.0),
            match time.unwrap_or(AstTriggerTime::Before) {
                AstTriggerTime::Before => CoreTriggerTiming::Before,
                AstTriggerTime::After => CoreTriggerTiming::After,
                AstTriggerTime::InsteadOf => CoreTriggerTiming::InsteadOf,
            },
            event,
            when_clause.map(|expression| expression.to_string()),
        )
    };
    Ok(Trigger {
        namespace: namespace.to_string(),
        name,
        target_namespace,
        target,
        timing,
        event,
        when_expression,
        definition,
    })
}

fn parse_table_definition(
    definition: &str,
    namespace: &str,
) -> rusqlite::Result<ParsedTableDefinition> {
    let bump = Bump::new();
    let mut parser = Parser::new(&bump, definition.as_bytes());
    let command = parser
        .next()
        .map_err(|error| metadata_conversion_error(0, Type::Text, error))?
        .ok_or_else(|| metadata_conversion_error(0, Type::Text, MissingTableStatement))?;
    if parser
        .next()
        .map_err(|error| metadata_conversion_error(0, Type::Text, error))?
        .is_some()
    {
        return Err(metadata_conversion_error(
            0,
            Type::Text,
            MultipleTableStatements,
        ));
    }

    let body = match command {
        Cmd::Stmt(Stmt::CreateTable { body, .. }) => body,
        Cmd::Stmt(Stmt::CreateVirtualTable {
            module_name, args, ..
        }) => {
            return Ok(ParsedTableDefinition {
                virtual_module: Some(sqlite_identifier(module_name.0)),
                virtual_arguments: args
                    .map(|arguments| {
                        arguments
                            .iter()
                            .map(|argument| (*argument).to_string())
                            .collect()
                    })
                    .unwrap_or_default(),
                ..ParsedTableDefinition::default()
            });
        }
        _ => {
            return Err(metadata_conversion_error(
                0,
                Type::Text,
                ExpectedCreateTable,
            ));
        }
    };

    let CreateTableBody::ColumnsAndConstraints {
        columns,
        constraints: table_constraints,
        ..
    } = body
    else {
        return Ok(ParsedTableDefinition::default());
    };

    let mut parsed = ParsedTableDefinition::default();
    for column in &columns {
        let column_name = sqlite_identifier(column.col_name.0);
        let mut parsed_column = ParsedColumn {
            name: column_name.clone(),
            data_type: column
                .col_type
                .as_ref()
                .map(|column_type| column_type.name.to_string()),
            collation: None,
            generated_expression: None,
        };

        for named_constraint in column.constraints {
            let name = named_constraint
                .name
                .as_ref()
                .map(|name| sqlite_identifier(name.0));
            match &named_constraint.constraint {
                ColumnConstraint::PrimaryKey {
                    order,
                    conflict_clause,
                    auto_increment,
                } => {
                    parsed.constraints.push(new_sqlite_constraint(
                        name,
                        ConstraintKind::PrimaryKey,
                        vec![column_name.clone()],
                        None,
                        None,
                        parsed_sqlite_constraint(*conflict_clause, *auto_increment, true),
                    ));
                    if column
                        .col_type
                        .as_ref()
                        .is_some_and(|column_type| column_type.name.eq_ignore_ascii_case("INTEGER"))
                        && !matches!(order, Some(SortOrder::Desc))
                    {
                        parsed.rowid_alias = Some(column_name.clone());
                    }
                }
                ColumnConstraint::NotNull {
                    nullable,
                    conflict_clause,
                } if !nullable => parsed.constraints.push(new_sqlite_constraint(
                    name,
                    ConstraintKind::NotNull,
                    vec![column_name.clone()],
                    None,
                    None,
                    parsed_sqlite_constraint(*conflict_clause, false, true),
                )),
                ColumnConstraint::Unique(conflict_clause) => {
                    parsed.constraints.push(new_sqlite_constraint(
                        name,
                        ConstraintKind::Unique,
                        vec![column_name.clone()],
                        None,
                        None,
                        parsed_sqlite_constraint(*conflict_clause, false, true),
                    ));
                }
                ColumnConstraint::Check(expression) => {
                    parsed.constraints.push(new_sqlite_constraint(
                        name,
                        ConstraintKind::Check,
                        vec![column_name.clone()],
                        Some(expression.to_string()),
                        None,
                        parsed_sqlite_constraint(None, false, true),
                    ));
                }
                ColumnConstraint::Collate { collation_name } => {
                    parsed_column.collation = Some(sqlite_identifier(collation_name.0));
                }
                ColumnConstraint::ForeignKey {
                    clause,
                    defer_clause,
                } => parsed.constraints.push(new_sqlite_constraint(
                    name,
                    ConstraintKind::ForeignKey,
                    vec![column_name.clone()],
                    None,
                    Some(parse_foreign_key_reference(
                        clause,
                        defer_clause.as_ref(),
                        namespace,
                    )),
                    parsed_sqlite_constraint(None, false, true),
                )),
                ColumnConstraint::Generated { expr, .. } => {
                    parsed_column.generated_expression = Some(expr.to_string());
                }
                ColumnConstraint::Defer(defer_clause) => {
                    if let Some(reference) = parsed
                        .constraints
                        .iter_mut()
                        .rev()
                        .find(|constraint| {
                            constraint.kind == ConstraintKind::ForeignKey
                                && constraint.columns == [column_name.as_str()]
                        })
                        .and_then(|constraint| constraint.references.as_mut())
                    {
                        reference.deferrability = parse_deferrability(Some(defer_clause));
                    }
                }
                ColumnConstraint::Default(_) | ColumnConstraint::NotNull { .. } => {}
            }
        }
        parsed.columns.push(parsed_column);
    }

    if let Some(table_constraints) = table_constraints {
        for named_constraint in table_constraints {
            let name = named_constraint
                .name
                .as_ref()
                .map(|name| sqlite_identifier(name.0));
            match &named_constraint.constraint {
                TableConstraint::PrimaryKey {
                    columns,
                    auto_increment,
                    conflict_clause,
                } => {
                    let column_names = columns
                        .iter()
                        .map(|column| constraint_column_name(&column.expr))
                        .collect::<rusqlite::Result<Vec<_>>>()?;
                    if column_names.len() == 1 && parsed.column_has_integer_type(&column_names[0]) {
                        parsed.rowid_alias = Some(column_names[0].clone());
                    }
                    parsed.constraints.push(new_sqlite_constraint(
                        name,
                        ConstraintKind::PrimaryKey,
                        column_names,
                        None,
                        None,
                        parsed_sqlite_constraint(*conflict_clause, *auto_increment, false),
                    ));
                }
                TableConstraint::Unique {
                    columns,
                    conflict_clause,
                } => parsed.constraints.push(new_sqlite_constraint(
                    name,
                    ConstraintKind::Unique,
                    columns
                        .iter()
                        .map(|column| constraint_column_name(&column.expr))
                        .collect::<rusqlite::Result<Vec<_>>>()?,
                    None,
                    None,
                    parsed_sqlite_constraint(*conflict_clause, false, false),
                )),
                TableConstraint::Check(expression, conflict_clause) => {
                    parsed.constraints.push(new_sqlite_constraint(
                        name,
                        ConstraintKind::Check,
                        Vec::new(),
                        Some(expression.to_string()),
                        None,
                        parsed_sqlite_constraint(*conflict_clause, false, false),
                    ));
                }
                TableConstraint::ForeignKey {
                    columns,
                    clause,
                    defer_clause,
                } => parsed.constraints.push(new_sqlite_constraint(
                    name,
                    ConstraintKind::ForeignKey,
                    columns
                        .iter()
                        .map(|column| sqlite_identifier(column.col_name.0))
                        .collect(),
                    None,
                    Some(parse_foreign_key_reference(
                        clause,
                        defer_clause.as_ref(),
                        namespace,
                    )),
                    parsed_sqlite_constraint(None, false, false),
                )),
            }
        }
    }

    Ok(parsed)
}

fn constraint_column_name(expression: &Expr<'_>) -> rusqlite::Result<String> {
    match expression {
        Expr::Id(identifier) => Ok(sqlite_identifier(identifier.0)),
        Expr::Name(name) => Ok(sqlite_identifier(name.0)),
        Expr::Collate(expression, _) => constraint_column_name(expression),
        expression => Err(metadata_conversion_error(
            0,
            Type::Text,
            InvalidConstraintColumn(expression.to_string()),
        )),
    }
}

fn parse_foreign_key_reference(
    clause: &ForeignKeyClause<'_>,
    defer_clause: Option<&DeferSubclause>,
    namespace: &str,
) -> ForeignKeyReference {
    let mut reference = ForeignKeyReference::new(
        namespace,
        sqlite_identifier(clause.tbl_name.0),
        clause
            .columns
            .map(|columns| {
                columns
                    .iter()
                    .map(|column| sqlite_identifier(column.col_name.0))
                    .collect()
            })
            .unwrap_or_default(),
    )
    .with_deferrability(parse_deferrability(defer_clause));
    for argument in clause.args {
        match argument {
            RefArg::OnDelete(action) => reference.on_delete = parse_reference_action(*action),
            RefArg::OnUpdate(action) => reference.on_update = parse_reference_action(*action),
            RefArg::Match(name) => {
                reference.match_type = Some(sqlite_foreign_key_match(&sqlite_identifier(name.0)));
            }
            RefArg::OnInsert(_) => {}
        }
    }
    reference
}

fn sqlite_foreign_key_match(name: &str) -> ForeignKeyMatch {
    if name.eq_ignore_ascii_case("simple") {
        ForeignKeyMatch::Simple
    } else if name.eq_ignore_ascii_case("partial") {
        ForeignKeyMatch::Partial
    } else if name.eq_ignore_ascii_case("full") {
        ForeignKeyMatch::Full
    } else {
        ForeignKeyMatch::Named(name.to_string())
    }
}

fn parse_reference_action(action: RefAct) -> ForeignKeyAction {
    match action {
        RefAct::SetNull => ForeignKeyAction::SetNull,
        RefAct::SetDefault => ForeignKeyAction::SetDefault,
        RefAct::Cascade => ForeignKeyAction::Cascade,
        RefAct::Restrict => ForeignKeyAction::Restrict,
        RefAct::NoAction => ForeignKeyAction::NoAction,
    }
}

fn parse_deferrability(clause: Option<&DeferSubclause>) -> ForeignKeyDeferrability {
    clause.map_or_else(ForeignKeyDeferrability::default, |clause| {
        ForeignKeyDeferrability::new(
            clause.deferrable,
            match clause.init_deferred {
                Some(InitDeferredPred::InitiallyDeferred) => ForeignKeyInitialTiming::Deferred,
                Some(InitDeferredPred::InitiallyImmediate) | None => {
                    ForeignKeyInitialTiming::Immediate
                }
            },
        )
    })
}

fn new_sqlite_constraint(
    name: Option<String>,
    kind: ConstraintKind,
    columns: Vec<String>,
    expression: Option<String>,
    references: Option<ForeignKeyReference>,
    sqlite: ConstraintSemantics,
) -> Constraint {
    Constraint {
        name,
        kind,
        columns,
        expression,
        references,
        conflict_resolution: sqlite.conflict_resolution,
        auto_increment: sqlite.auto_increment,
        declared_on_column: sqlite.declared_on_column,
    }
}

fn parsed_sqlite_constraint(
    conflict_resolution: Option<ResolveType>,
    auto_increment: bool,
    declared_on_column: bool,
) -> ConstraintSemantics {
    ConstraintSemantics {
        conflict_resolution: conflict_resolution.map(parse_conflict_resolution),
        auto_increment,
        declared_on_column,
    }
}

fn parse_conflict_resolution(resolution: ResolveType) -> ConflictResolution {
    match resolution {
        ResolveType::Rollback => ConflictResolution::Rollback,
        ResolveType::Abort => ConflictResolution::Abort,
        ResolveType::Fail => ConflictResolution::Fail,
        ResolveType::Ignore => ConflictResolution::Ignore,
        ResolveType::Replace => ConflictResolution::Replace,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConstraintSemantics {
    conflict_resolution: Option<ConflictResolution>,
    auto_increment: bool,
    declared_on_column: bool,
}

fn merge_catalog_constraints(
    constraints: &mut Vec<Constraint>,
    catalog_primary_key: Vec<Constraint>,
    catalog_foreign_keys: Vec<Constraint>,
) {
    if !constraints
        .iter()
        .any(|constraint| constraint.kind == ConstraintKind::PrimaryKey)
    {
        constraints.extend(catalog_primary_key);
    }

    let mut matched_constraints = BTreeSet::new();
    for catalog_constraint in catalog_foreign_keys {
        let parsed = constraints
            .iter_mut()
            .enumerate()
            .find(|(index, constraint)| {
                !matched_constraints.contains(index)
                    && foreign_keys_match(constraint, &catalog_constraint)
            });
        if let Some((index, parsed)) = parsed {
            matched_constraints.insert(index);
            if let (Some(parsed_reference), Some(catalog_reference)) =
                (&mut parsed.references, catalog_constraint.references)
            {
                parsed_reference.columns = catalog_reference.columns;
                parsed_reference.on_update = catalog_reference.on_update;
                parsed_reference.on_delete = catalog_reference.on_delete;
            }
        } else {
            constraints.push(catalog_constraint);
        }
    }
}

fn foreign_keys_match(parsed: &Constraint, catalog: &Constraint) -> bool {
    if parsed.kind != ConstraintKind::ForeignKey || parsed.columns != catalog.columns {
        return false;
    }
    let (Some(parsed_reference), Some(catalog_reference)) =
        (&parsed.references, &catalog.references)
    else {
        return false;
    };
    parsed_reference.table == catalog_reference.table
        && (parsed_reference.columns.is_empty()
            || parsed_reference.columns == catalog_reference.columns)
        && parsed_reference.on_update == catalog_reference.on_update
        && parsed_reference.on_delete == catalog_reference.on_delete
}

fn sqlite_identifier(value: &str) -> String {
    let Some(first) = value.chars().next() else {
        return String::new();
    };
    let Some(last) = value.chars().last() else {
        return String::new();
    };
    let closing = match first {
        '"' => '"',
        '\'' => '\'',
        '`' => '`',
        '[' => ']',
        _ => return value.to_string(),
    };
    if last != closing || value.len() < 2 {
        return value.to_string();
    }
    let inner = &value[first.len_utf8()..value.len() - last.len_utf8()];
    if first == '[' {
        inner.to_string()
    } else {
        inner.replace(&format!("{closing}{closing}"), &closing.to_string())
    }
}

#[derive(Default)]
struct ParsedTableDefinition {
    columns: Vec<ParsedColumn>,
    constraints: Vec<Constraint>,
    rowid_alias: Option<String>,
    virtual_module: Option<String>,
    virtual_arguments: Vec<String>,
}

impl ParsedTableDefinition {
    fn column(&self, name: &str) -> Option<&ParsedColumn> {
        self.columns
            .iter()
            .find(|column| column.name.eq_ignore_ascii_case(name))
    }

    fn column_has_integer_type(&self, name: &str) -> bool {
        self.column(name)
            .and_then(|column| column.data_type.as_deref())
            .is_some_and(|data_type| data_type.eq_ignore_ascii_case("INTEGER"))
    }
}

struct ParsedColumn {
    name: String,
    data_type: Option<String>,
    collation: Option<String>,
    generated_expression: Option<String>,
}

struct CatalogTable {
    namespace: String,
    name: String,
    kind: CatalogTableKind,
}

enum CatalogTableKind {
    Ordinary,
    Virtual,
    Shadow { virtual_table: Option<String> },
}

#[derive(Debug, Error)]
#[error("unsupported SQLite table kind `{0}`")]
struct UnsupportedTableKind(String);

#[derive(Debug, Error)]
#[error("SQLite virtual table `{0}` has no stored module name")]
struct MissingVirtualTableModule(String);

#[derive(Debug, Clone, PartialEq, Eq)]
struct SqliteAttachment {
    namespace: String,
    path: PathBuf,
}

fn validate_attachment_namespace(
    namespace: &str,
    attachments: &[SqliteAttachment],
) -> Result<(), SqliteSourceError> {
    if namespace.is_empty() {
        return Err(SqliteSourceError::EmptyNamespace);
    }
    if namespace.contains('\0') {
        return Err(SqliteSourceError::NamespaceContainsNul);
    }
    if namespace.eq_ignore_ascii_case("main") || namespace.eq_ignore_ascii_case("temp") {
        return Err(SqliteSourceError::ReservedNamespace(namespace.to_string()));
    }
    if attachments
        .iter()
        .any(|attachment| attachment.namespace.eq_ignore_ascii_case(namespace))
    {
        return Err(SqliteSourceError::DuplicateNamespace(namespace.to_string()));
    }
    Ok(())
}

fn quoted_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

/// Why a SQLite source attachment could not be configured.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum SqliteSourceError {
    /// The attachment namespace was empty.
    #[error("attached SQLite namespace cannot be empty")]
    EmptyNamespace,
    /// The attachment namespace contained a NUL byte.
    #[error("attached SQLite namespace cannot contain NUL")]
    NamespaceContainsNul,
    /// The attachment attempted to use `main` or `temp`.
    #[error("SQLite namespace `{0}` is reserved")]
    ReservedNamespace(String),
    /// The attachment namespace duplicated another configured attachment.
    #[error("SQLite namespace `{0}` is attached more than once")]
    DuplicateNamespace(String),
}

#[derive(Debug, Error)]
#[error("stored SQLite table definition contained no statement")]
struct MissingTableStatement;

#[derive(Debug, Error)]
#[error("stored SQLite table definition contained multiple statements")]
struct MultipleTableStatements;

#[derive(Debug, Error)]
#[error("stored SQLite table definition was not CREATE TABLE")]
struct ExpectedCreateTable;

#[derive(Debug, Error)]
#[error("SQLite table constraint contained non-column expression `{0}`")]
struct InvalidConstraintColumn(String);

#[derive(Debug, Error)]
#[error("stored SQLite trigger definition contained no statement")]
struct MissingTriggerStatement;

#[derive(Debug, Error)]
#[error("stored SQLite trigger definition contained multiple statements")]
struct MultipleTriggerStatements;

#[derive(Debug, Error)]
#[error("stored SQLite trigger definition was not CREATE TRIGGER")]
struct ExpectedCreateTrigger;

fn read_indexes(
    connection: &Connection,
    namespace: &str,
    table_name: &str,
) -> rusqlite::Result<Vec<Index>> {
    let mut statement = connection.prepare(
        "SELECT name, \"unique\", origin, partial
         FROM pragma_index_list(?1, ?2)
         ORDER BY name COLLATE BINARY",
    )?;
    let catalog_indexes = statement
        .query_map([table_name, namespace], |row| {
            Ok(CatalogIndex {
                name: row.get(0)?,
                unique: row.get(1)?,
                origin: sqlite_index_origin(&row.get::<_, String>(2)?, 2)?,
                partial: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut indexes = Vec::with_capacity(catalog_indexes.len());
    let schema = quoted_identifier(namespace);
    for catalog_index in catalog_indexes {
        let definition = connection
            .query_row(
                &format!(
                    "SELECT sql FROM {schema}.sqlite_schema WHERE type = 'index' AND name = ?1"
                ),
                [&catalog_index.name],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten();
        let parsed = definition
            .as_deref()
            .map(parse_index_definition)
            .transpose()?;
        let terms = read_index_terms(connection, namespace, &catalog_index.name, parsed.as_ref())?;
        let predicate = parsed.and_then(|parsed| parsed.predicate);

        if catalog_index.partial != predicate.is_some() {
            return Err(metadata_conversion_error(
                3,
                Type::Integer,
                InconsistentPartialIndex(catalog_index.name),
            ));
        }

        indexes.push(Index {
            name: catalog_index.name,
            unique: catalog_index.unique,
            terms,
            predicate,
            definition,
            origin: catalog_index.origin,
        });
    }

    Ok(indexes)
}

fn read_index_terms(
    connection: &Connection,
    namespace: &str,
    index_name: &str,
    parsed: Option<&ParsedIndexDefinition>,
) -> rusqlite::Result<Vec<IndexTerm>> {
    let mut statement = connection.prepare(
        "SELECT seqno, cid, name, \"desc\", coll
         FROM pragma_index_xinfo(?1, ?2)
         WHERE key = 1
         ORDER BY seqno",
    )?;
    let terms = statement
        .query_map([index_name, namespace], |row| {
            let sequence = usize::from(row.get::<_, u16>(0)?);
            let column_id = row.get::<_, i64>(1)?;
            let column_name = row.get::<_, Option<String>>(2)?;
            let target = match (column_id, column_name) {
                (-2, None) => {
                    let expression = parsed
                        .and_then(|definition| definition.terms.get(sequence))
                        .cloned()
                        .ok_or_else(|| {
                            metadata_conversion_error(
                                1,
                                Type::Integer,
                                MissingIndexExpression(index_name.to_string(), sequence),
                            )
                        })?;
                    IndexTarget::Expression(expression)
                }
                (-1, None) => IndexTarget::RowId,
                (0.., Some(column)) => IndexTarget::Column(column),
                _ => {
                    return Err(metadata_conversion_error(
                        1,
                        Type::Integer,
                        InvalidIndexTerm(index_name.to_string(), sequence),
                    ));
                }
            };

            Ok(IndexTerm {
                target,
                collation: Some(row.get(4)?),
                order: if row.get::<_, bool>(3)? {
                    IndexSortOrder::Descending
                } else {
                    IndexSortOrder::Ascending
                },
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(terms)
}

fn parse_index_definition(definition: &str) -> rusqlite::Result<ParsedIndexDefinition> {
    let bump = Bump::new();
    let mut parser = Parser::new(&bump, definition.as_bytes());
    let command = parser
        .next()
        .map_err(|error| metadata_conversion_error(0, Type::Text, error))?
        .ok_or_else(|| metadata_conversion_error(0, Type::Text, MissingIndexStatement))?;
    if parser
        .next()
        .map_err(|error| metadata_conversion_error(0, Type::Text, error))?
        .is_some()
    {
        return Err(metadata_conversion_error(
            0,
            Type::Text,
            MultipleIndexStatements,
        ));
    }

    let Cmd::Stmt(Stmt::CreateIndex {
        columns,
        where_clause,
        ..
    }) = command
    else {
        return Err(metadata_conversion_error(
            0,
            Type::Text,
            ExpectedCreateIndex,
        ));
    };

    let terms = columns
        .iter()
        .map(|column| match &column.expr {
            Expr::Collate(expression, _) => expression.to_string(),
            expression => expression.to_string(),
        })
        .collect();
    Ok(ParsedIndexDefinition {
        terms,
        predicate: where_clause.map(|expression| expression.to_string()),
    })
}

fn sqlite_index_origin(value: &str, column_index: usize) -> rusqlite::Result<IndexOrigin> {
    match value {
        "c" => Ok(IndexOrigin::CreateIndex),
        "u" => Ok(IndexOrigin::UniqueConstraint),
        "pk" => Ok(IndexOrigin::PrimaryKey),
        value => Err(metadata_conversion_error(
            column_index,
            Type::Text,
            UnsupportedIndexOrigin(value.to_string()),
        )),
    }
}

fn metadata_conversion_error(
    column_index: usize,
    column_type: Type,
    error: impl std::error::Error + Send + Sync + 'static,
) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(column_index, column_type, Box::new(error))
}

struct CatalogIndex {
    name: String,
    unique: bool,
    origin: IndexOrigin,
    partial: bool,
}

struct ParsedIndexDefinition {
    terms: Vec<String>,
    predicate: Option<String>,
}

#[derive(Debug, Error)]
#[error(
    "SQLite index `{0}` has an expression term missing from its stored definition at position {1}"
)]
struct MissingIndexExpression(String, usize);

#[derive(Debug, Error)]
#[error("SQLite index `{0}` has an invalid catalog term at position {1}")]
struct InvalidIndexTerm(String, usize);

#[derive(Debug, Error)]
#[error("SQLite index `{0}` disagrees with its catalog partial-index flag")]
struct InconsistentPartialIndex(String);

#[derive(Debug, Error)]
#[error("stored SQLite index definition contained no statement")]
struct MissingIndexStatement;

#[derive(Debug, Error)]
#[error("stored SQLite index definition contained multiple statements")]
struct MultipleIndexStatements;

#[derive(Debug, Error)]
#[error("stored SQLite index definition was not CREATE INDEX")]
struct ExpectedCreateIndex;

#[derive(Debug, Error)]
#[error("unsupported SQLite index origin `{0}`")]
struct UnsupportedIndexOrigin(String);

fn read_foreign_keys(
    connection: &Connection,
    namespace: &str,
    table_name: &str,
) -> rusqlite::Result<Vec<Constraint>> {
    let mut statement = connection.prepare(
        "SELECT id, seq, \"table\", \"from\", \"to\", on_update, on_delete
         FROM pragma_foreign_key_list(?1, ?2)
         ORDER BY id, seq",
    )?;
    let rows = statement
        .query_map([table_name, namespace], |row| {
            Ok(ForeignKeyRow {
                id: row.get(0)?,
                sequence: row.get(1)?,
                referenced_table: row.get(2)?,
                column: row.get(3)?,
                referenced_column: row.get(4)?,
                on_update: foreign_key_action(&row.get::<_, String>(5)?, 5)?,
                on_delete: foreign_key_action(&row.get::<_, String>(6)?, 6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut groups = BTreeMap::<u32, ForeignKeyGroup>::new();
    for row in rows {
        let group = groups.entry(row.id).or_insert_with(|| ForeignKeyGroup {
            referenced_table: row.referenced_table.clone(),
            on_update: row.on_update,
            on_delete: row.on_delete,
            columns: Vec::new(),
        });
        group
            .columns
            .push((row.sequence, row.column, row.referenced_column));
    }

    let mut constraints = groups
        .into_values()
        .map(|group| group.into_constraint(connection, namespace))
        .collect::<rusqlite::Result<Vec<_>>>()?;
    constraints.sort_by(|left, right| left.columns.cmp(&right.columns));
    Ok(constraints)
}

struct ForeignKeyRow {
    id: u32,
    sequence: u32,
    referenced_table: String,
    column: String,
    referenced_column: Option<String>,
    on_update: ForeignKeyAction,
    on_delete: ForeignKeyAction,
}

struct ForeignKeyGroup {
    referenced_table: String,
    on_update: ForeignKeyAction,
    on_delete: ForeignKeyAction,
    columns: Vec<(u32, String, Option<String>)>,
}

impl ForeignKeyGroup {
    fn into_constraint(
        mut self,
        connection: &Connection,
        namespace: &str,
    ) -> rusqlite::Result<Constraint> {
        self.columns.sort_by_key(|(sequence, _, _)| *sequence);
        let implicit_target_columns = if self
            .columns
            .iter()
            .any(|(_, _, referenced_column)| referenced_column.is_none())
        {
            Some(read_primary_key_columns(
                connection,
                namespace,
                &self.referenced_table,
            )?)
        } else {
            None
        };
        if implicit_target_columns
            .as_ref()
            .is_some_and(|columns| columns.len() != self.columns.len())
        {
            return Err(metadata_conversion_error(
                4,
                Type::Null,
                InvalidImplicitForeignKeyTarget(self.referenced_table),
            ));
        }

        let mut columns = Vec::with_capacity(self.columns.len());
        let mut referenced_columns = Vec::with_capacity(self.columns.len());
        for (sequence, column, referenced_column) in self.columns {
            columns.push(column);
            let referenced_column = if let Some(referenced_column) = referenced_column {
                referenced_column
            } else {
                let sequence = usize::try_from(sequence)
                    .map_err(|error| metadata_conversion_error(0, Type::Integer, error))?;
                implicit_target_columns
                    .as_ref()
                    .and_then(|columns| columns.get(sequence))
                    .cloned()
                    .ok_or_else(|| {
                        metadata_conversion_error(
                            4,
                            Type::Null,
                            InvalidImplicitForeignKeyTarget(self.referenced_table.clone()),
                        )
                    })?
            };
            referenced_columns.push(referenced_column);
        }

        Ok(Constraint {
            name: None,
            kind: ConstraintKind::ForeignKey,
            columns,
            expression: None,
            references: Some(
                ForeignKeyReference::new(namespace, self.referenced_table, referenced_columns)
                    .with_actions(self.on_update, self.on_delete),
            ),
            conflict_resolution: None,
            auto_increment: false,
            declared_on_column: false,
        })
    }
}

fn read_primary_key_columns(
    connection: &Connection,
    namespace: &str,
    table_name: &str,
) -> rusqlite::Result<Vec<String>> {
    let mut statement = connection.prepare(
        "SELECT name, pk
         FROM pragma_table_xinfo(?1, ?2)
         WHERE pk > 0
         ORDER BY pk",
    )?;
    let columns = statement
        .query_map([table_name, namespace], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(columns)
}

fn foreign_key_action(value: &str, column_index: usize) -> rusqlite::Result<ForeignKeyAction> {
    match value {
        "NO ACTION" => Ok(ForeignKeyAction::NoAction),
        "RESTRICT" => Ok(ForeignKeyAction::Restrict),
        "SET NULL" => Ok(ForeignKeyAction::SetNull),
        "SET DEFAULT" => Ok(ForeignKeyAction::SetDefault),
        "CASCADE" => Ok(ForeignKeyAction::Cascade),
        value => Err(rusqlite::Error::FromSqlConversionFailure(
            column_index,
            Type::Text,
            Box::new(UnsupportedForeignKeyAction(value.to_string())),
        )),
    }
}

#[derive(Debug, Error)]
#[error("SQLite foreign key references an implicit key on `{0}` with incompatible arity")]
struct InvalidImplicitForeignKeyTarget(String);

#[derive(Debug, Error)]
#[error("unsupported SQLite foreign-key action `{0}`")]
struct UnsupportedForeignKeyAction(String);

fn sqlite_column_kind(value: u32) -> rusqlite::Result<ColumnKind> {
    match value {
        0 => Ok(ColumnKind::Normal),
        1 => Ok(ColumnKind::VirtualTableHidden),
        2 => Ok(ColumnKind::VirtualGenerated),
        3 => Ok(ColumnKind::StoredGenerated),
        value => Err(rusqlite::Error::FromSqlConversionFailure(
            5,
            Type::Integer,
            Box::new(UnsupportedSqliteColumnKind(value)),
        )),
    }
}

#[derive(Debug, Error)]
#[error("unsupported SQLite table_xinfo hidden value {0}")]
struct UnsupportedSqliteColumnKind(u32);

/// Why SQLite could not produce a source snapshot.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IntrospectionError {
    /// The configured SQLite database could not be opened read-only.
    #[error("failed to open SQLite source `{source_id}`")]
    Open {
        /// Stable identity of the failing source.
        source_id: SourceId,
        /// Underlying SQLite error.
        #[source]
        source: rusqlite::Error,
    },
    /// A configured attached database could not be opened.
    #[error("failed to attach SQLite namespace `{namespace}` for source `{source_id}`")]
    Attach {
        /// Stable identity of the failing source.
        source_id: SourceId,
        /// Configured namespace that failed.
        namespace: String,
        /// Underlying SQLite error.
        #[source]
        source: rusqlite::Error,
    },
    /// Required SQLite catalog metadata could not be queried.
    #[error("failed to query SQLite source `{source_id}`")]
    Query {
        /// Stable identity of the failing source.
        source_id: SourceId,
        /// Underlying SQLite error.
        #[source]
        source: rusqlite::Error,
    },
}
