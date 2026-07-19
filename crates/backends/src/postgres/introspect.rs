#![doc = include_str!("README.md")]

use std::{collections::HashMap, fmt};

use dbmd_core::{SourceId, SourceSnapshot};
use postgres::{Client, NoTls, Row};
use thiserror::Error;

use super::catalog::{
    Catalog, Column, Constraint, ConstraintTrigger, EnumType, Function, FunctionParallel,
    FunctionVolatility, Index, Policy, PolicyCommand, Snapshot, Table, TableKind, Trigger,
    TriggerEnabled, TriggerEvent, TriggerOrientation, TriggerTiming, View,
};
use crate::relational::{
    ConstraintKind, ForeignKeyAction, ForeignKeyDeferrability, ForeignKeyInitialTiming,
    ForeignKeyReference, IndexNullsOrder, IndexSortOrder, IndexTarget, IndexTerm, Namespace,
};

/// Connection-backed PostgreSQL source selected for introspection.
#[derive(Clone, PartialEq, Eq)]
pub struct PostgresSource {
    id: SourceId,
    display_name: Option<String>,
    connection_url: String,
}

impl PostgresSource {
    /// Creates a PostgreSQL source from stable identity and a resolved connection URL.
    #[must_use]
    pub fn new(id: SourceId, connection_url: impl Into<String>) -> Self {
        Self {
            id,
            display_name: None,
            connection_url: connection_url.into(),
        }
    }

    /// Adds a presentation-only source name.
    #[must_use]
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Returns the stable configured source identity.
    #[must_use]
    pub fn id(&self) -> &SourceId {
        &self.id
    }
}

impl fmt::Debug for PostgresSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PostgresSource")
            .field("id", &self.id)
            .field("display_name", &self.display_name)
            .field("connection_url", &"[REDACTED]")
            .finish()
    }
}

/// Reads PostgreSQL catalogs and returns one deterministically ordered snapshot.
///
/// # Errors
///
/// Returns [`IntrospectionError`] when the source cannot connect or a required
/// catalog query fails.
pub fn introspect(source: &PostgresSource) -> Result<Snapshot, IntrospectionError> {
    let mut client = Client::connect(&source.connection_url, NoTls).map_err(|error| {
        IntrospectionError::Connect {
            source_id: source.id.clone(),
            source: error,
        }
    })?;
    let snapshot = SourceSnapshot::new(
        source.id.clone(),
        Catalog {
            namespaces: load_namespaces(&mut client, &source.id)?,
            enums: load_enums(&mut client, &source.id)?,
            tables: load_tables(&mut client, &source.id)?,
            views: load_views(&mut client, &source.id)?,
            triggers: load_triggers(&mut client, &source.id)?,
            functions: load_functions(&mut client, &source.id)?,
        },
    );
    Ok(match &source.display_name {
        Some(name) => snapshot.with_display_name(name),
        None => snapshot,
    })
}

fn load_namespaces(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Namespace>, IntrospectionError> {
    query(
        client,
        source_id,
        "namespaces",
        r#"
SELECT namespace.nspname,
       pg_catalog.obj_description(namespace.oid, 'pg_namespace')
FROM pg_catalog.pg_namespace AS namespace
WHERE namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        Ok(Namespace {
            name: row.get(0),
            comment: row.get(1),
        })
    })
    .collect()
}

fn load_enums(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<EnumType>, IntrospectionError> {
    query(
        client,
        source_id,
        "enum types",
        r#"
SELECT namespace.nspname,
       type_record.typname,
       pg_catalog.obj_description(type_record.oid, 'pg_type'),
       ARRAY(
           SELECT enum_value.enumlabel
           FROM pg_catalog.pg_enum AS enum_value
           WHERE enum_value.enumtypid = type_record.oid
           ORDER BY enum_value.enumsortorder
       )
FROM pg_catalog.pg_type AS type_record
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = type_record.typnamespace
WHERE type_record.typtype = 'e'
  AND namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C", type_record.typname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        Ok(EnumType {
            namespace: row.get(0),
            name: row.get(1),
            comment: row.get(2),
            values: row.get(3),
        })
    })
    .collect()
}

