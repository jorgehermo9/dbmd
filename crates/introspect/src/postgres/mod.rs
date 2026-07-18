#![doc = include_str!("README.md")]

use std::{collections::HashMap, fmt};

use dbmd_core::{
    Backend, Column, ColumnBackend, Constraint, ConstraintBackend, ConstraintKind, EnumType,
    ForeignKeyAction, ForeignKeyDeferrability, ForeignKeyInitialTiming, ForeignKeyReference,
    Function, FunctionBackend, Index, IndexBackend, IndexNullsOrder, IndexSortOrder, IndexTarget,
    IndexTerm, Namespace, PostgresColumn, PostgresConstraint, PostgresFunction,
    PostgresFunctionParallel, PostgresFunctionVolatility, PostgresIndex, PostgresPolicy,
    PostgresPolicyCommand, PostgresTable, PostgresTableKind, SourceId, SourceSnapshot, Table,
    TableBackend, View,
};
use postgres::{Client, NoTls, Row};
use thiserror::Error;

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
pub fn introspect(source: &PostgresSource) -> Result<SourceSnapshot, IntrospectionError> {
    let mut client = Client::connect(&source.connection_url, NoTls).map_err(|error| {
        IntrospectionError::Connect {
            source_id: source.id.clone(),
            source: error,
        }
    })?;
    let mut snapshot = SourceSnapshot::new(source.id.clone(), Backend::Postgres);
    snapshot.display_name.clone_from(&source.display_name);
    snapshot.namespaces = load_namespaces(&mut client, &source.id)?;
    snapshot.enums = load_enums(&mut client, &source.id)?;
    snapshot.tables = load_tables(&mut client, &source.id)?;
    snapshot.views = load_views(&mut client, &source.id)?;
    snapshot.functions = load_functions(&mut client, &source.id)?;
    Ok(snapshot)
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
            backend: TableBackend::Postgres(PostgresTable {
                table_kind: table_kind(row.get::<_, String>(3).as_str(), row.get(9)),
                tablespace: row.get(5),
                inherits: row.get(12),
                partition_key: row.get(8),
                partition_parent: row.get(11),
                partition_bound: row.get(10),
                row_level_security: row.get(6),
                force_row_level_security: row.get(7),
                policies: Vec::new(),
            }),
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
        backend: ColumnBackend::Postgres(PostgresColumn {
            enum_values: row.get(8),
            identity,
            generated,
        }),
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
            backend: FunctionBackend::Postgres(PostgresFunction {
                return_type: row.get(5),
                language: row.get(6),
                volatility: function_volatility(&row.get::<_, String>(7)),
                parallel: function_parallel(&row.get::<_, String>(8)),
                security_definer: row.get(9),
            }),
        })
    })
    .collect()
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
        let kind = constraint_kind(&constraint_code);
        let references = (kind == ConstraintKind::ForeignKey).then(|| ForeignKeyReference {
            namespace: row.get::<_, Option<String>>(5).unwrap_or_default(),
            table: row.get::<_, Option<String>>(6).unwrap_or_default(),
            columns: row.get(7),
            on_update: foreign_key_action(&row.get::<_, String>(8)),
            on_delete: foreign_key_action(&row.get::<_, String>(9)),
            match_name: Some(foreign_key_match(&row.get::<_, String>(10)).to_string()),
            deferrability: ForeignKeyDeferrability {
                deferrable: row.get(11),
                initially: if row.get(12) {
                    ForeignKeyInitialTiming::Deferred
                } else {
                    ForeignKeyInitialTiming::Immediate
                },
            },
        });
        tables[table_index].constraints.push(Constraint {
            name: Some(row.get(1)),
            kind,
            columns: row.get(3),
            expression: row.get(4),
            references,
            backend: ConstraintBackend::Postgres(PostgresConstraint {
                definition: row.get(13),
                deferrable: row.get(11),
                initially_deferred: row.get(12),
                validated: row.get(14),
                locally_defined: row.get(15),
                no_inherit: row.get(16),
            }),
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
            backend: IndexBackend::Postgres(PostgresIndex {
                method,
                predicate,
                included_columns: row.get(10),
                nulls_not_distinct: row.get(13),
                valid: row.get(14),
                ready: row.get(15),
                clustered: row.get(16),
                replica_identity: row.get(17),
            }),
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
        let TableBackend::Postgres(table) = &mut tables[table_index].backend else {
            continue;
        };
        table.policies.push(PostgresPolicy {
            name: row.get(1),
            permissive: row.get(2),
            command: policy_command(&row.get::<_, String>(3)),
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

fn table_kind(code: &str, is_partition: bool) -> PostgresTableKind {
    if is_partition {
        return PostgresTableKind::Partition;
    }
    match code {
        "p" => PostgresTableKind::PartitionedTable,
        "f" => PostgresTableKind::ForeignTable,
        "r" => PostgresTableKind::Table,
        _ => unreachable!("relation query filters to represented PostgreSQL table kinds"),
    }
}

fn policy_command(code: &str) -> PostgresPolicyCommand {
    match code {
        "r" => PostgresPolicyCommand::Select,
        "a" => PostgresPolicyCommand::Insert,
        "w" => PostgresPolicyCommand::Update,
        "d" => PostgresPolicyCommand::Delete,
        "*" => PostgresPolicyCommand::All,
        _ => unreachable!("pg_policy.polcmd is constrained to documented PostgreSQL codes"),
    }
}

fn constraint_kind(code: &str) -> ConstraintKind {
    match code {
        "p" => ConstraintKind::PrimaryKey,
        "f" => ConstraintKind::ForeignKey,
        "u" => ConstraintKind::Unique,
        "x" => ConstraintKind::Exclusion,
        "c" => ConstraintKind::Check,
        _ => unreachable!("constraint query filters to represented PostgreSQL constraint kinds"),
    }
}

fn foreign_key_action(code: &str) -> ForeignKeyAction {
    match code {
        "r" => ForeignKeyAction::Restrict,
        "c" => ForeignKeyAction::Cascade,
        "n" => ForeignKeyAction::SetNull,
        "d" => ForeignKeyAction::SetDefault,
        "a" => ForeignKeyAction::NoAction,
        _ => unreachable!("PostgreSQL foreign-key actions use documented catalog codes"),
    }
}

fn foreign_key_match(code: &str) -> &'static str {
    match code {
        "f" => "FULL",
        "p" => "PARTIAL",
        "s" => "SIMPLE",
        _ => unreachable!("PostgreSQL foreign-key match types use documented catalog codes"),
    }
}

fn function_volatility(code: &str) -> PostgresFunctionVolatility {
    match code {
        "i" => PostgresFunctionVolatility::Immutable,
        "s" => PostgresFunctionVolatility::Stable,
        "v" => PostgresFunctionVolatility::Volatile,
        _ => unreachable!("PostgreSQL function volatility uses documented catalog codes"),
    }
}

fn function_parallel(code: &str) -> PostgresFunctionParallel {
    match code {
        "s" => PostgresFunctionParallel::Safe,
        "r" => PostgresFunctionParallel::Restricted,
        "u" => PostgresFunctionParallel::Unsafe,
        _ => unreachable!("PostgreSQL function parallel safety uses documented catalog codes"),
    }
}

/// Why a PostgreSQL source could not be introspected.
#[derive(Debug, Error)]
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
}