fn load_tables(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Table>, IntrospectionError> {
    let relation_rows = query(
        client,
        source_id,
        "relations",
        r#"
SELECT relation.oid::bigint,
       namespace.nspname,
       relation.relname,
       relation.relkind::text,
       pg_catalog.obj_description(relation.oid, 'pg_class'),
       tablespace.spcname,
       relation.relrowsecurity,
       relation.relforcerowsecurity,
       CASE WHEN relation.relkind = 'p'
            THEN pg_catalog.pg_get_partkeydef(relation.oid)
            ELSE NULL
       END,
       relation.relispartition,
       CASE WHEN relation.relispartition
            THEN pg_catalog.pg_get_expr(relation.relpartbound, relation.oid, true)
            ELSE NULL
       END,
       CASE WHEN relation.relispartition THEN (
           SELECT parent_namespace.nspname || '.' || parent_relation.relname
           FROM pg_catalog.pg_inherits AS inheritance
           JOIN pg_catalog.pg_class AS parent_relation
             ON parent_relation.oid = inheritance.inhparent
           JOIN pg_catalog.pg_namespace AS parent_namespace
             ON parent_namespace.oid = parent_relation.relnamespace
           WHERE inheritance.inhrelid = relation.oid
           ORDER BY inheritance.inhseqno
           LIMIT 1
       ) ELSE NULL END,
       ARRAY(
           SELECT parent_namespace.nspname || '.' || parent_relation.relname
           FROM pg_catalog.pg_inherits AS inheritance
           JOIN pg_catalog.pg_class AS parent_relation
             ON parent_relation.oid = inheritance.inhparent
           JOIN pg_catalog.pg_namespace AS parent_namespace
             ON parent_namespace.oid = parent_relation.relnamespace
           WHERE inheritance.inhrelid = relation.oid
           ORDER BY inheritance.inhseqno
       )
FROM pg_catalog.pg_class AS relation
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = relation.relnamespace
LEFT JOIN pg_catalog.pg_tablespace AS tablespace
  ON tablespace.oid = NULLIF(relation.reltablespace, 0)
WHERE relation.relkind IN ('r', 'p', 'f')
  AND namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C", relation.relname COLLATE "C"
"#,
    )?;

    let mut tables = Vec::with_capacity(relation_rows.len());
    let mut table_by_oid = HashMap::with_capacity(relation_rows.len());
    for row in relation_rows {
        let oid = row.get::<_, i64>(0);
        table_by_oid.insert(oid, tables.len());
        tables.push(Table {
            namespace: row.get(1),
            name: row.get(2),
            comment: row.get(4),
            columns: Vec::new(),
            constraints: Vec::new(),
            indexes: Vec::new(),
            kind: table_kind(source_id, row.get::<_, String>(3).as_str(), row.get(9))?,
            tablespace: row.get(5),
            inherits: row.get(12),
            partition_key: row.get(8),
            partition_parent: row.get(11),
            partition_bound: row.get(10),
            row_level_security: row.get(6),
            force_row_level_security: row.get(7),
            policies: Vec::new(),
        });
    }

    load_columns(client, source_id, &table_by_oid, &mut tables)?;
    load_constraints(client, source_id, &table_by_oid, &mut tables)?;
    load_indexes(client, source_id, &table_by_oid, &mut tables)?;
    load_policies(client, source_id, &table_by_oid, &mut tables)?;
    Ok(tables)
}

fn load_columns(
    client: &mut Client,
    source_id: &SourceId,
    table_by_oid: &HashMap<i64, usize>,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let rows = query(
        client,
        source_id,
        "columns",
        r#"
SELECT attribute.attrelid::bigint,
       attribute.attname,
       pg_catalog.format_type(attribute.atttypid, attribute.atttypmod),
       NOT attribute.attnotnull,
       pg_catalog.pg_get_expr(default_value.adbin, default_value.adrelid, true),
       pg_catalog.col_description(attribute.attrelid, attribute.attnum),
       attribute.attidentity::text,
       attribute.attgenerated::text,
       COALESCE(
           ARRAY(
               SELECT enum_value.enumlabel
               FROM pg_catalog.pg_enum AS enum_value
               WHERE enum_value.enumtypid = attribute.atttypid
               ORDER BY enum_value.enumsortorder
           ),
           ARRAY[]::text[]
       )
FROM pg_catalog.pg_attribute AS attribute
JOIN pg_catalog.pg_class AS relation
  ON relation.oid = attribute.attrelid
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = relation.relnamespace
LEFT JOIN pg_catalog.pg_attrdef AS default_value
  ON default_value.adrelid = attribute.attrelid
 AND default_value.adnum = attribute.attnum
WHERE relation.relkind IN ('r', 'p', 'f')
  AND namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
  AND attribute.attnum > 0
  AND NOT attribute.attisdropped
ORDER BY namespace.nspname COLLATE "C",
         relation.relname COLLATE "C",
         attribute.attnum
"#,
    )?;

    for row in rows {
        let Some(&table_index) = table_by_oid.get(&row.get::<_, i64>(0)) else {
            continue;
        };
        let identity_code = row.get::<_, String>(6);
        let generated_code = row.get::<_, String>(7);
        let expression = row.get::<_, Option<String>>(4);
        let identity = match identity_code.as_str() {
            "a" => Some("always".to_string()),
            "d" => Some("by_default".to_string()),
            _ => None,
        };
        let generated = (!generated_code.is_empty())
            .then(|| expression.clone())
            .flatten();
        tables[table_index].columns.push(postgres_column(
            &row,
            identity,
            generated,
            generated_code.is_empty().then_some(expression).flatten(),
        ));
    }
    Ok(())
}

fn postgres_column(
    row: &Row,
    identity: Option<String>,
    generated: Option<String>,
    default: Option<String>,
) -> Column {
    Column {
        name: row.get(1),
        data_type: row.get(2),
        nullable: Some(row.get(3)),
        default,
        comment: row.get(5),
        enum_values: row.get(8),
        identity,
        generated,
    }
}

fn load_views(client: &mut Client, source_id: &SourceId) -> Result<Vec<View>, IntrospectionError> {
    let rows = query(
        client,
        source_id,
        "views",
        r#"
SELECT relation.oid::bigint,
       namespace.nspname,
       relation.relname,
       relation.relkind = 'm',
       pg_catalog.obj_description(relation.oid, 'pg_class'),
       pg_catalog.pg_get_viewdef(relation.oid, true)
FROM pg_catalog.pg_class AS relation
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = relation.relnamespace
WHERE relation.relkind IN ('v', 'm')
  AND namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C", relation.relname COLLATE "C"
"#,
    )?;
    let mut views = Vec::with_capacity(rows.len());
    let mut view_by_oid = HashMap::with_capacity(rows.len());
    for row in rows {
        view_by_oid.insert(row.get::<_, i64>(0), views.len());
        views.push(View {
            namespace: row.get(1),
            name: row.get(2),
            materialized: row.get(3),
            comment: row.get(4),
            definition: row.get(5),
            columns: Vec::new(),
        });
    }

    let column_rows = query(
        client,
        source_id,
        "view columns",
        r#"
SELECT attribute.attrelid::bigint,
       attribute.attname,
       pg_catalog.format_type(attribute.atttypid, attribute.atttypmod),
       NOT attribute.attnotnull,
       NULL::text,
       pg_catalog.col_description(attribute.attrelid, attribute.attnum),
       ''::text,
       ''::text,
       COALESCE(
           ARRAY(
               SELECT enum_value.enumlabel
               FROM pg_catalog.pg_enum AS enum_value
               WHERE enum_value.enumtypid = attribute.atttypid
               ORDER BY enum_value.enumsortorder
           ),
           ARRAY[]::text[]
       )
FROM pg_catalog.pg_attribute AS attribute
JOIN pg_catalog.pg_class AS relation
  ON relation.oid = attribute.attrelid
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = relation.relnamespace
WHERE relation.relkind IN ('v', 'm')
  AND namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
  AND attribute.attnum > 0
  AND NOT attribute.attisdropped
ORDER BY namespace.nspname COLLATE "C",
         relation.relname COLLATE "C",
         attribute.attnum
"#,
    )?;
    for row in column_rows {
        let Some(&view_index) = view_by_oid.get(&row.get::<_, i64>(0)) else {
            continue;
        };
        views[view_index]
            .columns
            .push(postgres_column(&row, None, None, None));
    }
    Ok(views)
}

fn load_triggers(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Trigger>, IntrospectionError> {
    query(
        client,
        source_id,
        "triggers",
        r#"
SELECT namespace.nspname,
       trigger_record.tgname,
       namespace.nspname,
       relation.relname,
       (trigger_record.tgtype::integer & 2) <> 0,
       (trigger_record.tgtype::integer & 64) <> 0,
       (trigger_record.tgtype::integer & 1) <> 0,
       (trigger_record.tgtype::integer & 4) <> 0,
       (trigger_record.tgtype::integer & 16) <> 0,
       (trigger_record.tgtype::integer & 8) <> 0,
       (trigger_record.tgtype::integer & 32) <> 0,
       ARRAY(
           SELECT attribute.attname
           FROM unnest(trigger_record.tgattr::smallint[]) WITH ORDINALITY
                AS trigger_column(attnum, position)
           JOIN pg_catalog.pg_attribute AS attribute
             ON attribute.attrelid = trigger_record.tgrelid
            AND attribute.attnum = trigger_column.attnum
           ORDER BY trigger_column.position
       ),
       pg_catalog.obj_description(trigger_record.oid, 'pg_trigger'),
       trigger_record.tgqual IS NOT NULL,
       pg_catalog.pg_get_triggerdef(trigger_record.oid, true),
       pg_catalog.format(
           '%I.%I(%s)',
           function_namespace.nspname,
           function_record.proname,
           pg_catalog.pg_get_function_identity_arguments(function_record.oid)
       ),
       CASE WHEN trigger_record.tgnargs = 0 THEN ARRAY[]::text[] ELSE ARRAY(
           SELECT pg_catalog.convert_from(
                      pg_catalog.decode(argument.argument_hex, 'hex'),
                      pg_catalog.current_setting('server_encoding')
                  )
           FROM pg_catalog.regexp_split_to_table(
                    pg_catalog.encode(trigger_record.tgargs, 'hex'),
                    '00'
                ) WITH ORDINALITY AS argument(argument_hex, position)
           WHERE argument.position <= trigger_record.tgnargs
           ORDER BY argument.position
       ) END,
       trigger_record.tgenabled::text,
       trigger_record.tgconstraint <> 0,
       CASE WHEN referenced_relation.oid IS NULL THEN NULL ELSE
           pg_catalog.format('%I.%I', referenced_namespace.nspname, referenced_relation.relname)
       END,
       trigger_record.tgdeferrable,
       trigger_record.tginitdeferred,
       trigger_record.tgoldtable,
       trigger_record.tgnewtable,
       CASE WHEN parent_trigger.oid IS NULL THEN NULL ELSE
           pg_catalog.format(
               '%I.%I.%I',
               parent_namespace.nspname,
               parent_relation.relname,
               parent_trigger.tgname
           )
       END
FROM pg_catalog.pg_trigger AS trigger_record
JOIN pg_catalog.pg_class AS relation
  ON relation.oid = trigger_record.tgrelid
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = relation.relnamespace
JOIN pg_catalog.pg_proc AS function_record
  ON function_record.oid = trigger_record.tgfoid
JOIN pg_catalog.pg_namespace AS function_namespace
  ON function_namespace.oid = function_record.pronamespace
LEFT JOIN pg_catalog.pg_class AS referenced_relation
  ON referenced_relation.oid = NULLIF(trigger_record.tgconstrrelid, 0)
LEFT JOIN pg_catalog.pg_namespace AS referenced_namespace
  ON referenced_namespace.oid = referenced_relation.relnamespace
LEFT JOIN pg_catalog.pg_trigger AS parent_trigger
  ON parent_trigger.oid = NULLIF(trigger_record.tgparentid, 0)
LEFT JOIN pg_catalog.pg_class AS parent_relation
  ON parent_relation.oid = parent_trigger.tgrelid
LEFT JOIN pg_catalog.pg_namespace AS parent_namespace
  ON parent_namespace.oid = parent_relation.relnamespace
WHERE (NOT trigger_record.tgisinternal OR trigger_record.tgparentid <> 0)
  AND namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C",
         relation.relname COLLATE "C",
         trigger_record.tgname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        let mut events = Vec::with_capacity(4);
        if row.get(7) {
            events.push(TriggerEvent::Insert);
        }
        if row.get(8) {
            events.push(TriggerEvent::Update {
                columns: row.get(11),
            });
        }
        if row.get(9) {
            events.push(TriggerEvent::Delete);
        }
        if row.get(10) {
            events.push(TriggerEvent::Truncate);
        }
        let constraint = row.get::<_, bool>(18).then(|| ConstraintTrigger {
            referenced_table: row.get(19),
            deferrable: row.get(20),
            initially_deferred: row.get(21),
        });
        let namespace = row.get::<_, String>(0);
        let name = row.get::<_, String>(1);
        let definition = row.get::<_, String>(14);
        let when_expression =
            trigger_when_expression(&definition, row.get(13)).ok_or_else(|| {
                IntrospectionError::TriggerDefinition {
                    source_id: source_id.clone(),
                    trigger: format!("{namespace}.{name}"),
                }
            })?;
        Ok(Trigger {
            namespace,
            name,
            target_namespace: row.get(2),
            target: row.get(3),
            timing: if row.get(4) {
                TriggerTiming::Before
            } else if row.get(5) {
                TriggerTiming::InsteadOf
            } else {
                TriggerTiming::After
            },
            events,
            orientation: if row.get(6) {
                TriggerOrientation::Row
            } else {
                TriggerOrientation::Statement
            },
            comment: row.get(12),
            when_expression,
            definition,
            function: row.get(15),
            arguments: row.get(16),
            enabled: trigger_enabled(source_id, &row.get::<_, String>(17))?,
            constraint,
            old_transition_table: row.get(22),
            new_transition_table: row.get(23),
            parent_trigger: row.get(24),
        })
    })
    .collect()
}

fn load_functions(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Function>, IntrospectionError> {
    query(
        client,
        source_id,
        "functions",
        r#"
SELECT namespace.nspname,
       procedure.proname,
       '(' || pg_catalog.pg_get_function_identity_arguments(procedure.oid) || ')',
       pg_catalog.pg_get_functiondef(procedure.oid),
       pg_catalog.obj_description(procedure.oid, 'pg_proc'),
       pg_catalog.pg_get_function_result(procedure.oid),
       language.lanname,
       procedure.provolatile::text,
       procedure.proparallel::text,
       procedure.prosecdef
FROM pg_catalog.pg_proc AS procedure
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = procedure.pronamespace
JOIN pg_catalog.pg_language AS language
  ON language.oid = procedure.prolang
WHERE procedure.prokind = 'f'
  AND namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C",
         procedure.proname COLLATE "C",
         pg_catalog.pg_get_function_identity_arguments(procedure.oid) COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        Ok(Function {
            namespace: row.get(0),
            name: row.get(1),
            signature: row.get(2),
            definition: row.get(3),
            comment: row.get(4),
            return_type: row.get(5),
            language: row.get(6),
            volatility: function_volatility(source_id, &row.get::<_, String>(7))?,
            parallel: function_parallel(source_id, &row.get::<_, String>(8))?,
            security_definer: row.get(9),
        })
    })
    .collect()
}

fn trigger_when_expression(definition: &str, has_when_expression: bool) -> Option<Option<String>> {
    if !has_when_expression {
        return Some(None);
    }
    let closing = definition.rfind(") EXECUTE FUNCTION ")?;
    let opening = matching_opening_parenthesis(definition, closing)?;
    if !definition[..opening].ends_with(" WHEN ") {
        return None;
    }
    Some(Some(definition[opening + 1..closing].to_string()))
}

fn matching_opening_parenthesis(value: &str, closing: usize) -> Option<usize> {
    #[derive(Debug)]
    enum State {
        Normal,
        SingleQuoted,
        DoubleQuoted,
        DollarQuoted(Vec<u8>),
        LineComment,
        BlockComment,
    }

    let bytes = value.as_bytes();
    let mut state = State::Normal;
    let mut stack = Vec::new();
    let mut index = 0;
    while index <= closing && index < bytes.len() {
        match &state {
            State::Normal => match bytes[index] {
                b'\'' => {
                    state = State::SingleQuoted;
                    index += 1;
                }
                b'"' => {
                    state = State::DoubleQuoted;
                    index += 1;
                }
                b'$' => {
                    if let Some(delimiter) = dollar_quote_delimiter(bytes, index) {
                        index += delimiter.len();
                        state = State::DollarQuoted(delimiter.to_vec());
                    } else {
                        index += 1;
                    }
                }
                b'-' if bytes.get(index + 1) == Some(&b'-') => {
                    state = State::LineComment;
                    index += 2;
                }
                b'/' if bytes.get(index + 1) == Some(&b'*') => {
                    state = State::BlockComment;
                    index += 2;
                }
                b'(' => {
                    stack.push(index);
                    index += 1;
                }
                b')' => {
                    let opening = stack.pop()?;
                    if index == closing {
                        return Some(opening);
                    }
                    index += 1;
                }
                _ => index += 1,
            },
            State::SingleQuoted => {
                if bytes[index] == b'\'' {
                    if bytes.get(index + 1) == Some(&b'\'') {
                        index += 2;
                    } else {
                        state = State::Normal;
                        index += 1;
                    }
                } else {
                    index += 1;
                }
            }
            State::DoubleQuoted => {
                if bytes[index] == b'"' {
                    if bytes.get(index + 1) == Some(&b'"') {
                        index += 2;
                    } else {
                        state = State::Normal;
                        index += 1;
                    }
                } else {
                    index += 1;
                }
            }
            State::DollarQuoted(delimiter) => {
                if bytes[index..].starts_with(delimiter) {
                    index += delimiter.len();
                    state = State::Normal;
                } else {
                    index += 1;
                }
            }
            State::LineComment => {
                if matches!(bytes[index], b'\n' | b'\r') {
                    state = State::Normal;
                }
                index += 1;
            }
            State::BlockComment => {
                if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
                    state = State::Normal;
                    index += 2;
                } else {
                    index += 1;
                }
            }
        }
    }
    None
}

fn dollar_quote_delimiter(bytes: &[u8], start: usize) -> Option<&[u8]> {
    let tail = bytes.get(start + 1..)?;
    let end = tail
        .iter()
        .position(|byte| *byte == b'$')?
        .checked_add(start + 1)?;
    bytes[start + 1..end]
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        .then(|| &bytes[start..=end])
}

fn load_constraints(
    client: &mut Client,
    source_id: &SourceId,
    table_by_oid: &HashMap<i64, usize>,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let rows = query(
        client,
        source_id,
        "constraints",
        r#"
SELECT constraint_record.conrelid::bigint,
       constraint_record.conname,
       constraint_record.contype::text,
       ARRAY(
           SELECT attribute.attname
           FROM unnest(constraint_record.conkey) WITH ORDINALITY AS key(attnum, position)
           JOIN pg_catalog.pg_attribute AS attribute
             ON attribute.attrelid = constraint_record.conrelid
            AND attribute.attnum = key.attnum
           ORDER BY key.position
       ),
       pg_catalog.pg_get_expr(
           constraint_record.conbin,
           constraint_record.conrelid,
           true
       ),
       target_namespace.nspname,
       target_relation.relname,
       ARRAY(
           SELECT attribute.attname
           FROM unnest(constraint_record.confkey) WITH ORDINALITY AS key(attnum, position)
           JOIN pg_catalog.pg_attribute AS attribute
             ON attribute.attrelid = constraint_record.confrelid
            AND attribute.attnum = key.attnum
           ORDER BY key.position
       ),
       constraint_record.confupdtype::text,
       constraint_record.confdeltype::text,
       constraint_record.confmatchtype::text,
       constraint_record.condeferrable,
       constraint_record.condeferred,
       pg_catalog.pg_get_constraintdef(constraint_record.oid, true),
       constraint_record.convalidated,
       constraint_record.conislocal,
       constraint_record.connoinherit
FROM pg_catalog.pg_constraint AS constraint_record
JOIN pg_catalog.pg_class AS relation
  ON relation.oid = constraint_record.conrelid
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = relation.relnamespace
LEFT JOIN pg_catalog.pg_class AS target_relation
  ON target_relation.oid = constraint_record.confrelid
LEFT JOIN pg_catalog.pg_namespace AS target_namespace
  ON target_namespace.oid = target_relation.relnamespace
WHERE namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
  AND constraint_record.contype IN ('c', 'f', 'p', 'u', 'x')
ORDER BY namespace.nspname COLLATE "C",
         relation.relname COLLATE "C",
         constraint_record.conname COLLATE "C"
"#,
    )?;

    for row in rows {
        let Some(&table_index) = table_by_oid.get(&row.get::<_, i64>(0)) else {
            continue;
        };
        let constraint_code = row.get::<_, String>(2);
        let kind = constraint_kind(source_id, &constraint_code)?;
        let references = if kind == ConstraintKind::ForeignKey {
            Some(ForeignKeyReference {
                namespace: row.get::<_, Option<String>>(5).unwrap_or_default(),
                table: row.get::<_, Option<String>>(6).unwrap_or_default(),
                columns: row.get(7),
                on_update: foreign_key_action(source_id, &row.get::<_, String>(8))?,
                on_delete: foreign_key_action(source_id, &row.get::<_, String>(9))?,
                match_name: Some(
                    foreign_key_match(source_id, &row.get::<_, String>(10))?.to_string(),
                ),
                deferrability: ForeignKeyDeferrability {
                    deferrable: row.get(11),
                    initially: if row.get(12) {
                        ForeignKeyInitialTiming::Deferred
                    } else {
                        ForeignKeyInitialTiming::Immediate
                    },
                },
            })
        } else {
            None
        };
        tables[table_index].constraints.push(Constraint {
            name: Some(row.get(1)),
            kind,
            columns: row.get(3),
            expression: row.get(4),
            references,
            definition: row.get(13),
            deferrable: row.get(11),
            initially_deferred: row.get(12),
            validated: row.get(14),
            locally_defined: row.get(15),
            no_inherit: row.get(16),
        });
    }
    Ok(())
}

fn load_indexes(
    client: &mut Client,
    source_id: &SourceId,
    table_by_oid: &HashMap<i64, usize>,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let rows = query(
        client,
        source_id,
        "indexes",
        r#"
SELECT index_record.indrelid::bigint,
       index_relation.relname,
       index_record.indisunique,
       access_method.amname,
       pg_catalog.pg_get_expr(index_record.indpred, index_record.indrelid, true),
       pg_catalog.pg_get_indexdef(index_record.indexrelid),
       ARRAY(
           SELECT pg_catalog.pg_get_indexdef(index_record.indexrelid, position, true)
           FROM generate_series(1, index_record.indnkeyatts) AS position
           ORDER BY position
       ),
       ARRAY(
           SELECT (index_record.indkey)[position - 1] <> 0
           FROM generate_series(1, index_record.indnkeyatts) AS position
           ORDER BY position
       ),
       ARRAY(
           SELECT ((index_record.indoption)[position - 1] & 1) = 1
           FROM generate_series(1, index_record.indnkeyatts) AS position
           ORDER BY position
       ),
       ARRAY(
           SELECT CASE WHEN collation_record.oid IS NULL THEN NULL
                       ELSE pg_catalog.format(
                           '%I.%I',
                           collation_namespace.nspname,
                           collation_record.collname
                       )
                  END
           FROM generate_series(1, index_record.indnkeyatts) AS position
           LEFT JOIN pg_catalog.pg_collation AS collation_record
             ON collation_record.oid = (index_record.indcollation)[position - 1]
           LEFT JOIN pg_catalog.pg_namespace AS collation_namespace
             ON collation_namespace.oid = collation_record.collnamespace
           ORDER BY position
       ),
       ARRAY(
           SELECT pg_catalog.pg_get_indexdef(index_record.indexrelid, position, true)
           FROM generate_series(index_record.indnkeyatts + 1, index_record.indnatts) AS position
           ORDER BY position
       ),
       ARRAY(
           SELECT CASE WHEN operator_class.oid IS NULL THEN NULL
                       ELSE pg_catalog.format(
                           '%I.%I',
                           operator_namespace.nspname,
                           operator_class.opcname
                       )
                  END
           FROM generate_series(1, index_record.indnkeyatts) AS position
           LEFT JOIN pg_catalog.pg_opclass AS operator_class
             ON operator_class.oid = (index_record.indclass)[position - 1]
           LEFT JOIN pg_catalog.pg_namespace AS operator_namespace
             ON operator_namespace.oid = operator_class.opcnamespace
           ORDER BY position
       ),
       ARRAY(
           SELECT ((index_record.indoption)[position - 1] & 2) = 2
           FROM generate_series(1, index_record.indnkeyatts) AS position
           ORDER BY position
       ),
       index_record.indnullsnotdistinct,
       index_record.indisvalid,
       index_record.indisready,
       index_record.indisclustered,
       index_record.indisreplident
FROM pg_catalog.pg_index AS index_record
JOIN pg_catalog.pg_class AS table_relation
  ON table_relation.oid = index_record.indrelid
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = table_relation.relnamespace
JOIN pg_catalog.pg_class AS index_relation
  ON index_relation.oid = index_record.indexrelid
JOIN pg_catalog.pg_am AS access_method
  ON access_method.oid = index_relation.relam
WHERE table_relation.relkind IN ('r', 'p', 'f')
  AND namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C",
         table_relation.relname COLLATE "C",
         index_relation.relname COLLATE "C"
"#,
    )?;

    for row in rows {
        let Some(&table_index) = table_by_oid.get(&row.get::<_, i64>(0)) else {
            continue;
        };
        let targets = row.get::<_, Vec<String>>(6);
        let column_flags = row.get::<_, Vec<bool>>(7);
        let descending = row.get::<_, Vec<bool>>(8);
        let collations = row.get::<_, Vec<Option<String>>>(9);
        let operator_classes = row.get::<_, Vec<Option<String>>>(11);
        let nulls_first = row.get::<_, Vec<bool>>(12);
        let method = row.get::<_, String>(3);
        let terms = targets
            .into_iter()
            .zip(column_flags)
            .zip(descending)
            .zip(collations)
            .zip(operator_classes)
            .zip(nulls_first)
            .map(
                |(
                    ((((target, is_column), descending), collation), operator_class),
                    nulls_first,
                )| {
                    IndexTerm {
                        target: if is_column {
                            IndexTarget::Column(target)
                        } else {
                            IndexTarget::Expression(target)
                        },
                        collation,
                        operator_class,
                        order: if descending {
                            IndexSortOrder::Descending
                        } else {
                            IndexSortOrder::Ascending
                        },
                        nulls_order: (method == "btree").then_some(if nulls_first {
                            IndexNullsOrder::First
                        } else {
                            IndexNullsOrder::Last
                        }),
                    }
                },
            )
            .collect();
        let predicate: Option<String> = row.get(4);
        tables[table_index].indexes.push(Index {
            name: row.get(1),
            unique: row.get(2),
            terms,
            predicate: predicate.clone(),
            definition: row.get(5),
            method,
            included_columns: row.get(10),
            nulls_not_distinct: row.get(13),
            valid: row.get(14),
            ready: row.get(15),
            clustered: row.get(16),
            replica_identity: row.get(17),
        });
    }
    Ok(())
}

fn load_policies(
    client: &mut Client,
    source_id: &SourceId,
    table_by_oid: &HashMap<i64, usize>,
    tables: &mut [Table],
) -> Result<(), IntrospectionError> {
    let rows = query(
        client,
        source_id,
        "row-level security policies",
        r#"
SELECT policy.polrelid::bigint,
       policy.polname,
       policy.polpermissive,
       policy.polcmd::text,
       ARRAY(
           SELECT CASE role_oid
                    WHEN 0 THEN 'PUBLIC'
                    ELSE pg_catalog.pg_get_userbyid(role_oid)
                  END
           FROM unnest(policy.polroles) WITH ORDINALITY AS role(role_oid, position)
           ORDER BY role.position
       ),
       pg_catalog.pg_get_expr(policy.polqual, policy.polrelid, true),
       pg_catalog.pg_get_expr(policy.polwithcheck, policy.polrelid, true)
FROM pg_catalog.pg_policy AS policy
JOIN pg_catalog.pg_class AS relation
  ON relation.oid = policy.polrelid
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = relation.relnamespace
WHERE namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C",
         relation.relname COLLATE "C",
         policy.polname COLLATE "C"
"#,
    )?;

    for row in rows {
        let Some(&table_index) = table_by_oid.get(&row.get::<_, i64>(0)) else {
            continue;
        };
        tables[table_index].policies.push(Policy {
            name: row.get(1),
            permissive: row.get(2),
            command: policy_command(source_id, &row.get::<_, String>(3))?,
            roles: row.get(4),
            using_expression: row.get(5),
            check_expression: row.get(6),
        });
    }
    Ok(())
}

fn query(
    client: &mut Client,
    source_id: &SourceId,
    catalog: &'static str,
    sql: &str,
) -> Result<Vec<Row>, IntrospectionError> {
    client
        .query(sql, &[])
        .map_err(|source| IntrospectionError::Catalog {
            source_id: source_id.clone(),
            catalog,
            source,
        })
}

fn table_kind(
    source_id: &SourceId,
    code: &str,
    is_partition: bool,
) -> Result<TableKind, IntrospectionError> {
    if is_partition {
        return Ok(TableKind::Partition);
    }
    match code {
        "p" => Ok(TableKind::PartitionedTable),
        "f" => Ok(TableKind::ForeignTable),
        "r" => Ok(TableKind::Table),
        _ => Err(unsupported_catalog_value(source_id, "relation kind", code)),
    }
}

fn policy_command(source_id: &SourceId, code: &str) -> Result<PolicyCommand, IntrospectionError> {
    match code {
        "r" => Ok(PolicyCommand::Select),
        "a" => Ok(PolicyCommand::Insert),
        "w" => Ok(PolicyCommand::Update),
        "d" => Ok(PolicyCommand::Delete),
        "*" => Ok(PolicyCommand::All),
        _ => Err(unsupported_catalog_value(source_id, "policy command", code)),
    }
}

fn constraint_kind(source_id: &SourceId, code: &str) -> Result<ConstraintKind, IntrospectionError> {
    match code {
        "p" => Ok(ConstraintKind::PrimaryKey),
        "f" => Ok(ConstraintKind::ForeignKey),
        "u" => Ok(ConstraintKind::Unique),
        "x" => Ok(ConstraintKind::Exclusion),
        "c" => Ok(ConstraintKind::Check),
        _ => Err(unsupported_catalog_value(
            source_id,
            "constraint kind",
            code,
        )),
    }
}

fn foreign_key_action(
    source_id: &SourceId,
    code: &str,
) -> Result<ForeignKeyAction, IntrospectionError> {
    match code {
        "r" => Ok(ForeignKeyAction::Restrict),
        "c" => Ok(ForeignKeyAction::Cascade),
        "n" => Ok(ForeignKeyAction::SetNull),
        "d" => Ok(ForeignKeyAction::SetDefault),
        "a" => Ok(ForeignKeyAction::NoAction),
        _ => Err(unsupported_catalog_value(
            source_id,
            "foreign-key action",
            code,
        )),
    }
}

fn foreign_key_match(source_id: &SourceId, code: &str) -> Result<&'static str, IntrospectionError> {
    match code {
        "f" => Ok("FULL"),
        "p" => Ok("PARTIAL"),
        "s" => Ok("SIMPLE"),
        _ => Err(unsupported_catalog_value(
            source_id,
            "foreign-key match type",
            code,
        )),
    }
}

fn function_volatility(
    source_id: &SourceId,
    code: &str,
) -> Result<FunctionVolatility, IntrospectionError> {
    match code {
        "i" => Ok(FunctionVolatility::Immutable),
        "s" => Ok(FunctionVolatility::Stable),
        "v" => Ok(FunctionVolatility::Volatile),
        _ => Err(unsupported_catalog_value(
            source_id,
            "function volatility",
            code,
        )),
    }
}

fn function_parallel(
    source_id: &SourceId,
    code: &str,
) -> Result<FunctionParallel, IntrospectionError> {
    match code {
        "s" => Ok(FunctionParallel::Safe),
        "r" => Ok(FunctionParallel::Restricted),
        "u" => Ok(FunctionParallel::Unsafe),
        _ => Err(unsupported_catalog_value(
            source_id,
            "function parallel safety",
            code,
        )),
    }
}

fn trigger_enabled(source_id: &SourceId, code: &str) -> Result<TriggerEnabled, IntrospectionError> {
    match code {
        "O" => Ok(TriggerEnabled::Origin),
        "D" => Ok(TriggerEnabled::Disabled),
        "R" => Ok(TriggerEnabled::Replica),
        "A" => Ok(TriggerEnabled::Always),
        _ => Err(unsupported_catalog_value(
            source_id,
            "trigger enablement",
            code,
        )),
    }
}

fn unsupported_catalog_value(
    source_id: &SourceId,
    catalog: &'static str,
    value: &str,
) -> IntrospectionError {
    IntrospectionError::UnsupportedCatalogValue {
        source_id: source_id.clone(),
        catalog,
        value: value.to_string(),
    }
}

/// Why a PostgreSQL source could not be introspected.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IntrospectionError {
    /// The configured source could not be connected.
    #[error("failed to connect to PostgreSQL source `{source_id}`")]
    Connect {
        /// Stable source identity, safe for diagnostics.
        source_id: SourceId,
        /// Driver error without the configured URL.
        #[source]
        source: postgres::Error,
    },
    /// A required catalog query failed.
    #[error("failed to query PostgreSQL {catalog} for source `{source_id}`")]
    Catalog {
        /// Stable source identity, safe for diagnostics.
        source_id: SourceId,
        /// Catalog surface being read.
        catalog: &'static str,
        /// Driver query error.
        #[source]
        source: postgres::Error,
    },
    /// A trigger predicate was present but could not be recovered from the
    /// server-normalized definition.
    #[error(
        "failed to interpret PostgreSQL definition for trigger `{trigger}` in source `{source_id}`"
    )]
    TriggerDefinition {
        /// Stable source identity, safe for diagnostics.
        source_id: SourceId,
        /// Qualified trigger name, safe for diagnostics.
        trigger: String,
    },
    /// A catalog code was outside the values understood by this adapter.
    #[error(
        "unsupported PostgreSQL {catalog} value `{value}` while introspecting source `{source_id}`"
    )]
    UnsupportedCatalogValue {
        /// Stable source identity, safe for diagnostics.
        source_id: SourceId,
        /// Catalog field being interpreted.
        catalog: &'static str,
        /// Unexpected server-provided code.
        value: String,
    },
}

#[cfg(test)]
mod tests {
    use super::trigger_when_expression;

    #[test]
    fn extracts_trigger_predicate_around_quoted_delimiter_text() {
        let definition = concat!(
            "CREATE TRIGGER \"name WHEN (not a predicate)\" BEFORE UPDATE ON audit.accounts ",
            "FOR EACH ROW WHEN ((new.email = 'literal WHEN (still literal)') ",
            "AND pg_trigger_depth() = 0) EXECUTE FUNCTION audit.capture_row_change()"
        );

        assert_eq!(
            trigger_when_expression(definition, true),
            Some(Some(
                "(new.email = 'literal WHEN (still literal)') AND pg_trigger_depth() = 0"
                    .to_string()
            ))
        );
    }

    #[test]
    fn extracts_trigger_predicate_around_dollar_quoted_parentheses() {
        let definition = concat!(
            "CREATE TRIGGER test BEFORE UPDATE ON audit.accounts FOR EACH ROW ",
            "WHEN (new.email = $tag$) EXECUTE FUNCTION fake WHEN ($tag$) ",
            "EXECUTE FUNCTION audit.capture_row_change()"
        );

        assert_eq!(
            trigger_when_expression(definition, true),
            Some(Some(
                "new.email = $tag$) EXECUTE FUNCTION fake WHEN ($tag$".to_string()
            ))
        );
    }

    #[test]
    fn distinguishes_absent_from_unrecoverable_trigger_predicates() {
        let definition =
            "CREATE TRIGGER test BEFORE UPDATE ON audit.accounts EXECUTE FUNCTION audit.fn()";

        assert_eq!(trigger_when_expression(definition, false), Some(None));
        assert_eq!(trigger_when_expression(definition, true), None);
    }
}
