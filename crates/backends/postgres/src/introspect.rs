//! PostgreSQL catalog introspection and normalization.
use std::{collections::HashMap, fmt};

use dbmd_core::{SourceId, SourceSnapshot};
use postgres::{Client, NoTls, Row};
use thiserror::Error;

use super::catalog::{
    AccessMethod, AccessMethodKind, Aggregate, AggregateFinalModify, AggregateKind, BaseType,
    BaseTypeDetails, Cast, CastContext, CastMethod, Catalog, Collation, CollationProvider, Column,
    ColumnCompression, ColumnStorage, CompositeAttribute, CompositeType, Constraint,
    ConstraintKind, ConstraintTrigger, Conversion, Database, DatabaseLocaleProvider,
    DefaultPrivilege, DefaultPrivilegeObject, Domain, DomainConstraint, EnumType, EventTrigger,
    EventTriggerEvent, ExtendedStatistics, Extension, ExtensionConfiguration, ExtensionMember,
    ForeignDataWrapper, ForeignServer, ForeignTable, Function, FunctionKind, FunctionParallel,
    FunctionVolatility, GeneratedColumn, GeneratedColumnKind, IdentityGeneration, Index,
    IndexNullsOrder, IndexTarget, IndexTerm, Language, LargeObject, MultirangeType, Namespace,
    ObjectPrivilege, Operator, OperatorClass, OperatorFamily, OperatorFamilyFunction,
    OperatorFamilyOperator, OperatorKind, OperatorPurpose, Policy, PolicyCommand, PrivilegeKind,
    PrivilegeObjectKind, Procedure, Publication, PublicationGeneratedColumns, PublicationTable,
    RangeType, RelationPersistence, ReplicaIdentity, RewriteRule, RewriteRuleEvent, Role,
    RoleDatabaseSetting, RoleMembership, SecurityLabel, SecurityLabelObjectKind, Sequence,
    SequencePersistence, Snapshot, StatisticsKind, Subscription, SubscriptionOrigin,
    SubscriptionStreaming, SubscriptionTwoPhase, SynchronousCommit, Table, TableKind, Tablespace,
    TextSearchConfiguration, TextSearchDictionary, TextSearchMapping, TextSearchParser,
    TextSearchTemplate, Transform, Trigger, TriggerEnabled, TriggerEvent, TriggerOrientation,
    TriggerTiming, TypeAlignment, TypeStorage, UserMapping, View, ViewCheckOption,
};
use dbmd_relational::{
    ForeignKeyAction, ForeignKeyDeferrability, ForeignKeyInitialTiming, ForeignKeyMatch,
    ForeignKeyReference, IndexSortOrder,
};

/// Connection-backed PostgreSQL source selected for introspection.
#[derive(Clone, PartialEq, Eq)]
pub struct PostgresSource {
    id: SourceId,
    display_name: Option<String>,
    connection_url: String,
    include_cluster_objects: bool,
}

impl PostgresSource {
    /// Creates a PostgreSQL source from stable identity and a resolved connection URL.
    #[must_use]
    pub fn new(id: SourceId, connection_url: impl Into<String>) -> Self {
        Self {
            id,
            display_name: None,
            connection_url: connection_url.into(),
            include_cluster_objects: false,
        }
    }

    /// Adds a presentation-only source name.
    #[must_use]
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Selects whether cluster-wide databases, roles, memberships, and tablespaces are included.
    #[must_use]
    pub const fn with_cluster_objects(mut self, include: bool) -> Self {
        self.include_cluster_objects = include;
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
            .field("include_cluster_objects", &self.include_cluster_objects)
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
    let mut tables = load_tables(&mut client, &source.id)?;
    let mut views = load_views(&mut client, &source.id)?;
    load_indexes(&mut client, &source.id, &mut tables, &mut views)?;
    let snapshot = SourceSnapshot::new(
        source.id.clone(),
        Catalog {
            database: load_database(&mut client, &source.id)?,
            cluster_databases: if source.include_cluster_objects {
                load_cluster_databases(&mut client, &source.id)?
            } else {
                Vec::new()
            },
            tablespaces: if source.include_cluster_objects {
                load_tablespaces(&mut client, &source.id)?
            } else {
                Vec::new()
            },
            namespaces: load_namespaces(&mut client, &source.id)?,
            enums: load_enums(&mut client, &source.id)?,
            composite_types: load_composite_types(&mut client, &source.id)?,
            domains: load_domains(&mut client, &source.id)?,
            base_types: load_base_types(&mut client, &source.id)?,
            range_types: load_range_types(&mut client, &source.id)?,
            sequences: load_sequences(&mut client, &source.id)?,
            tables,
            views,
            triggers: load_triggers(&mut client, &source.id)?,
            functions: load_functions(&mut client, &source.id)?,
            procedures: load_procedures(&mut client, &source.id)?,
            aggregates: load_aggregates(&mut client, &source.id)?,
            casts: load_casts(&mut client, &source.id)?,
            conversions: load_conversions(&mut client, &source.id)?,
            operators: load_operators(&mut client, &source.id)?,
            operator_families: load_operator_families(&mut client, &source.id)?,
            operator_classes: load_operator_classes(&mut client, &source.id)?,
            access_methods: load_access_methods(&mut client, &source.id)?,
            languages: load_languages(&mut client, &source.id)?,
            transforms: load_transforms(&mut client, &source.id)?,
            rules: load_rules(&mut client, &source.id)?,
            event_triggers: load_event_triggers(&mut client, &source.id)?,
            statistics: load_extended_statistics(&mut client, &source.id)?,
            foreign_data_wrappers: load_foreign_data_wrappers(&mut client, &source.id)?,
            foreign_servers: load_foreign_servers(&mut client, &source.id)?,
            user_mappings: load_user_mappings(&mut client, &source.id)?,
            text_search_parsers: load_text_search_parsers(&mut client, &source.id)?,
            text_search_templates: load_text_search_templates(&mut client, &source.id)?,
            text_search_dictionaries: load_text_search_dictionaries(&mut client, &source.id)?,
            text_search_configurations: load_text_search_configurations(&mut client, &source.id)?,
            publications: load_publications(&mut client, &source.id)?,
            subscriptions: load_subscriptions(&mut client, &source.id)?,
            roles: if source.include_cluster_objects {
                load_roles(&mut client, &source.id)?
            } else {
                Vec::new()
            },
            role_database_settings: if source.include_cluster_objects {
                load_role_database_settings(&mut client, &source.id)?
            } else {
                Vec::new()
            },
            privileges: load_object_privileges(
                &mut client,
                &source.id,
                source.include_cluster_objects,
            )?,
            default_privileges: load_default_privileges(&mut client, &source.id)?,
            security_labels: load_security_labels(
                &mut client,
                &source.id,
                source.include_cluster_objects,
            )?,
            large_objects: load_large_objects(&mut client, &source.id)?,
            collations: load_collations(&mut client, &source.id)?,
            extensions: load_extensions(&mut client, &source.id)?,
        },
    );
    Ok(match &source.display_name {
        Some(name) => snapshot.with_display_name(name),
        None => snapshot,
    })
}

fn load_database(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Database, IntrospectionError> {
    let row = query(
        client,
        source_id,
        "database",
        r#"
SELECT database.datname,
       owner.rolname,
       pg_catalog.pg_encoding_to_char(database.encoding),
       database.datlocprovider::text,
       database.datcollate,
       database.datctype,
       database.datlocale,
       database.daticurules,
       database.datcollversion,
       tablespace.spcname,
       database.datistemplate,
       database.datallowconn,
       database.datconnlimit,
       COALESCE(ARRAY(
           SELECT setting
           FROM pg_catalog.pg_db_role_setting AS database_setting,
                unnest(database_setting.setconfig) AS setting
           WHERE database_setting.setdatabase = database.oid
             AND database_setting.setrole = 0
           ORDER BY setting COLLATE "C"
       ), ARRAY[]::text[]),
       pg_catalog.shobj_description(database.oid, 'pg_database')
FROM pg_catalog.pg_database AS database
JOIN pg_catalog.pg_roles AS owner ON owner.oid = database.datdba
JOIN pg_catalog.pg_tablespace AS tablespace ON tablespace.oid = database.dattablespace
WHERE database.datname = pg_catalog.current_database()
"#,
    )?
    .into_iter()
    .next()
    .ok_or_else(|| IntrospectionError::CatalogInvariant {
        source_id: source_id.clone(),
        catalog: "pg_database",
        detail: "current database row is missing",
    })?;

    database_from_row(source_id, &row)
}

fn load_cluster_databases(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Database>, IntrospectionError> {
    query(
        client,
        source_id,
        "cluster databases",
        r#"
SELECT database.datname,
       owner.rolname,
       pg_catalog.pg_encoding_to_char(database.encoding),
       database.datlocprovider::text,
       database.datcollate,
       database.datctype,
       database.datlocale,
       database.daticurules,
       database.datcollversion,
       tablespace.spcname,
       database.datistemplate,
       database.datallowconn,
       database.datconnlimit,
       COALESCE(ARRAY(
           SELECT setting
           FROM pg_catalog.pg_db_role_setting AS database_setting,
                unnest(database_setting.setconfig) AS setting
           WHERE database_setting.setdatabase = database.oid
             AND database_setting.setrole = 0
           ORDER BY setting COLLATE "C"
       ), ARRAY[]::text[]),
       pg_catalog.shobj_description(database.oid, 'pg_database')
FROM pg_catalog.pg_database AS database
JOIN pg_catalog.pg_roles AS owner ON owner.oid = database.datdba
JOIN pg_catalog.pg_tablespace AS tablespace ON tablespace.oid = database.dattablespace
ORDER BY database.datname COLLATE "C"
"#,
    )?
    .iter()
    .map(|row| database_from_row(source_id, row))
    .collect()
}

fn database_from_row(source_id: &SourceId, row: &Row) -> Result<Database, IntrospectionError> {
    Ok(Database {
        name: row.get(0),
        owner: row.get(1),
        encoding: row.get(2),
        locale_provider: database_locale_provider(source_id, &row.get::<_, String>(3))?,
        lc_collate: row.get(4),
        lc_ctype: row.get(5),
        locale: row.get(6),
        icu_rules: row.get(7),
        collation_version: row.get(8),
        tablespace: row.get(9),
        template: row.get(10),
        allow_connections: row.get(11),
        connection_limit: row.get(12),
        configuration: row.get(13),
        comment: row.get(14),
    })
}

fn load_tablespaces(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Tablespace>, IntrospectionError> {
    Ok(query(
        client,
        source_id,
        "tablespaces",
        r#"
SELECT tablespace.spcname, owner.rolname,
       COALESCE(tablespace.spcoptions, ARRAY[]::text[]),
       pg_catalog.shobj_description(tablespace.oid, 'pg_tablespace')
FROM pg_catalog.pg_tablespace AS tablespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = tablespace.spcowner
ORDER BY tablespace.spcname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| Tablespace {
        name: row.get(0),
        owner: row.get(1),
        options: row.get(2),
        comment: row.get(3),
        location_redacted: true,
    })
    .collect())
}

fn database_locale_provider(
    source_id: &SourceId,
    value: &str,
) -> Result<DatabaseLocaleProvider, IntrospectionError> {
    match value {
        "b" => Ok(DatabaseLocaleProvider::Builtin),
        "c" => Ok(DatabaseLocaleProvider::Libc),
        "i" => Ok(DatabaseLocaleProvider::Icu),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_database.datlocprovider",
            other,
        )),
    }
}

fn load_extensions(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Extension>, IntrospectionError> {
    let mut extensions = query(
        client,
        source_id,
        "extensions",
        r#"
SELECT extension.extname,
       owner.rolname,
       namespace.nspname,
       extension.extrelocatable,
       extension.extversion,
       pg_catalog.obj_description(extension.oid, 'pg_extension')
FROM pg_catalog.pg_extension AS extension
JOIN pg_catalog.pg_roles AS owner ON owner.oid = extension.extowner
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = extension.extnamespace
ORDER BY extension.extname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| Extension {
        name: row.get(0),
        owner: row.get(1),
        namespace: row.get(2),
        relocatable: row.get(3),
        version: row.get(4),
        configuration: Vec::new(),
        members: Vec::new(),
        comment: row.get(5),
    })
    .collect::<Vec<_>>();
    let positions = extensions
        .iter()
        .enumerate()
        .map(|(index, extension)| (extension.name.clone(), index))
        .collect::<HashMap<_, _>>();

    for row in query(
        client,
        source_id,
        "extension configuration tables",
        r#"
SELECT extension.extname,
       pg_catalog.format('%I.%I', namespace.nspname, relation.relname),
       extension.extcondition[position]
FROM pg_catalog.pg_extension AS extension
CROSS JOIN LATERAL pg_catalog.generate_subscripts(extension.extconfig, 1) AS position
JOIN pg_catalog.pg_class AS relation ON relation.oid = extension.extconfig[position]
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace
ORDER BY extension.extname COLLATE "C", position
"#,
    )? {
        let extension_name = row.get::<_, String>(0);
        if let Some(index) = positions.get(&extension_name) {
            extensions[*index]
                .configuration
                .push(ExtensionConfiguration {
                    relation: row.get(1),
                    condition: row.get(2),
                });
        }
    }

    for row in query(
        client,
        source_id,
        "extension members",
        r#"
SELECT extension.extname,
       object_address.object_type,
       object_address.object_names,
       object_address.object_arguments
FROM pg_catalog.pg_depend AS dependency
JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
CROSS JOIN LATERAL pg_catalog.pg_identify_object_as_address(
    dependency.classid,
    dependency.objid,
    dependency.objsubid
) AS object_address(object_type, object_names, object_arguments)
WHERE dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
  AND dependency.deptype = 'e'
ORDER BY extension.extname COLLATE "C",
         object_address.object_type COLLATE "C",
         object_address.object_names::text COLLATE "C",
         object_address.object_arguments::text COLLATE "C"
"#,
    )? {
        let extension_name = row.get::<_, String>(0);
        if let Some(index) = positions.get(&extension_name) {
            extensions[*index].members.push(ExtensionMember {
                object_type: row.get(1),
                names: row.get(2),
                arguments: row.get(3),
            });
        }
    }
    Ok(extensions)
}

fn load_composite_types(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<CompositeType>, IntrospectionError> {
    let rows = query(
        client,
        source_id,
        "composite types",
        r#"
SELECT type_record.oid::bigint, namespace.nspname, type_record.typname,
       owner.rolname, pg_catalog.obj_description(type_record.oid, 'pg_type'),
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
             AND extension_dependency.objid = type_record.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       )
FROM pg_catalog.pg_type AS type_record
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = type_record.typnamespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = type_record.typowner
JOIN pg_catalog.pg_class AS relation ON relation.oid = type_record.typrelid
WHERE type_record.typtype = 'c' AND relation.relkind = 'c'
  AND namespace.nspname <> 'information_schema' AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C", type_record.typname COLLATE "C"
"#,
    )?;
    let mut composites = Vec::with_capacity(rows.len());
    let mut composite_by_oid = HashMap::with_capacity(rows.len());
    for row in rows {
        composite_by_oid.insert(row.get::<_, i64>(0), composites.len());
        composites.push(CompositeType {
            namespace: row.get(1),
            name: row.get(2),
            owner: row.get(3),
            extension: row.get(5),
            attributes: Vec::new(),
            comment: row.get(4),
            definition: String::new(),
        });
    }
    for row in query(
        client,
        source_id,
        "composite type attributes",
        r#"
SELECT type_record.oid::bigint, attribute.attname,
       pg_catalog.format_type(attribute.atttypid, attribute.atttypmod),
       CASE WHEN attribute.attcollation = 0 THEN NULL ELSE
           pg_catalog.format('%I.%I', collation_namespace.nspname, collation_record.collname)
       END,
       pg_catalog.col_description(relation.oid, attribute.attnum)
FROM pg_catalog.pg_type AS type_record
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = type_record.typnamespace
JOIN pg_catalog.pg_class AS relation ON relation.oid = type_record.typrelid
JOIN pg_catalog.pg_attribute AS attribute ON attribute.attrelid = relation.oid
LEFT JOIN pg_catalog.pg_collation AS collation_record ON collation_record.oid = attribute.attcollation
LEFT JOIN pg_catalog.pg_namespace AS collation_namespace
       ON collation_namespace.oid = collation_record.collnamespace
WHERE type_record.typtype = 'c' AND relation.relkind = 'c'
  AND namespace.nspname <> 'information_schema' AND namespace.nspname !~ '^pg_'
  AND attribute.attnum > 0 AND NOT attribute.attisdropped
ORDER BY namespace.nspname COLLATE "C", type_record.typname COLLATE "C", attribute.attnum
"#,
    )? {
        let Some(&index) = composite_by_oid.get(&row.get::<_, i64>(0)) else {
            continue;
        };
        composites[index].attributes.push(CompositeAttribute {
            name: row.get(1),
            data_type: row.get(2),
            collation: row.get(3),
            comment: row.get(4),
        });
    }
    for composite in &mut composites {
        let attributes = composite
            .attributes
            .iter()
            .map(|attribute| {
                let mut value = format!(
                    "{} {}",
                    quote_postgres_identifier(&attribute.name),
                    attribute.data_type
                );
                if let Some(collation) = &attribute.collation {
                    let _ = fmt::Write::write_fmt(&mut value, format_args!(" COLLATE {collation}"));
                }
                value
            })
            .collect::<Vec<_>>()
            .join(",\n    ");
        composite.definition = format!(
            "CREATE TYPE {}.{} AS (\n    {}\n);",
            quote_postgres_identifier(&composite.namespace),
            quote_postgres_identifier(&composite.name),
            attributes
        );
    }
    Ok(composites)
}

fn load_publications(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Publication>, IntrospectionError> {
    let rows = query(
        client,
        source_id,
        "publications",
        r#"
SELECT publication.oid::bigint, publication.pubname, owner.rolname,
       publication.puballtables, publication.pubinsert, publication.pubupdate,
       publication.pubdelete, publication.pubtruncate, publication.pubviaroot,
       publication.pubgencols::text,
       pg_catalog.obj_description(publication.oid, 'pg_publication')
FROM pg_catalog.pg_publication AS publication
JOIN pg_catalog.pg_roles AS owner ON owner.oid = publication.pubowner
ORDER BY publication.pubname COLLATE "C"
"#,
    )?;
    let mut publications = Vec::with_capacity(rows.len());
    let mut publication_by_oid = HashMap::with_capacity(rows.len());
    for row in rows {
        publication_by_oid.insert(row.get::<_, i64>(0), publications.len());
        publications.push(Publication {
            name: row.get(1),
            owner: row.get(2),
            all_tables: row.get(3),
            publish_insert: row.get(4),
            publish_update: row.get(5),
            publish_delete: row.get(6),
            publish_truncate: row.get(7),
            publish_via_partition_root: row.get(8),
            generated_columns: publication_generated_columns(source_id, &row.get::<_, String>(9))?,
            schemas: Vec::new(),
            tables: Vec::new(),
            comment: row.get(10),
        });
    }
    for row in query(
        client,
        source_id,
        "publication schemas",
        r#"
SELECT mapping.pnpubid::bigint, namespace.nspname
FROM pg_catalog.pg_publication_namespace AS mapping
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = mapping.pnnspid
ORDER BY mapping.pnpubid, namespace.nspname COLLATE "C"
"#,
    )? {
        if let Some(&index) = publication_by_oid.get(&row.get::<_, i64>(0)) {
            publications[index].schemas.push(row.get(1));
        }
    }
    for row in query(
        client,
        source_id,
        "publication tables",
        r#"
SELECT mapping.prpubid::bigint, namespace.nspname, relation.relname,
       CASE WHEN mapping.prattrs IS NULL THEN NULL ELSE ARRAY(
           SELECT attribute.attname
           FROM unnest(mapping.prattrs::smallint[]) WITH ORDINALITY AS selected(attnum, ordinal)
           JOIN pg_catalog.pg_attribute AS attribute
             ON attribute.attrelid = mapping.prrelid AND attribute.attnum = selected.attnum
           ORDER BY selected.ordinal
       ) END,
       pg_catalog.pg_get_expr(mapping.prqual, mapping.prrelid, true)
FROM pg_catalog.pg_publication_rel AS mapping
JOIN pg_catalog.pg_class AS relation ON relation.oid = mapping.prrelid
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace
ORDER BY mapping.prpubid, namespace.nspname COLLATE "C", relation.relname COLLATE "C"
"#,
    )? {
        if let Some(&index) = publication_by_oid.get(&row.get::<_, i64>(0)) {
            publications[index].tables.push(PublicationTable {
                namespace: row.get(1),
                name: row.get(2),
                columns: row.get(3),
                row_filter: row.get(4),
            });
        }
    }
    Ok(publications)
}

fn publication_generated_columns(
    source_id: &SourceId,
    value: &str,
) -> Result<PublicationGeneratedColumns, IntrospectionError> {
    match value {
        "n" => Ok(PublicationGeneratedColumns::None),
        "s" => Ok(PublicationGeneratedColumns::Stored),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_publication.pubgencols",
            other,
        )),
    }
}

fn load_subscriptions(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Subscription>, IntrospectionError> {
    query(
        client,
        source_id,
        "subscriptions",
        r#"
SELECT subscription.subname, owner.rolname, subscription.subenabled,
       subscription.subbinary, subscription.substream::text,
       subscription.subtwophasestate::text, subscription.subdisableonerr,
       subscription.subpasswordrequired, subscription.subrunasowner,
       subscription.subfailover, subscription.subslotname,
       subscription.subsynccommit, subscription.subpublications,
       subscription.suborigin,
       NULLIF(subscription.subskiplsn, '0/0'::pg_catalog.pg_lsn)::text,
       pg_catalog.obj_description(subscription.oid, 'pg_subscription')
FROM pg_catalog.pg_subscription AS subscription
JOIN pg_catalog.pg_roles AS owner ON owner.oid = subscription.subowner
WHERE subscription.subdbid = (SELECT oid FROM pg_catalog.pg_database WHERE datname = pg_catalog.current_database())
ORDER BY subscription.subname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        Ok(Subscription {
            name: row.get(0),
            owner: row.get(1),
            enabled: row.get(2),
            binary: row.get(3),
            streaming: subscription_streaming(source_id, &row.get::<_, String>(4))?,
            two_phase: subscription_two_phase(source_id, &row.get::<_, String>(5))?,
            disable_on_error: row.get(6),
            password_required: row.get(7),
            run_as_owner: row.get(8),
            failover: row.get(9),
            slot_name: row.get(10),
            synchronous_commit: synchronous_commit(source_id, &row.get::<_, String>(11))?,
            publications: row.get(12),
            origin: subscription_origin(source_id, &row.get::<_, String>(13))?,
            skip_lsn: row.get(14),
            connection_redacted: true,
            comment: row.get(15),
        })
    })
    .collect()
}

fn subscription_streaming(
    source_id: &SourceId,
    value: &str,
) -> Result<SubscriptionStreaming, IntrospectionError> {
    match value {
        "f" => Ok(SubscriptionStreaming::Off),
        "t" => Ok(SubscriptionStreaming::On),
        "p" => Ok(SubscriptionStreaming::Parallel),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_subscription.substream",
            other,
        )),
    }
}

fn subscription_two_phase(
    source_id: &SourceId,
    value: &str,
) -> Result<SubscriptionTwoPhase, IntrospectionError> {
    match value {
        "d" => Ok(SubscriptionTwoPhase::Disabled),
        "p" => Ok(SubscriptionTwoPhase::Pending),
        "e" => Ok(SubscriptionTwoPhase::Enabled),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_subscription.subtwophasestate",
            other,
        )),
    }
}

fn synchronous_commit(
    source_id: &SourceId,
    value: &str,
) -> Result<SynchronousCommit, IntrospectionError> {
    match value {
        "off" => Ok(SynchronousCommit::Off),
        "local" => Ok(SynchronousCommit::Local),
        "remote_write" => Ok(SynchronousCommit::RemoteWrite),
        "on" => Ok(SynchronousCommit::On),
        "remote_apply" => Ok(SynchronousCommit::RemoteApply),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_subscription.subsynccommit",
            other,
        )),
    }
}

fn subscription_origin(
    source_id: &SourceId,
    value: &str,
) -> Result<SubscriptionOrigin, IntrospectionError> {
    match value {
        "none" => Ok(SubscriptionOrigin::None),
        "any" => Ok(SubscriptionOrigin::Any),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_subscription.suborigin",
            other,
        )),
    }
}

fn load_roles(client: &mut Client, source_id: &SourceId) -> Result<Vec<Role>, IntrospectionError> {
    let rows = query(
        client,
        source_id,
        "roles",
        r#"
SELECT role.oid::bigint, role.rolname, role.rolsuper, role.rolinherit,
       role.rolcreaterole, role.rolcreatedb, role.rolcanlogin,
       role.rolreplication, role.rolbypassrls, role.rolconnlimit,
       role.rolvaliduntil::text, role.rolpassword IS NOT NULL,
       COALESCE(ARRAY(
           SELECT setting
           FROM pg_catalog.pg_db_role_setting AS role_setting,
                unnest(role_setting.setconfig) AS setting
           WHERE role_setting.setrole = role.oid
             AND role_setting.setdatabase = 0
           ORDER BY setting COLLATE "C"
       ), ARRAY[]::text[]),
       pg_catalog.shobj_description(role.oid, 'pg_authid')
FROM pg_catalog.pg_roles AS role
WHERE role.oid >= 16384
ORDER BY role.rolname COLLATE "C"
"#,
    )?;
    let mut roles = Vec::with_capacity(rows.len());
    let mut role_by_oid = HashMap::with_capacity(rows.len());
    for row in rows {
        role_by_oid.insert(row.get::<_, i64>(0), roles.len());
        roles.push(Role {
            name: row.get(1),
            superuser: row.get(2),
            inherit: row.get(3),
            create_role: row.get(4),
            create_database: row.get(5),
            login: row.get(6),
            replication: row.get(7),
            bypass_row_level_security: row.get(8),
            connection_limit: row.get(9),
            valid_until: row.get(10),
            password_configured: row.get(11),
            configuration: row.get(12),
            memberships: Vec::new(),
            comment: row.get(13),
        });
    }
    for row in query(
        client,
        source_id,
        "role memberships",
        r#"
SELECT membership.member::bigint, granted.rolname, grantor.rolname,
       membership.admin_option, membership.inherit_option, membership.set_option
FROM pg_catalog.pg_auth_members AS membership
JOIN pg_catalog.pg_roles AS granted ON granted.oid = membership.roleid
JOIN pg_catalog.pg_roles AS member ON member.oid = membership.member
JOIN pg_catalog.pg_roles AS grantor ON grantor.oid = membership.grantor
WHERE member.oid >= 16384
ORDER BY member.rolname COLLATE "C", granted.rolname COLLATE "C", grantor.rolname COLLATE "C"
"#,
    )? {
        if let Some(&index) = role_by_oid.get(&row.get::<_, i64>(0)) {
            roles[index].memberships.push(RoleMembership {
                role: row.get(1),
                grantor: row.get(2),
                admin: row.get(3),
                inherit: row.get(4),
                set: row.get(5),
            });
        }
    }
    Ok(roles)
}

fn load_role_database_settings(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<RoleDatabaseSetting>, IntrospectionError> {
    Ok(query(
        client,
        source_id,
        "role database settings",
        r#"
SELECT database.datname, role.rolname,
       ARRAY(
           SELECT setting
           FROM unnest(role_setting.setconfig) AS setting
           ORDER BY setting COLLATE "C"
       )
FROM pg_catalog.pg_db_role_setting AS role_setting
JOIN pg_catalog.pg_database AS database ON database.oid = role_setting.setdatabase
JOIN pg_catalog.pg_roles AS role ON role.oid = role_setting.setrole
WHERE role_setting.setrole <> 0
  AND role_setting.setdatabase <> 0
ORDER BY database.datname COLLATE "C", role.rolname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| RoleDatabaseSetting {
        database: row.get(0),
        role: row.get(1),
        settings: row.get(2),
    })
    .collect())
}

fn load_collations(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Collation>, IntrospectionError> {
    query(client, source_id, "collations", r#"
SELECT namespace.nspname, collation_record.collname, owner.rolname, extension.extname,
       collation_record.collprovider::text, collation_record.collisdeterministic,
       CASE WHEN collation_record.collencoding = -1 THEN NULL
            ELSE pg_catalog.pg_encoding_to_char(collation_record.collencoding) END,
       collation_record.collcollate, collation_record.collctype, collation_record.colllocale,
       collation_record.collicurules, collation_record.collversion,
       pg_catalog.obj_description(collation_record.oid, 'pg_collation')
FROM pg_catalog.pg_collation AS collation_record
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = collation_record.collnamespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = collation_record.collowner
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_collation'::pg_catalog.regclass
 AND dependency.objid = collation_record.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE namespace.nspname <> 'information_schema' AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C", collation_record.collname COLLATE "C"
"#)?
    .into_iter()
    .map(|row| Ok(Collation {
        namespace: row.get(0), name: row.get(1), owner: row.get(2), extension: row.get(3),
        provider: collation_provider(source_id, &row.get::<_, String>(4))?,
        deterministic: row.get(5), encoding: row.get(6), lc_collate: row.get(7),
        lc_ctype: row.get(8), locale: row.get(9), icu_rules: row.get(10),
        version: row.get(11), comment: row.get(12),
    }))
    .collect()
}

fn collation_provider(
    source_id: &SourceId,
    value: &str,
) -> Result<CollationProvider, IntrospectionError> {
    match value {
        "d" => Ok(CollationProvider::DatabaseDefault),
        "b" => Ok(CollationProvider::Builtin),
        "c" => Ok(CollationProvider::Libc),
        "i" => Ok(CollationProvider::Icu),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_collation.collprovider",
            other,
        )),
    }
}

fn load_domains(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Domain>, IntrospectionError> {
    query(client, source_id, "domains", r#"
SELECT namespace.nspname, type_record.typname,
       pg_catalog.format_type(type_record.typbasetype, type_record.typtypmod),
       CASE WHEN collation_record.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', collation_namespace.nspname, collation_record.collname)
       END,
       pg_catalog.pg_get_expr(type_record.typdefaultbin, 0, true), type_record.typnotnull,
       owner.rolname, pg_catalog.obj_description(type_record.oid, 'pg_type'),
       ARRAY(SELECT conname FROM pg_catalog.pg_constraint WHERE contypid = type_record.oid AND contype = 'c' ORDER BY conname COLLATE "C"),
       ARRAY(SELECT pg_catalog.pg_get_constraintdef(oid, true) FROM pg_catalog.pg_constraint WHERE contypid = type_record.oid AND contype = 'c' ORDER BY conname COLLATE "C"),
       ARRAY(SELECT convalidated FROM pg_catalog.pg_constraint WHERE contypid = type_record.oid AND contype = 'c' ORDER BY conname COLLATE "C"),
       ARRAY(SELECT pg_catalog.obj_description(oid, 'pg_constraint') FROM pg_catalog.pg_constraint WHERE contypid = type_record.oid AND contype = 'c' ORDER BY conname COLLATE "C"),
       extension.extname
FROM pg_catalog.pg_type AS type_record
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = type_record.typnamespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = type_record.typowner
LEFT JOIN pg_catalog.pg_collation AS collation_record
       ON collation_record.oid = NULLIF(type_record.typcollation, 0)
LEFT JOIN pg_catalog.pg_namespace AS collation_namespace
       ON collation_namespace.oid = collation_record.collnamespace
LEFT JOIN pg_catalog.pg_depend AS dependency ON dependency.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
 AND dependency.objid = type_record.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE type_record.typtype = 'd' AND namespace.nspname <> 'information_schema' AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C", type_record.typname COLLATE "C"
"#)?
    .into_iter()
    .map(|row| {
        let namespace: String = row.get(0);
        let name: String = row.get(1);
        let base_type: String = row.get(2);
        let collation: Option<String> = row.get(3);
        let default: Option<String> = row.get(4);
        let not_null: bool = row.get(5);
        let constraints = row.get::<_, Vec<String>>(8).into_iter()
            .zip(row.get::<_, Vec<String>>(9)).zip(row.get::<_, Vec<bool>>(10))
            .zip(row.get::<_, Vec<Option<String>>>(11))
            .map(|(((name, definition), validated), comment)| DomainConstraint { name, definition, validated, comment })
            .collect::<Vec<_>>();
        let mut clauses = vec![format!("AS {base_type}")];
        if let Some(value) = &collation { clauses.push(format!("COLLATE {value}")); }
        if let Some(value) = &default { clauses.push(format!("DEFAULT {value}")); }
        if not_null { clauses.push("NOT NULL".into()); }
        clauses.extend(constraints.iter().map(|constraint| format!("CONSTRAINT {} {}{}",
            quote_postgres_identifier(&constraint.name), constraint.definition,
            if constraint.validated { "" } else { " NOT VALID" })));
        Ok(Domain {
            namespace: namespace.clone(), name: name.clone(), base_type, collation, default, not_null,
            owner: row.get(6), comment: row.get(7), constraints, extension: row.get(12),
            definition: format!("CREATE DOMAIN {}.{}\n    {};", quote_postgres_identifier(&namespace),
                quote_postgres_identifier(&name), clauses.join("\n    ")),
        })
    }).collect()
}

fn load_base_types(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<BaseType>, IntrospectionError> {
    query(
        client,
        source_id,
        "base and shell types",
        r#"
SELECT namespace.nspname,
       type_record.typname,
       owner.rolname,
       type_record.typisdefined,
       type_record.typlen,
       type_record.typbyval,
       type_record.typcategory::text,
       type_record.typispreferred,
       type_record.typdelim::text,
       CASE WHEN input_function.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', input_namespace.nspname, input_function.proname)
       END,
       CASE WHEN output_function.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', output_namespace.nspname, output_function.proname)
       END,
       CASE WHEN receive_function.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', receive_namespace.nspname, receive_function.proname)
       END,
       CASE WHEN send_function.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', send_namespace.nspname, send_function.proname)
       END,
       CASE WHEN typmod_in_function.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', typmod_in_namespace.nspname, typmod_in_function.proname)
       END,
       CASE WHEN typmod_out_function.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', typmod_out_namespace.nspname, typmod_out_function.proname)
       END,
       CASE WHEN analyze_function.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', analyze_namespace.nspname, analyze_function.proname)
       END,
       CASE WHEN subscript_function.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', subscript_namespace.nspname, subscript_function.proname)
       END,
       CASE WHEN type_record.typelem = 0 THEN NULL
            ELSE pg_catalog.format_type(type_record.typelem, NULL)
       END,
       CASE WHEN array_type.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', array_namespace.nspname, array_type.typname)
       END,
       type_record.typalign::text,
       type_record.typstorage::text,
       type_record.typcollation <> 0,
       type_record.typdefault,
       pg_catalog.obj_description(type_record.oid, 'pg_type'),
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
             AND extension_dependency.objid = type_record.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       )
FROM pg_catalog.pg_type AS type_record
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = type_record.typnamespace
JOIN pg_catalog.pg_roles AS owner
  ON owner.oid = type_record.typowner
LEFT JOIN pg_catalog.pg_proc AS input_function ON input_function.oid = NULLIF(type_record.typinput, 0)
LEFT JOIN pg_catalog.pg_namespace AS input_namespace ON input_namespace.oid = input_function.pronamespace
LEFT JOIN pg_catalog.pg_proc AS output_function ON output_function.oid = NULLIF(type_record.typoutput, 0)
LEFT JOIN pg_catalog.pg_namespace AS output_namespace ON output_namespace.oid = output_function.pronamespace
LEFT JOIN pg_catalog.pg_proc AS receive_function ON receive_function.oid = NULLIF(type_record.typreceive, 0)
LEFT JOIN pg_catalog.pg_namespace AS receive_namespace ON receive_namespace.oid = receive_function.pronamespace
LEFT JOIN pg_catalog.pg_proc AS send_function ON send_function.oid = NULLIF(type_record.typsend, 0)
LEFT JOIN pg_catalog.pg_namespace AS send_namespace ON send_namespace.oid = send_function.pronamespace
LEFT JOIN pg_catalog.pg_proc AS typmod_in_function ON typmod_in_function.oid = NULLIF(type_record.typmodin, 0)
LEFT JOIN pg_catalog.pg_namespace AS typmod_in_namespace ON typmod_in_namespace.oid = typmod_in_function.pronamespace
LEFT JOIN pg_catalog.pg_proc AS typmod_out_function ON typmod_out_function.oid = NULLIF(type_record.typmodout, 0)
LEFT JOIN pg_catalog.pg_namespace AS typmod_out_namespace ON typmod_out_namespace.oid = typmod_out_function.pronamespace
LEFT JOIN pg_catalog.pg_proc AS analyze_function ON analyze_function.oid = NULLIF(type_record.typanalyze, 0)
LEFT JOIN pg_catalog.pg_namespace AS analyze_namespace ON analyze_namespace.oid = analyze_function.pronamespace
LEFT JOIN pg_catalog.pg_proc AS subscript_function ON subscript_function.oid = NULLIF(type_record.typsubscript, 0)
LEFT JOIN pg_catalog.pg_namespace AS subscript_namespace ON subscript_namespace.oid = subscript_function.pronamespace
LEFT JOIN pg_catalog.pg_type AS array_type ON array_type.oid = NULLIF(type_record.typarray, 0)
LEFT JOIN pg_catalog.pg_namespace AS array_namespace ON array_namespace.oid = array_type.typnamespace
WHERE (NOT type_record.typisdefined OR (type_record.typtype = 'b' AND type_record.typarray <> 0))
  AND namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C", type_record.typname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        let namespace = row.get::<_, String>(0);
        let name = row.get::<_, String>(1);
        let defined = row.get::<_, bool>(3);
        let details = if defined {
            Some(BaseTypeDetails {
                internal_length: row.get(4),
                passed_by_value: row.get(5),
                category: row.get(6),
                preferred: row.get(7),
                delimiter: row.get(8),
                input: required_catalog_text(source_id, "pg_type.typinput", row.get(9))?,
                output: required_catalog_text(source_id, "pg_type.typoutput", row.get(10))?,
                receive: row.get(11),
                send: row.get(12),
                type_modifier_input: row.get(13),
                type_modifier_output: row.get(14),
                analyze: row.get(15),
                subscript: row.get(16),
                element_type: row.get(17),
                alignment: type_alignment(source_id, &row.get::<_, String>(19))?,
                storage: type_storage(source_id, &row.get::<_, String>(20))?,
                collatable: row.get(21),
                default: row.get(22),
            })
        } else {
            None
        };
        let array_type = row.get::<_, Option<String>>(18);
        let definition = base_type_definition(&namespace, &name, details.as_ref(), array_type.as_deref());
        Ok(BaseType {
            namespace,
            name,
            owner: row.get(2),
            extension: row.get(24),
            defined,
            details,
            array_type,
            comment: row.get(23),
            definition,
        })
    })
    .collect()
}

fn load_range_types(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<RangeType>, IntrospectionError> {
    query(
        client,
        source_id,
        "range and multirange types",
        r#"
SELECT range_namespace.nspname,
       range_type.typname,
       range_owner.rolname,
       pg_catalog.format_type(range_record.rngsubtype, NULL),
       pg_catalog.format('%I.%I', opclass_namespace.nspname, operator_class.opcname),
       CASE WHEN collation_record.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', collation_namespace.nspname, collation_record.collname)
       END,
       CASE WHEN canonical_function.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', canonical_namespace.nspname, canonical_function.proname)
       END,
       CASE WHEN diff_function.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', diff_namespace.nspname, diff_function.proname)
       END,
       multirange_namespace.nspname,
       multirange_type.typname,
       multirange_owner.rolname,
       pg_catalog.obj_description(range_type.oid, 'pg_type'),
       pg_catalog.obj_description(multirange_type.oid, 'pg_type'),
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
             AND extension_dependency.objid = range_type.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       ),
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
             AND extension_dependency.objid = multirange_type.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       )
FROM pg_catalog.pg_range AS range_record
JOIN pg_catalog.pg_type AS range_type ON range_type.oid = range_record.rngtypid
JOIN pg_catalog.pg_namespace AS range_namespace ON range_namespace.oid = range_type.typnamespace
JOIN pg_catalog.pg_roles AS range_owner ON range_owner.oid = range_type.typowner
JOIN pg_catalog.pg_opclass AS operator_class ON operator_class.oid = range_record.rngsubopc
JOIN pg_catalog.pg_namespace AS opclass_namespace ON opclass_namespace.oid = operator_class.opcnamespace
LEFT JOIN pg_catalog.pg_collation AS collation_record ON collation_record.oid = NULLIF(range_record.rngcollation, 0)
LEFT JOIN pg_catalog.pg_namespace AS collation_namespace ON collation_namespace.oid = collation_record.collnamespace
LEFT JOIN pg_catalog.pg_proc AS canonical_function ON canonical_function.oid = NULLIF(range_record.rngcanonical, 0)
LEFT JOIN pg_catalog.pg_namespace AS canonical_namespace ON canonical_namespace.oid = canonical_function.pronamespace
LEFT JOIN pg_catalog.pg_proc AS diff_function ON diff_function.oid = NULLIF(range_record.rngsubdiff, 0)
LEFT JOIN pg_catalog.pg_namespace AS diff_namespace ON diff_namespace.oid = diff_function.pronamespace
JOIN pg_catalog.pg_type AS multirange_type ON multirange_type.oid = range_record.rngmultitypid
JOIN pg_catalog.pg_namespace AS multirange_namespace ON multirange_namespace.oid = multirange_type.typnamespace
JOIN pg_catalog.pg_roles AS multirange_owner ON multirange_owner.oid = multirange_type.typowner
WHERE range_namespace.nspname <> 'information_schema'
  AND range_namespace.nspname !~ '^pg_'
ORDER BY range_namespace.nspname COLLATE "C", range_type.typname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        let namespace = row.get::<_, String>(0);
        let name = row.get::<_, String>(1);
        let subtype = row.get::<_, String>(3);
        let subtype_operator_class = row.get::<_, String>(4);
        let collation = row.get::<_, Option<String>>(5);
        let canonical = row.get::<_, Option<String>>(6);
        let subtype_diff = row.get::<_, Option<String>>(7);
        let multirange_namespace = row.get::<_, String>(8);
        let multirange_name = row.get::<_, String>(9);
        let definition = range_type_definition(
            &namespace,
            &name,
            &subtype,
            &subtype_operator_class,
            collation.as_deref(),
            canonical.as_deref(),
            subtype_diff.as_deref(),
            &multirange_namespace,
            &multirange_name,
        );
        Ok(RangeType {
            namespace,
            name,
            owner: row.get(2),
            extension: row.get(13),
            subtype,
            subtype_operator_class,
            collation,
            canonical,
            subtype_diff,
            multirange: MultirangeType {
                namespace: multirange_namespace,
                name: multirange_name,
                owner: row.get(10),
                extension: row.get(14),
                comment: row.get(12),
            },
            comment: row.get(11),
            definition,
        })
    })
    .collect()
}

fn load_sequences(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Sequence>, IntrospectionError> {
    query(
        client,
        source_id,
        "sequences",
        r#"
SELECT namespace.nspname,
       relation.relname,
       owner.rolname,
       pg_catalog.format_type(sequence.seqtypid, NULL),
       sequence.seqstart,
       sequence.seqmin,
       sequence.seqmax,
       sequence.seqincrement,
       sequence.seqcache,
       sequence.seqcycle,
       relation.relpersistence::text,
       CASE WHEN owned_attribute.attname IS NULL THEN NULL
            ELSE pg_catalog.format(
                '%I.%I.%I',
                owned_namespace.nspname,
                owned_relation.relname,
                owned_attribute.attname
            )
       END,
       pg_catalog.obj_description(relation.oid, 'pg_class'),
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
             AND extension_dependency.objid = relation.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       )
FROM pg_catalog.pg_class AS relation
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = relation.relnamespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = relation.relowner
JOIN pg_catalog.pg_sequence AS sequence
  ON sequence.seqrelid = relation.oid
LEFT JOIN pg_catalog.pg_depend AS ownership
  ON ownership.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
 AND ownership.objid = relation.oid
 AND ownership.objsubid = 0
 AND ownership.refclassid = 'pg_catalog.pg_class'::pg_catalog.regclass
 AND ownership.deptype IN ('a', 'i')
LEFT JOIN pg_catalog.pg_class AS owned_relation
  ON owned_relation.oid = ownership.refobjid
LEFT JOIN pg_catalog.pg_namespace AS owned_namespace
  ON owned_namespace.oid = owned_relation.relnamespace
LEFT JOIN pg_catalog.pg_attribute AS owned_attribute
  ON owned_attribute.attrelid = ownership.refobjid
 AND owned_attribute.attnum = ownership.refobjsubid
WHERE relation.relkind = 'S'
  AND namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C", relation.relname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        let namespace = row.get::<_, String>(0);
        let name = row.get::<_, String>(1);
        let data_type = row.get::<_, String>(3);
        let start = row.get(4);
        let minimum = row.get(5);
        let maximum = row.get(6);
        let increment = row.get(7);
        let cache = row.get(8);
        let cycle = row.get(9);
        let persistence = sequence_persistence(source_id, &row.get::<_, String>(10))?;
        let owned_by = row.get::<_, Option<String>>(11);
        let definition = format!(
            "CREATE {}SEQUENCE {}.{} AS {} INCREMENT BY {} MINVALUE {} MAXVALUE {} START WITH {} CACHE {} {} OWNED BY {};",
            match persistence {
                SequencePersistence::Permanent => "",
                SequencePersistence::Unlogged => "UNLOGGED ",
            },
            quote_postgres_identifier(&namespace),
            quote_postgres_identifier(&name),
            data_type,
            increment,
            minimum,
            maximum,
            start,
            cache,
            if cycle { "CYCLE" } else { "NO CYCLE" },
            owned_by.as_deref().unwrap_or("NONE")
        );
        Ok(Sequence {
            namespace,
            name,
            owner: row.get(2),
            extension: row.get(13),
            data_type,
            start,
            minimum,
            maximum,
            increment,
            cache,
            cycle,
            persistence,
            owned_by,
            comment: row.get(12),
            definition,
        })
    })
    .collect()
}

fn sequence_persistence(
    source_id: &SourceId,
    value: &str,
) -> Result<SequencePersistence, IntrospectionError> {
    match value {
        "p" => Ok(SequencePersistence::Permanent),
        "u" => Ok(SequencePersistence::Unlogged),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_class.relpersistence",
            other,
        )),
    }
}

fn quote_postgres_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn quote_postgres_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn base_type_definition(
    namespace: &str,
    name: &str,
    details: Option<&BaseTypeDetails>,
    array_type: Option<&str>,
) -> String {
    let qualified_name = format!(
        "{}.{}",
        quote_postgres_identifier(namespace),
        quote_postgres_identifier(name)
    );
    let Some(details) = details else {
        return format!("CREATE TYPE {qualified_name};");
    };
    let mut properties = vec![
        format!("INPUT = {}", details.input),
        format!("OUTPUT = {}", details.output),
    ];
    for (label, value) in [
        ("RECEIVE", details.receive.as_deref()),
        ("SEND", details.send.as_deref()),
        ("TYPMOD_IN", details.type_modifier_input.as_deref()),
        ("TYPMOD_OUT", details.type_modifier_output.as_deref()),
        ("ANALYZE", details.analyze.as_deref()),
        ("SUBSCRIPT", details.subscript.as_deref()),
    ] {
        if let Some(value) = value {
            properties.push(format!("{label} = {value}"));
        }
    }
    properties.push(format!(
        "INTERNALLENGTH = {}",
        if details.internal_length == -1 {
            "VARIABLE".to_string()
        } else {
            details.internal_length.to_string()
        }
    ));
    if details.passed_by_value {
        properties.push("PASSEDBYVALUE".to_string());
    }
    properties.push(format!(
        "ALIGNMENT = {}",
        match details.alignment {
            TypeAlignment::Char => "char",
            TypeAlignment::Short => "int2",
            TypeAlignment::Int => "int4",
            TypeAlignment::Double => "double",
        }
    ));
    properties.push(format!(
        "STORAGE = {}",
        match details.storage {
            TypeStorage::Plain => "plain",
            TypeStorage::External => "external",
            TypeStorage::Main => "main",
            TypeStorage::Extended => "extended",
        }
    ));
    properties.push(format!(
        "CATEGORY = {}",
        quote_postgres_literal(&details.category)
    ));
    if details.preferred {
        properties.push("PREFERRED = true".to_string());
    }
    if let Some(default) = &details.default {
        properties.push(format!("DEFAULT = {}", quote_postgres_literal(default)));
    }
    if let Some(element_type) = &details.element_type {
        properties.push(format!("ELEMENT = {element_type}"));
    }
    if details.delimiter != "," {
        properties.push(format!(
            "DELIMITER = {}",
            quote_postgres_literal(&details.delimiter)
        ));
    }
    if details.collatable {
        properties.push("COLLATABLE = true".to_string());
    }
    if let Some(array_type) = array_type {
        properties.push(format!("ARRAY_TYPE = {array_type}"));
    }
    format!(
        "CREATE TYPE {qualified_name} (\n    {}\n);",
        properties.join(",\n    ")
    )
}

#[allow(clippy::too_many_arguments)]
fn range_type_definition(
    namespace: &str,
    name: &str,
    subtype: &str,
    subtype_operator_class: &str,
    collation: Option<&str>,
    canonical: Option<&str>,
    subtype_diff: Option<&str>,
    multirange_namespace: &str,
    multirange_name: &str,
) -> String {
    let mut properties = vec![
        format!("SUBTYPE = {subtype}"),
        format!("SUBTYPE_OPCLASS = {subtype_operator_class}"),
    ];
    if let Some(collation) = collation {
        properties.push(format!("COLLATION = {collation}"));
    }
    if let Some(canonical) = canonical {
        properties.push(format!("CANONICAL = {canonical}"));
    }
    if let Some(subtype_diff) = subtype_diff {
        properties.push(format!("SUBTYPE_DIFF = {subtype_diff}"));
    }
    properties.push(format!(
        "MULTIRANGE_TYPE_NAME = {}.{}",
        quote_postgres_identifier(multirange_namespace),
        quote_postgres_identifier(multirange_name)
    ));
    format!(
        "CREATE TYPE {}.{} AS RANGE (\n    {}\n);",
        quote_postgres_identifier(namespace),
        quote_postgres_identifier(name),
        properties.join(",\n    ")
    )
}

fn required_catalog_text(
    source_id: &SourceId,
    catalog: &'static str,
    value: Option<String>,
) -> Result<String, IntrospectionError> {
    value.ok_or_else(|| IntrospectionError::CatalogInvariant {
        source_id: source_id.clone(),
        catalog,
        detail: "required function reference is absent",
    })
}

fn type_alignment(source_id: &SourceId, value: &str) -> Result<TypeAlignment, IntrospectionError> {
    match value {
        "c" => Ok(TypeAlignment::Char),
        "s" => Ok(TypeAlignment::Short),
        "i" => Ok(TypeAlignment::Int),
        "d" => Ok(TypeAlignment::Double),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_type.typalign",
            other,
        )),
    }
}

fn type_storage(source_id: &SourceId, value: &str) -> Result<TypeStorage, IntrospectionError> {
    match value {
        "p" => Ok(TypeStorage::Plain),
        "e" => Ok(TypeStorage::External),
        "m" => Ok(TypeStorage::Main),
        "x" => Ok(TypeStorage::Extended),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_type.typstorage",
            other,
        )),
    }
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
       owner.rolname,
       pg_catalog.obj_description(namespace.oid, 'pg_namespace'),
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_namespace'::pg_catalog.regclass
             AND extension_dependency.objid = namespace.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       )
FROM pg_catalog.pg_namespace AS namespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = namespace.nspowner
WHERE namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        Ok(Namespace {
            name: row.get(0),
            owner: row.get(1),
            extension: row.get(3),
            comment: row.get(2),
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
       owner.rolname,
       pg_catalog.obj_description(type_record.oid, 'pg_type'),
       ARRAY(
           SELECT enum_value.enumlabel
           FROM pg_catalog.pg_enum AS enum_value
           WHERE enum_value.enumtypid = type_record.oid
           ORDER BY enum_value.enumsortorder
       ),
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_type'::pg_catalog.regclass
             AND extension_dependency.objid = type_record.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       )
FROM pg_catalog.pg_type AS type_record
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = type_record.typnamespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = type_record.typowner
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
            owner: row.get(2),
            extension: row.get(5),
            comment: row.get(3),
            values: row.get(4),
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
           SELECT pg_catalog.format('%I.%I', parent_namespace.nspname, parent_relation.relname)
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
           SELECT pg_catalog.format('%I.%I', parent_namespace.nspname, parent_relation.relname)
           FROM pg_catalog.pg_inherits AS inheritance
           JOIN pg_catalog.pg_class AS parent_relation
             ON parent_relation.oid = inheritance.inhparent
           JOIN pg_catalog.pg_namespace AS parent_namespace
             ON parent_namespace.oid = parent_relation.relnamespace
           WHERE inheritance.inhrelid = relation.oid
           ORDER BY inheritance.inhseqno
       ),
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
             AND extension_dependency.objid = relation.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       ),
       owner.rolname,
       relation.relpersistence::text,
       access_method.amname,
       CASE WHEN relation.reloftype = 0 THEN NULL
            ELSE pg_catalog.format_type(relation.reloftype, NULL)
       END,
       relation.relreplident::text,
       COALESCE(relation.reloptions, ARRAY[]::text[]),
       foreign_server.srvname,
       foreign_wrapper.fdwname,
       ARRAY(
           SELECT CASE
                    WHEN pg_catalog.split_part(option, '=', 1) ~* '(password|secret|token|credential|private[_-]?key)'
                    THEN pg_catalog.split_part(option, '=', 1) || '=<redacted>'
                    ELSE option
                  END
           FROM unnest(COALESCE(foreign_table.ftoptions, ARRAY[]::text[])) WITH ORDINALITY AS item(option, position)
           ORDER BY position
       )
FROM pg_catalog.pg_class AS relation
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = relation.relnamespace
LEFT JOIN pg_catalog.pg_tablespace AS tablespace
  ON tablespace.oid = NULLIF(relation.reltablespace, 0)
JOIN pg_catalog.pg_roles AS owner ON owner.oid = relation.relowner
LEFT JOIN pg_catalog.pg_am AS access_method ON access_method.oid = NULLIF(relation.relam, 0)
LEFT JOIN pg_catalog.pg_foreign_table AS foreign_table ON foreign_table.ftrelid = relation.oid
LEFT JOIN pg_catalog.pg_foreign_server AS foreign_server ON foreign_server.oid = foreign_table.ftserver
LEFT JOIN pg_catalog.pg_foreign_data_wrapper AS foreign_wrapper ON foreign_wrapper.oid = foreign_server.srvfdw
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
            extension: row.get(13),
            comment: row.get(4),
            columns: Vec::new(),
            constraints: Vec::new(),
            indexes: Vec::new(),
            kind: table_kind(source_id, row.get::<_, String>(3).as_str(), row.get(9))?,
            owner: row.get(14),
            persistence: relation_persistence(source_id, &row.get::<_, String>(15))?,
            access_method: row.get(16),
            typed_table: row.get(17),
            replica_identity: replica_identity(source_id, &row.get::<_, String>(18))?,
            options: row.get(19),
            foreign: row.get::<_, Option<String>>(20).map(|server| ForeignTable {
                server,
                wrapper: row.get::<_, Option<String>>(21).unwrap_or_default(),
                options: row.get(22),
            }),
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
       ),
       CASE WHEN collation_record.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', collation_namespace.nspname, collation_record.collname)
       END,
       attribute.attstorage::text,
       NULLIF(attribute.attcompression::text, ''),
       attribute.attstattarget::text,
       COALESCE(attribute.attoptions, ARRAY[]::text[]),
       ARRAY(
           SELECT CASE
                    WHEN pg_catalog.split_part(option, '=', 1) ~* '(password|secret|token|credential|private[_-]?key)'
                    THEN pg_catalog.split_part(option, '=', 1) || '=<redacted>'
                    ELSE option
                  END
           FROM unnest(COALESCE(attribute.attfdwoptions, ARRAY[]::text[])) WITH ORDINALITY AS item(option, position)
           ORDER BY position
       ),
       attribute.attislocal,
       attribute.attinhcount::integer
FROM pg_catalog.pg_attribute AS attribute
JOIN pg_catalog.pg_class AS relation
  ON relation.oid = attribute.attrelid
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = relation.relnamespace
LEFT JOIN pg_catalog.pg_attrdef AS default_value
  ON default_value.adrelid = attribute.attrelid
 AND default_value.adnum = attribute.attnum
LEFT JOIN pg_catalog.pg_collation AS collation_record
  ON collation_record.oid = NULLIF(attribute.attcollation, 0)
LEFT JOIN pg_catalog.pg_namespace AS collation_namespace
  ON collation_namespace.oid = collation_record.collnamespace
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
            "" => None,
            "a" => Some(IdentityGeneration::Always),
            "d" => Some(IdentityGeneration::ByDefault),
            value => {
                return Err(unsupported_catalog_value(
                    source_id,
                    "pg_attribute.attidentity",
                    value,
                ));
            }
        };
        let generated = match generated_code.as_str() {
            "" => None,
            "s" => expression.clone().map(|expression| GeneratedColumn {
                expression,
                kind: GeneratedColumnKind::Stored,
            }),
            "v" => expression.clone().map(|expression| GeneratedColumn {
                expression,
                kind: GeneratedColumnKind::Virtual,
            }),
            value => {
                return Err(unsupported_catalog_value(
                    source_id,
                    "generated column kind",
                    value,
                ));
            }
        };
        tables[table_index].columns.push(postgres_column(
            source_id,
            &row,
            identity,
            generated,
            generated_code.is_empty().then_some(expression).flatten(),
        )?);
    }
    Ok(())
}

fn postgres_column(
    source_id: &SourceId,
    row: &Row,
    identity: Option<IdentityGeneration>,
    generated: Option<GeneratedColumn>,
    default: Option<String>,
) -> Result<Column, IntrospectionError> {
    let statistics_target = row
        .get::<_, Option<String>>(12)
        .as_deref()
        .unwrap_or("-1")
        .parse::<i32>()
        .map_err(|_| IntrospectionError::CatalogInvariant {
            source_id: source_id.clone(),
            catalog: "pg_attribute.attstattarget",
            detail: "statistics target is not a signed integer",
        })?;
    Ok(Column {
        name: row.get(1),
        data_type: row.get(2),
        nullable: Some(row.get(3)),
        default,
        comment: row.get(5),
        enum_values: row.get(8),
        collation: row.get(9),
        identity,
        generated,
        storage: column_storage(source_id, &row.get::<_, String>(10))?,
        compression: row
            .get::<_, Option<String>>(11)
            .as_deref()
            .map(|value| column_compression(source_id, value))
            .transpose()?,
        statistics_target,
        options: row.get(13),
        foreign_options: row.get(14),
        locally_defined: row.get(15),
        inheritance_count: row.get(16),
    })
}

fn column_storage(source_id: &SourceId, value: &str) -> Result<ColumnStorage, IntrospectionError> {
    match value {
        "p" => Ok(ColumnStorage::Plain),
        "e" => Ok(ColumnStorage::External),
        "m" => Ok(ColumnStorage::Main),
        "x" => Ok(ColumnStorage::Extended),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_attribute.attstorage",
            other,
        )),
    }
}

fn column_compression(
    source_id: &SourceId,
    value: &str,
) -> Result<ColumnCompression, IntrospectionError> {
    match value {
        "p" => Ok(ColumnCompression::Pglz),
        "l" => Ok(ColumnCompression::Lz4),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_attribute.attcompression",
            other,
        )),
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
       pg_catalog.pg_get_viewdef(relation.oid, true),
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
             AND extension_dependency.objid = relation.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       ),
       owner.rolname, relation.relpersistence::text, access_method.amname,
       tablespace.spcname, COALESCE(relation.reloptions, ARRAY[]::text[]),
       relation.relispopulated
FROM pg_catalog.pg_class AS relation
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = relation.relnamespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = relation.relowner
LEFT JOIN pg_catalog.pg_am AS access_method ON access_method.oid = NULLIF(relation.relam, 0)
LEFT JOIN pg_catalog.pg_tablespace AS tablespace ON tablespace.oid = NULLIF(relation.reltablespace, 0)
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
        let options = row.get::<_, Vec<String>>(11);
        views.push(View {
            namespace: row.get(1),
            name: row.get(2),
            extension: row.get(6),
            materialized: row.get(3),
            comment: row.get(4),
            definition: row.get(5),
            columns: Vec::new(),
            indexes: Vec::new(),
            owner: row.get(7),
            persistence: relation_persistence(source_id, &row.get::<_, String>(8))?,
            access_method: row.get(9),
            tablespace: row.get(10),
            security_barrier: relation_option_enabled(&options, "security_barrier"),
            security_invoker: relation_option_enabled(&options, "security_invoker"),
            check_option: view_check_option(source_id, &options)?,
            options,
            populated: row.get(12),
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
       ),
       CASE WHEN collation_record.oid IS NULL THEN NULL
            ELSE pg_catalog.format('%I.%I', collation_namespace.nspname, collation_record.collname)
       END,
       attribute.attstorage::text,
       NULLIF(attribute.attcompression::text, ''),
       attribute.attstattarget::text,
       COALESCE(attribute.attoptions, ARRAY[]::text[]),
       ARRAY(
           SELECT CASE
                    WHEN pg_catalog.split_part(option, '=', 1) ~* '(password|secret|token|credential|private[_-]?key)'
                    THEN pg_catalog.split_part(option, '=', 1) || '=<redacted>'
                    ELSE option
                  END
           FROM unnest(COALESCE(attribute.attfdwoptions, ARRAY[]::text[])) WITH ORDINALITY AS item(option, position)
           ORDER BY position
       ),
       attribute.attislocal,
       attribute.attinhcount::integer
FROM pg_catalog.pg_attribute AS attribute
JOIN pg_catalog.pg_class AS relation
  ON relation.oid = attribute.attrelid
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = relation.relnamespace
LEFT JOIN pg_catalog.pg_collation AS collation_record
  ON collation_record.oid = NULLIF(attribute.attcollation, 0)
LEFT JOIN pg_catalog.pg_namespace AS collation_namespace
  ON collation_namespace.oid = collation_record.collnamespace
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
            .push(postgres_column(source_id, &row, None, None, None)?);
    }
    Ok(views)
}

fn relation_option_enabled(options: &[String], name: &str) -> bool {
    options.iter().any(|option| {
        option == name
            || option
                .strip_prefix(name)
                .and_then(|value| value.strip_prefix('='))
                .is_some_and(|value| matches!(value, "true" | "on" | "1"))
    })
}

fn view_check_option(
    source_id: &SourceId,
    options: &[String],
) -> Result<Option<ViewCheckOption>, IntrospectionError> {
    let Some(value) = options
        .iter()
        .find_map(|option| option.strip_prefix("check_option="))
    else {
        return Ok(None);
    };
    match value {
        "local" => Ok(Some(ViewCheckOption::Local)),
        "cascaded" => Ok(Some(ViewCheckOption::Cascaded)),
        other => Err(unsupported_catalog_value(
            source_id,
            "view check_option",
            other,
        )),
    }
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
       END,
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_trigger'::pg_catalog.regclass
             AND extension_dependency.objid = trigger_record.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       ),
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
             AND extension_dependency.objid = relation.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       )
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
        let extension = row
            .get::<_, Option<String>>(25)
            .or_else(|| row.get::<_, Option<String>>(26));
        Ok(Trigger {
            namespace,
            name,
            extension,
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
       pg_catalog.pg_get_function_arguments(procedure.oid),
       pg_catalog.pg_get_functiondef(procedure.oid),
       pg_catalog.obj_description(procedure.oid, 'pg_proc'),
       pg_catalog.pg_get_function_result(procedure.oid),
       owner.rolname,
       procedure.prokind::text,
       language.lanname,
       procedure.provolatile::text,
       procedure.proparallel::text,
       procedure.prosecdef,
       procedure.proisstrict,
       procedure.proleakproof,
       procedure.proretset,
       CASE WHEN procedure.prosupport = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, procedure.prosupport, 0)).identity
       END,
       procedure.procost::text,
       CASE WHEN procedure.proretset THEN procedure.prorows::text ELSE NULL END,
       COALESCE(procedure.proconfig, ARRAY[]::text[]),
       COALESCE(ARRAY(
           SELECT pg_catalog.format_type(transform_type, NULL)
           FROM unnest(procedure.protrftypes::oid[]) WITH ORDINALITY AS transform(transform_type, ordinal)
           ORDER BY ordinal
       ), ARRAY[]::text[]),
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
             AND extension_dependency.objid = procedure.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       )
FROM pg_catalog.pg_proc AS procedure
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = procedure.pronamespace
JOIN pg_catalog.pg_language AS language
  ON language.oid = procedure.prolang
JOIN pg_catalog.pg_roles AS owner
  ON owner.oid = procedure.proowner
WHERE procedure.prokind IN ('f', 'w')
  AND namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
  AND NOT EXISTS (
      SELECT 1
      FROM pg_catalog.pg_depend AS dependency
      WHERE dependency.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
        AND dependency.objid = procedure.oid
        AND dependency.deptype = 'i'
  )
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
            extension: row.get(21),
            signature: row.get(2),
            arguments: row.get(3),
            definition: row.get(4),
            comment: row.get(5),
            return_type: row.get(6),
            owner: row.get(7),
            kind: function_kind(source_id, &row.get::<_, String>(8))?,
            language: row.get(9),
            volatility: function_volatility(source_id, &row.get::<_, String>(10))?,
            parallel: function_parallel(source_id, &row.get::<_, String>(11))?,
            security_definer: row.get(12),
            strict: row.get(13),
            leakproof: row.get(14),
            returns_set: row.get(15),
            support_function: row.get(16),
            cost: row.get(17),
            rows: row.get(18),
            configuration: row.get(19),
            transforms: row.get(20),
        })
    })
    .collect()
}

fn function_kind(source_id: &SourceId, value: &str) -> Result<FunctionKind, IntrospectionError> {
    match value {
        "f" => Ok(FunctionKind::Ordinary),
        "w" => Ok(FunctionKind::Window),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_proc.prokind",
            other,
        )),
    }
}

fn load_procedures(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Procedure>, IntrospectionError> {
    query(
        client,
        source_id,
        "procedures",
        r#"
SELECT namespace.nspname,
       procedure.proname,
       '(' || pg_catalog.pg_get_function_identity_arguments(procedure.oid) || ')',
       pg_catalog.pg_get_function_arguments(procedure.oid),
       pg_catalog.pg_get_functiondef(procedure.oid),
       pg_catalog.obj_description(procedure.oid, 'pg_proc'),
       owner.rolname,
       language.lanname,
       procedure.prosecdef,
       COALESCE(procedure.proconfig, ARRAY[]::text[]),
       COALESCE(ARRAY(
           SELECT pg_catalog.format_type(transform_type, NULL)
           FROM unnest(procedure.protrftypes::oid[]) WITH ORDINALITY AS transform(transform_type, ordinal)
           ORDER BY ordinal
       ), ARRAY[]::text[]),
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
             AND extension_dependency.objid = procedure.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       )
FROM pg_catalog.pg_proc AS procedure
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = procedure.pronamespace
JOIN pg_catalog.pg_roles AS owner
  ON owner.oid = procedure.proowner
JOIN pg_catalog.pg_language AS language
  ON language.oid = procedure.prolang
WHERE procedure.prokind = 'p'
  AND namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
  AND NOT EXISTS (
      SELECT 1
      FROM pg_catalog.pg_depend AS dependency
      WHERE dependency.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
        AND dependency.objid = procedure.oid
        AND dependency.deptype = 'i'
  )
ORDER BY namespace.nspname COLLATE "C",
         procedure.proname COLLATE "C",
         pg_catalog.pg_get_function_identity_arguments(procedure.oid) COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        Ok(Procedure {
            namespace: row.get(0),
            name: row.get(1),
            extension: row.get(11),
            signature: row.get(2),
            arguments: row.get(3),
            definition: row.get(4),
            comment: row.get(5),
            owner: row.get(6),
            language: row.get(7),
            security_definer: row.get(8),
            configuration: row.get(9),
            transforms: row.get(10),
        })
    })
    .collect()
}

fn load_aggregates(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Aggregate>, IntrospectionError> {
    query(
        client,
        source_id,
        "aggregates",
        r#"
SELECT namespace.nspname,
       procedure.proname,
       '(' || pg_catalog.pg_get_function_identity_arguments(procedure.oid) || ')',
       pg_catalog.pg_get_function_arguments(procedure.oid),
       owner.rolname,
       pg_catalog.pg_get_function_result(procedure.oid),
       aggregate.aggkind::text,
       aggregate.aggnumdirectargs,
       (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, aggregate.aggtransfn, 0)).identity,
       CASE WHEN aggregate.aggfinalfn = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, aggregate.aggfinalfn, 0)).identity
       END,
       CASE WHEN aggregate.aggcombinefn = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, aggregate.aggcombinefn, 0)).identity
       END,
       CASE WHEN aggregate.aggserialfn = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, aggregate.aggserialfn, 0)).identity
       END,
       CASE WHEN aggregate.aggdeserialfn = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, aggregate.aggdeserialfn, 0)).identity
       END,
       CASE WHEN aggregate.aggmtransfn = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, aggregate.aggmtransfn, 0)).identity
       END,
       CASE WHEN aggregate.aggminvtransfn = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, aggregate.aggminvtransfn, 0)).identity
       END,
       CASE WHEN aggregate.aggmfinalfn = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, aggregate.aggmfinalfn, 0)).identity
       END,
       aggregate.aggfinalextra,
       aggregate.aggmfinalextra,
       aggregate.aggfinalmodify::text,
       aggregate.aggmfinalmodify::text,
       CASE WHEN aggregate.aggsortop = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_operator'::pg_catalog.regclass, aggregate.aggsortop, 0)).identity
       END,
       pg_catalog.format_type(aggregate.aggtranstype, NULL),
       aggregate.aggtransspace,
       CASE WHEN aggregate.aggmtranstype = 0 THEN NULL
            ELSE pg_catalog.format_type(aggregate.aggmtranstype, NULL)
       END,
       aggregate.aggmtransspace,
       aggregate.agginitval,
       aggregate.aggminitval,
       procedure.proparallel::text,
       pg_catalog.obj_description(procedure.oid, 'pg_proc'),
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
             AND extension_dependency.objid = procedure.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       )
FROM pg_catalog.pg_aggregate AS aggregate
JOIN pg_catalog.pg_proc AS procedure ON procedure.oid = aggregate.aggfnoid
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = procedure.pronamespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = procedure.proowner
WHERE namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
  AND NOT EXISTS (
      SELECT 1
      FROM pg_catalog.pg_depend AS dependency
      WHERE dependency.classid = 'pg_catalog.pg_proc'::pg_catalog.regclass
        AND dependency.objid = procedure.oid
        AND dependency.deptype = 'i'
  )
ORDER BY namespace.nspname COLLATE "C",
         procedure.proname COLLATE "C",
         pg_catalog.pg_get_function_identity_arguments(procedure.oid) COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        Ok(Aggregate {
            namespace: row.get(0),
            name: row.get(1),
            extension: row.get(29),
            signature: row.get(2),
            arguments: row.get(3),
            owner: row.get(4),
            result_type: row.get(5),
            kind: aggregate_kind(source_id, &row.get::<_, String>(6))?,
            direct_arguments: row.get(7),
            transition_function: row.get(8),
            final_function: row.get(9),
            combine_function: row.get(10),
            serialization_function: row.get(11),
            deserialization_function: row.get(12),
            moving_transition_function: row.get(13),
            moving_inverse_function: row.get(14),
            moving_final_function: row.get(15),
            final_extra_arguments: row.get(16),
            moving_final_extra_arguments: row.get(17),
            final_modify: aggregate_final_modify(source_id, &row.get::<_, String>(18))?,
            moving_final_modify: aggregate_final_modify(
                source_id,
                &row.get::<_, String>(19),
            )?,
            sort_operator: row.get(20),
            transition_type: row.get(21),
            transition_space: row.get(22),
            moving_transition_type: row.get(23),
            moving_transition_space: row.get(24),
            initial_condition: row.get(25),
            moving_initial_condition: row.get(26),
            parallel: function_parallel(source_id, &row.get::<_, String>(27))?,
            comment: row.get(28),
        })
    })
    .collect()
}

fn aggregate_kind(source_id: &SourceId, value: &str) -> Result<AggregateKind, IntrospectionError> {
    match value {
        "n" => Ok(AggregateKind::Normal),
        "o" => Ok(AggregateKind::OrderedSet),
        "h" => Ok(AggregateKind::HypotheticalSet),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_aggregate.aggkind",
            other,
        )),
    }
}

fn aggregate_final_modify(
    source_id: &SourceId,
    value: &str,
) -> Result<AggregateFinalModify, IntrospectionError> {
    match value {
        "r" => Ok(AggregateFinalModify::ReadOnly),
        "s" => Ok(AggregateFinalModify::Shareable),
        "w" => Ok(AggregateFinalModify::ReadWrite),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_aggregate final-function mutation mode",
            other,
        )),
    }
}

fn load_casts(client: &mut Client, source_id: &SourceId) -> Result<Vec<Cast>, IntrospectionError> {
    query(
        client,
        source_id,
        "casts",
        r#"
SELECT pg_catalog.format_type(cast_record.castsource, NULL),
       pg_catalog.format_type(cast_record.casttarget, NULL),
       cast_record.castcontext::text,
       cast_record.castmethod::text,
       CASE WHEN cast_record.castfunc = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, cast_record.castfunc, 0)).identity
       END,
       (
           SELECT extension.extname
           FROM pg_catalog.pg_depend AS dependency
           JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
           WHERE dependency.classid = 'pg_catalog.pg_cast'::pg_catalog.regclass
             AND dependency.objid = cast_record.oid
             AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND dependency.deptype = 'e'
       ),
       pg_catalog.obj_description(cast_record.oid, 'pg_cast')
FROM pg_catalog.pg_cast AS cast_record
WHERE cast_record.oid >= 16384
  AND NOT EXISTS (
      SELECT 1 FROM pg_catalog.pg_depend AS internal_dependency
      WHERE internal_dependency.classid = 'pg_catalog.pg_cast'::pg_catalog.regclass
        AND internal_dependency.objid = cast_record.oid
        AND internal_dependency.objsubid = 0
        AND internal_dependency.deptype = 'i'
  )
ORDER BY pg_catalog.format_type(cast_record.castsource, NULL) COLLATE "C",
         pg_catalog.format_type(cast_record.casttarget, NULL) COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        Ok(Cast {
            source_type: row.get(0),
            target_type: row.get(1),
            context: cast_context(source_id, &row.get::<_, String>(2))?,
            method: cast_method(source_id, &row.get::<_, String>(3))?,
            function: row.get(4),
            extension: row.get(5),
            comment: row.get(6),
        })
    })
    .collect()
}

fn load_conversions(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Conversion>, IntrospectionError> {
    query(
        client,
        source_id,
        "conversions",
        r#"
SELECT namespace.nspname,
       conversion.conname,
       owner.rolname,
       pg_catalog.pg_encoding_to_char(conversion.conforencoding),
       pg_catalog.pg_encoding_to_char(conversion.contoencoding),
       (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, conversion.conproc, 0)).identity,
       conversion.condefault,
       pg_catalog.obj_description(conversion.oid, 'pg_conversion'),
       (
           SELECT extension.extname
           FROM pg_catalog.pg_depend AS dependency
           JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
           WHERE dependency.classid = 'pg_catalog.pg_conversion'::pg_catalog.regclass
             AND dependency.objid = conversion.oid
             AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND dependency.deptype = 'e'
       )
FROM pg_catalog.pg_conversion AS conversion
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = conversion.connamespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = conversion.conowner
WHERE namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
  AND NOT EXISTS (
      SELECT 1 FROM pg_catalog.pg_depend AS internal_dependency
      WHERE internal_dependency.classid = 'pg_catalog.pg_conversion'::pg_catalog.regclass
        AND internal_dependency.objid = conversion.oid
        AND internal_dependency.objsubid = 0
        AND internal_dependency.deptype = 'i'
  )
ORDER BY namespace.nspname COLLATE "C", conversion.conname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        Ok(Conversion {
            namespace: row.get(0),
            name: row.get(1),
            owner: row.get(2),
            source_encoding: row.get(3),
            target_encoding: row.get(4),
            function: row.get(5),
            default: row.get(6),
            comment: row.get(7),
            extension: row.get(8),
        })
    })
    .collect()
}

fn load_operators(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Operator>, IntrospectionError> {
    query(
        client,
        source_id,
        "operators",
        r#"
SELECT namespace.nspname,
       operator_record.oprname,
       owner.rolname,
       CASE WHEN operator_record.oprleft = 0 THEN NULL ELSE pg_catalog.format_type(operator_record.oprleft, NULL) END,
       pg_catalog.format_type(operator_record.oprright, NULL),
       pg_catalog.format_type(operator_record.oprresult, NULL),
       (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, operator_record.oprcode, 0)).identity,
       CASE WHEN operator_record.oprcom = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_operator'::pg_catalog.regclass, operator_record.oprcom, 0)).identity
       END,
       CASE WHEN operator_record.oprnegate = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_operator'::pg_catalog.regclass, operator_record.oprnegate, 0)).identity
       END,
       CASE WHEN operator_record.oprrest = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, operator_record.oprrest, 0)).identity
       END,
       CASE WHEN operator_record.oprjoin = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, operator_record.oprjoin, 0)).identity
       END,
       operator_record.oprcanmerge,
       operator_record.oprcanhash,
       pg_catalog.obj_description(operator_record.oid, 'pg_operator'),
       (
           SELECT extension.extname
           FROM pg_catalog.pg_depend AS dependency
           JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
           WHERE dependency.classid = 'pg_catalog.pg_operator'::pg_catalog.regclass
             AND dependency.objid = operator_record.oid
             AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND dependency.deptype = 'e'
       )
FROM pg_catalog.pg_operator AS operator_record
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = operator_record.oprnamespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = operator_record.oprowner
WHERE namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
  AND operator_record.oprkind IN ('b', 'l')
  AND NOT EXISTS (
      SELECT 1 FROM pg_catalog.pg_depend AS internal_dependency
      WHERE internal_dependency.classid = 'pg_catalog.pg_operator'::pg_catalog.regclass
        AND internal_dependency.objid = operator_record.oid
        AND internal_dependency.objsubid = 0
        AND internal_dependency.deptype = 'i'
  )
ORDER BY namespace.nspname COLLATE "C", operator_record.oprname COLLATE "C",
         pg_catalog.format_type(operator_record.oprleft, NULL) COLLATE "C",
         pg_catalog.format_type(operator_record.oprright, NULL) COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        let left_type: Option<String> = row.get(3);
        Ok(Operator {
            namespace: row.get(0),
            name: row.get(1),
            owner: row.get(2),
            extension: row.get(14),
            kind: if left_type.is_some() {
                OperatorKind::Binary
            } else {
                OperatorKind::Prefix
            },
            left_type,
            right_type: row.get(4),
            result_type: row.get(5),
            function: row.get(6),
            commutator: row.get(7),
            negator: row.get(8),
            restriction_selectivity: row.get(9),
            join_selectivity: row.get(10),
            can_merge: row.get(11),
            can_hash: row.get(12),
            comment: row.get(13),
        })
    })
    .collect()
}

fn load_operator_families(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<OperatorFamily>, IntrospectionError> {
    let rows = query(
        client,
        source_id,
        "operator families",
        r#"
SELECT family.oid::bigint, namespace.nspname, family.opfname, owner.rolname,
       access_method.amname, extension.extname,
       pg_catalog.obj_description(family.oid, 'pg_opfamily')
FROM pg_catalog.pg_opfamily AS family
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = family.opfnamespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = family.opfowner
JOIN pg_catalog.pg_am AS access_method ON access_method.oid = family.opfmethod
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_opfamily'::pg_catalog.regclass
 AND dependency.objid = family.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
 AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE family.oid >= 16384
  AND NOT EXISTS (
      SELECT 1 FROM pg_catalog.pg_depend AS internal_dependency
      WHERE internal_dependency.classid = 'pg_catalog.pg_opfamily'::pg_catalog.regclass
        AND internal_dependency.objid = family.oid
        AND internal_dependency.objsubid = 0
        AND internal_dependency.deptype = 'i'
  )
ORDER BY access_method.amname COLLATE "C", namespace.nspname COLLATE "C", family.opfname COLLATE "C"
"#,
    )?;
    let mut families = Vec::with_capacity(rows.len());
    let mut family_by_oid = HashMap::with_capacity(rows.len());
    for row in rows {
        family_by_oid.insert(row.get::<_, i64>(0), families.len());
        families.push(OperatorFamily {
            namespace: row.get(1),
            name: row.get(2),
            owner: row.get(3),
            access_method: row.get(4),
            extension: row.get(5),
            operators: Vec::new(),
            functions: Vec::new(),
            comment: row.get(6),
        });
    }
    for row in query(
        client,
        source_id,
        "operator family operators",
        r#"
SELECT member.amopfamily::bigint,
       pg_catalog.format_type(member.amoplefttype, NULL),
       pg_catalog.format_type(member.amoprighttype, NULL),
       member.amopstrategy, member.amoppurpose::text,
       (pg_catalog.pg_identify_object('pg_catalog.pg_operator'::pg_catalog.regclass, member.amopopr, 0)).identity,
       access_method.amname,
       CASE WHEN member.amopsortfamily = 0 THEN NULL
            ELSE pg_catalog.format('%I.%I', sort_namespace.nspname, sort_family.opfname) END
FROM pg_catalog.pg_amop AS member
JOIN pg_catalog.pg_opfamily AS family ON family.oid = member.amopfamily
JOIN pg_catalog.pg_am AS access_method ON access_method.oid = member.amopmethod
LEFT JOIN pg_catalog.pg_opfamily AS sort_family ON sort_family.oid = member.amopsortfamily
LEFT JOIN pg_catalog.pg_namespace AS sort_namespace ON sort_namespace.oid = sort_family.opfnamespace
WHERE family.oid >= 16384
ORDER BY member.amopfamily, member.amopstrategy, member.amoplefttype, member.amoprighttype
"#,
    )? {
        if let Some(&index) = family_by_oid.get(&row.get::<_, i64>(0)) {
            families[index].operators.push(OperatorFamilyOperator {
                left_type: row.get(1),
                right_type: row.get(2),
                strategy: row.get(3),
                purpose: operator_purpose(source_id, &row.get::<_, String>(4))?,
                operator: row.get(5),
                access_method: row.get(6),
                sort_family: row.get(7),
            });
        }
    }
    for row in query(
        client,
        source_id,
        "operator family functions",
        r#"
SELECT member.amprocfamily::bigint,
       pg_catalog.format_type(member.amproclefttype, NULL),
       pg_catalog.format_type(member.amprocrighttype, NULL),
       member.amprocnum,
       (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, member.amproc, 0)).identity
FROM pg_catalog.pg_amproc AS member
JOIN pg_catalog.pg_opfamily AS family ON family.oid = member.amprocfamily
WHERE family.oid >= 16384
ORDER BY member.amprocfamily, member.amprocnum, member.amproclefttype, member.amprocrighttype
"#,
    )? {
        if let Some(&index) = family_by_oid.get(&row.get::<_, i64>(0)) {
            families[index].functions.push(OperatorFamilyFunction {
                left_type: row.get(1),
                right_type: row.get(2),
                number: row.get(3),
                function: row.get(4),
            });
        }
    }
    Ok(families)
}

fn load_operator_classes(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<OperatorClass>, IntrospectionError> {
    Ok(query(
        client,
        source_id,
        "operator classes",
        r#"
SELECT namespace.nspname, class.opcname, owner.rolname, extension.extname,
       access_method.amname,
       pg_catalog.format('%I.%I', family_namespace.nspname, family.opfname),
       pg_catalog.format_type(class.opcintype, NULL), class.opcdefault,
       CASE WHEN class.opckeytype = 0 THEN NULL ELSE pg_catalog.format_type(class.opckeytype, NULL) END,
       pg_catalog.obj_description(class.oid, 'pg_opclass')
FROM pg_catalog.pg_opclass AS class
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = class.opcnamespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = class.opcowner
JOIN pg_catalog.pg_am AS access_method ON access_method.oid = class.opcmethod
JOIN pg_catalog.pg_opfamily AS family ON family.oid = class.opcfamily
JOIN pg_catalog.pg_namespace AS family_namespace ON family_namespace.oid = family.opfnamespace
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_opclass'::pg_catalog.regclass
 AND dependency.objid = class.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
 AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE class.oid >= 16384
  AND NOT EXISTS (
      SELECT 1 FROM pg_catalog.pg_depend AS internal_dependency
      WHERE internal_dependency.classid = 'pg_catalog.pg_opclass'::pg_catalog.regclass
        AND internal_dependency.objid = class.oid
        AND internal_dependency.objsubid = 0
        AND internal_dependency.deptype = 'i'
  )
ORDER BY access_method.amname COLLATE "C", namespace.nspname COLLATE "C", class.opcname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| OperatorClass {
        namespace: row.get(0), name: row.get(1), owner: row.get(2), extension: row.get(3),
        access_method: row.get(4), family: row.get(5), input_type: row.get(6),
        default: row.get(7), key_type: row.get(8), comment: row.get(9),
    })
    .collect())
}

fn load_access_methods(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<AccessMethod>, IntrospectionError> {
    query(
        client,
        source_id,
        "access methods",
        r#"
SELECT access_method.amname, extension.extname, access_method.amtype::text,
       (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, access_method.amhandler, 0)).identity,
       pg_catalog.obj_description(access_method.oid, 'pg_am')
FROM pg_catalog.pg_am AS access_method
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_am'::pg_catalog.regclass
 AND dependency.objid = access_method.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
 AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE access_method.oid >= 16384
  AND NOT EXISTS (
      SELECT 1 FROM pg_catalog.pg_depend AS internal_dependency
      WHERE internal_dependency.classid = 'pg_catalog.pg_am'::pg_catalog.regclass
        AND internal_dependency.objid = access_method.oid
        AND internal_dependency.objsubid = 0
        AND internal_dependency.deptype = 'i'
  )
ORDER BY access_method.amname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        Ok(AccessMethod {
            name: row.get(0),
            extension: row.get(1),
            kind: access_method_kind(source_id, &row.get::<_, String>(2))?,
            handler: row.get(3),
            comment: row.get(4),
        })
    })
    .collect()
}

fn load_languages(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Language>, IntrospectionError> {
    Ok(query(
        client,
        source_id,
        "procedural languages",
        r#"
SELECT language.lanname, owner.rolname, extension.extname,
       language.lanispl, language.lanpltrusted,
       CASE WHEN language.lanplcallfoid = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, language.lanplcallfoid, 0)).identity END,
       CASE WHEN language.laninline = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, language.laninline, 0)).identity END,
       CASE WHEN language.lanvalidator = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, language.lanvalidator, 0)).identity END,
       pg_catalog.obj_description(language.oid, 'pg_language')
FROM pg_catalog.pg_language AS language
JOIN pg_catalog.pg_roles AS owner ON owner.oid = language.lanowner
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_language'::pg_catalog.regclass
 AND dependency.objid = language.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
 AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE language.lanispl
  AND NOT EXISTS (
      SELECT 1 FROM pg_catalog.pg_depend AS internal_dependency
      WHERE internal_dependency.classid = 'pg_catalog.pg_language'::pg_catalog.regclass
        AND internal_dependency.objid = language.oid
        AND internal_dependency.objsubid = 0
        AND internal_dependency.deptype = 'i'
  )
ORDER BY language.lanname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| Language {
        name: row.get(0), owner: row.get(1), extension: row.get(2),
        procedural: row.get(3), trusted: row.get(4), handler: row.get(5),
        inline_handler: row.get(6), validator: row.get(7), comment: row.get(8),
    })
    .collect())
}

fn load_transforms(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<Transform>, IntrospectionError> {
    Ok(query(
        client,
        source_id,
        "transforms",
        r#"
SELECT pg_catalog.format_type(transform.trftype, NULL), language.lanname, extension.extname,
       CASE WHEN transform.trffromsql = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, transform.trffromsql, 0)).identity END,
       CASE WHEN transform.trftosql = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, transform.trftosql, 0)).identity END,
       pg_catalog.obj_description(transform.oid, 'pg_transform')
FROM pg_catalog.pg_transform AS transform
JOIN pg_catalog.pg_language AS language ON language.oid = transform.trflang
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_transform'::pg_catalog.regclass
 AND dependency.objid = transform.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
 AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE transform.oid >= 16384
  AND NOT EXISTS (
      SELECT 1 FROM pg_catalog.pg_depend AS internal_dependency
      WHERE internal_dependency.classid = 'pg_catalog.pg_transform'::pg_catalog.regclass
        AND internal_dependency.objid = transform.oid
        AND internal_dependency.objsubid = 0
        AND internal_dependency.deptype = 'i'
  )
ORDER BY pg_catalog.format_type(transform.trftype, NULL) COLLATE "C", language.lanname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| Transform {
        data_type: row.get(0), language: row.get(1), extension: row.get(2),
        from_sql: row.get(3), to_sql: row.get(4), comment: row.get(5),
    })
    .collect())
}

fn load_rules(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<RewriteRule>, IntrospectionError> {
    query(
        client,
        source_id,
        "rewrite rules",
        r#"
SELECT namespace.nspname, rule.rulename, relation.relname,
       rule.ev_type::text, rule.is_instead, rule.ev_enabled::text,
       pg_catalog.pg_get_ruledef(rule.oid, true),
       pg_catalog.obj_description(rule.oid, 'pg_rewrite'),
       extension.extname
FROM pg_catalog.pg_rewrite AS rule
JOIN pg_catalog.pg_class AS relation ON relation.oid = rule.ev_class
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_rewrite'::pg_catalog.regclass
 AND dependency.objid = rule.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
 AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE rule.rulename <> '_RETURN'
  AND namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C", relation.relname COLLATE "C",
         rule.rulename COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        Ok(RewriteRule {
            namespace: row.get(0),
            name: row.get(1),
            target: row.get(2),
            event: rewrite_rule_event(source_id, &row.get::<_, String>(3))?,
            instead: row.get(4),
            enabled: trigger_enabled(source_id, &row.get::<_, String>(5))?,
            definition: row.get(6),
            comment: row.get(7),
            extension: row.get(8),
        })
    })
    .collect()
}

fn rewrite_rule_event(
    source_id: &SourceId,
    value: &str,
) -> Result<RewriteRuleEvent, IntrospectionError> {
    match value {
        "1" => Ok(RewriteRuleEvent::Select),
        "2" => Ok(RewriteRuleEvent::Update),
        "3" => Ok(RewriteRuleEvent::Insert),
        "4" => Ok(RewriteRuleEvent::Delete),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_rewrite.ev_type",
            other,
        )),
    }
}

fn load_event_triggers(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<EventTrigger>, IntrospectionError> {
    query(
        client,
        source_id,
        "event triggers",
        r#"
SELECT trigger_record.evtname, owner.rolname, trigger_record.evtevent,
       COALESCE(trigger_record.evttags, ARRAY[]::text[]),
       (pg_catalog.pg_identify_object(
           'pg_catalog.pg_proc'::pg_catalog.regclass,
           trigger_record.evtfoid,
           0
       )).identity,
       trigger_record.evtenabled::text,
       pg_catalog.obj_description(trigger_record.oid, 'pg_event_trigger'),
       extension.extname
FROM pg_catalog.pg_event_trigger AS trigger_record
JOIN pg_catalog.pg_roles AS owner ON owner.oid = trigger_record.evtowner
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_event_trigger'::pg_catalog.regclass
 AND dependency.objid = trigger_record.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
 AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
ORDER BY trigger_record.evtname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        let name: String = row.get(0);
        let native_event: String = row.get(2);
        let event = event_trigger_event(source_id, &native_event)?;
        let tags: Vec<String> = row.get(3);
        let function: String = row.get(4);
        let mut definition = format!(
            "CREATE EVENT TRIGGER {} ON {}",
            quote_postgres_identifier(&name),
            native_event
        );
        if !tags.is_empty() {
            let tags = tags
                .iter()
                .map(|tag| format!("'{}'", tag.replace('\'', "''")))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = fmt::Write::write_fmt(&mut definition, format_args!(" WHEN TAG IN ({tags})"));
        }
        let _ = fmt::Write::write_fmt(
            &mut definition,
            format_args!(" EXECUTE FUNCTION {function};"),
        );
        Ok(EventTrigger {
            name,
            owner: row.get(1),
            event,
            tags,
            function,
            enabled: trigger_enabled(source_id, &row.get::<_, String>(5))?,
            comment: row.get(6),
            extension: row.get(7),
            definition,
        })
    })
    .collect()
}

fn event_trigger_event(
    source_id: &SourceId,
    value: &str,
) -> Result<EventTriggerEvent, IntrospectionError> {
    match value {
        "login" => Ok(EventTriggerEvent::Login),
        "ddl_command_start" => Ok(EventTriggerEvent::DdlCommandStart),
        "ddl_command_end" => Ok(EventTriggerEvent::DdlCommandEnd),
        "sql_drop" => Ok(EventTriggerEvent::SqlDrop),
        "table_rewrite" => Ok(EventTriggerEvent::TableRewrite),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_event_trigger.evtevent",
            other,
        )),
    }
}

fn load_extended_statistics(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<ExtendedStatistics>, IntrospectionError> {
    query(
        client,
        source_id,
        "extended statistics",
        r#"
SELECT namespace.nspname, statistics.stxname, owner.rolname,
       pg_catalog.array_to_string(statistics.stxkind, ''),
       ARRAY(
           SELECT attribute.attname
           FROM unnest(statistics.stxkeys::smallint[]) WITH ORDINALITY AS key(attnum, position)
           JOIN pg_catalog.pg_attribute AS attribute
             ON attribute.attrelid = statistics.stxrelid AND attribute.attnum = key.attnum
           ORDER BY key.position
       ),
       COALESCE(pg_catalog.pg_get_statisticsobjdef_expressions(statistics.oid), ARRAY[]::text[]),
       COALESCE(statistics.stxstattarget, -1)::integer,
       pg_catalog.pg_get_statisticsobjdef(statistics.oid),
       pg_catalog.obj_description(statistics.oid, 'pg_statistic_ext'),
       extension.extname
FROM pg_catalog.pg_statistic_ext AS statistics
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = statistics.stxnamespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = statistics.stxowner
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_statistic_ext'::pg_catalog.regclass
 AND dependency.objid = statistics.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
 AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE namespace.nspname <> 'information_schema' AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C", statistics.stxname COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        Ok(ExtendedStatistics {
            namespace: row.get(0),
            name: row.get(1),
            owner: row.get(2),
            kinds: row
                .get::<_, String>(3)
                .chars()
                .map(|kind| statistics_kind(source_id, kind))
                .collect::<Result<Vec<_>, _>>()?,
            columns: row.get(4),
            expressions: row.get(5),
            target: row.get(6),
            definition: row.get(7),
            comment: row.get(8),
            extension: row.get(9),
        })
    })
    .collect()
}

fn statistics_kind(
    source_id: &SourceId,
    value: char,
) -> Result<StatisticsKind, IntrospectionError> {
    match value {
        'd' => Ok(StatisticsKind::NdDistinct),
        'f' => Ok(StatisticsKind::Dependencies),
        'm' => Ok(StatisticsKind::MostCommonValues),
        'e' => Ok(StatisticsKind::Expressions),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_statistic_ext.stxkind",
            &other.to_string(),
        )),
    }
}

fn load_foreign_data_wrappers(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<ForeignDataWrapper>, IntrospectionError> {
    Ok(query(client, source_id, "foreign-data wrappers", r#"
SELECT wrapper.fdwname, owner.rolname,
       CASE WHEN wrapper.fdwhandler = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, wrapper.fdwhandler, 0)).identity END,
       CASE WHEN wrapper.fdwvalidator = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, wrapper.fdwvalidator, 0)).identity END,
       ARRAY(
           SELECT CASE
                    WHEN pg_catalog.split_part(option, '=', 1) ~* '(password|secret|token|credential|private[_-]?key)'
                    THEN pg_catalog.split_part(option, '=', 1) || '=<redacted>'
                    ELSE option
                  END
           FROM unnest(COALESCE(wrapper.fdwoptions, ARRAY[]::text[])) WITH ORDINALITY AS item(option, position)
           ORDER BY position
       ),
       pg_catalog.obj_description(wrapper.oid, 'pg_foreign_data_wrapper'),
       extension.extname
FROM pg_catalog.pg_foreign_data_wrapper AS wrapper
JOIN pg_catalog.pg_roles AS owner ON owner.oid = wrapper.fdwowner
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_foreign_data_wrapper'::pg_catalog.regclass
 AND dependency.objid = wrapper.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
 AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE wrapper.oid >= 16384
ORDER BY wrapper.fdwname COLLATE "C"
"#)?
        .into_iter()
        .map(|row| ForeignDataWrapper {
            name: row.get(0), owner: row.get(1), handler: row.get(2), validator: row.get(3),
            options: row.get(4), comment: row.get(5), extension: row.get(6),
        })
        .collect())
}

fn load_foreign_servers(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<ForeignServer>, IntrospectionError> {
    Ok(query(client, source_id, "foreign servers", r#"
SELECT server.srvname, owner.rolname, wrapper.fdwname, server.srvtype,
       server.srvversion,
       ARRAY(
           SELECT CASE
                    WHEN pg_catalog.split_part(option, '=', 1) ~* '(password|secret|token|credential|private[_-]?key)'
                    THEN pg_catalog.split_part(option, '=', 1) || '=<redacted>'
                    ELSE option
                  END
           FROM unnest(COALESCE(server.srvoptions, ARRAY[]::text[])) WITH ORDINALITY AS item(option, position)
           ORDER BY position
       ),
       pg_catalog.obj_description(server.oid, 'pg_foreign_server'), extension.extname
FROM pg_catalog.pg_foreign_server AS server
JOIN pg_catalog.pg_roles AS owner ON owner.oid = server.srvowner
JOIN pg_catalog.pg_foreign_data_wrapper AS wrapper ON wrapper.oid = server.srvfdw
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_foreign_server'::pg_catalog.regclass
 AND dependency.objid = server.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
 AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE server.oid >= 16384
ORDER BY server.srvname COLLATE "C"
"#)?
        .into_iter()
        .map(|row| ForeignServer {
            name: row.get(0), owner: row.get(1), wrapper: row.get(2), server_type: row.get(3),
            version: row.get(4), options: row.get(5), comment: row.get(6), extension: row.get(7),
        })
        .collect())
}

fn load_user_mappings(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<UserMapping>, IntrospectionError> {
    Ok(query(client, source_id, "foreign user mappings", r#"
SELECT server.srvname,
       CASE WHEN mapping.umuser = 0 THEN 'PUBLIC' ELSE role.rolname END,
       ARRAY(
           SELECT CASE
                    WHEN pg_catalog.split_part(option, '=', 1) ~* '(password|secret|token|credential|private[_-]?key)'
                    THEN pg_catalog.split_part(option, '=', 1) || '=<redacted>'
                    ELSE option
                  END
           FROM unnest(COALESCE(mapping.umoptions, ARRAY[]::text[])) WITH ORDINALITY AS item(option, position)
           ORDER BY position
       )
FROM pg_catalog.pg_user_mapping AS mapping
JOIN pg_catalog.pg_foreign_server AS server ON server.oid = mapping.umserver
LEFT JOIN pg_catalog.pg_roles AS role ON role.oid = NULLIF(mapping.umuser, 0)
WHERE server.oid >= 16384
ORDER BY server.srvname COLLATE "C",
         CASE WHEN mapping.umuser = 0 THEN 'PUBLIC' ELSE role.rolname END COLLATE "C"
"#)?
        .into_iter()
        .map(|row| UserMapping { server: row.get(0), user: row.get(1), options: row.get(2) })
        .collect())
}

fn load_text_search_parsers(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<TextSearchParser>, IntrospectionError> {
    Ok(query(client, source_id, "text-search parsers", r#"
SELECT namespace.nspname, parser.prsname,
       (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, parser.prsstart, 0)).identity,
       (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, parser.prstoken, 0)).identity,
       (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, parser.prsend, 0)).identity,
       (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, parser.prsheadline, 0)).identity,
       (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, parser.prslextype, 0)).identity,
       pg_catalog.obj_description(parser.oid, 'pg_ts_parser'), extension.extname
FROM pg_catalog.pg_ts_parser AS parser
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = parser.prsnamespace
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_ts_parser'::pg_catalog.regclass
 AND dependency.objid = parser.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
 AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE namespace.nspname <> 'information_schema' AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C", parser.prsname COLLATE "C"
"#)?
        .into_iter()
        .map(|row| TextSearchParser {
            namespace: row.get(0), name: row.get(1), start_function: row.get(2),
            token_function: row.get(3), end_function: row.get(4), headline_function: row.get(5),
            token_types_function: row.get(6), comment: row.get(7), extension: row.get(8),
        })
        .collect())
}

fn load_text_search_templates(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<TextSearchTemplate>, IntrospectionError> {
    Ok(query(client, source_id, "text-search templates", r#"
SELECT namespace.nspname, template.tmplname,
       CASE WHEN template.tmplinit = 0 THEN NULL ELSE
           (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, template.tmplinit, 0)).identity END,
       (pg_catalog.pg_identify_object('pg_catalog.pg_proc'::pg_catalog.regclass, template.tmpllexize, 0)).identity,
       pg_catalog.obj_description(template.oid, 'pg_ts_template'), extension.extname
FROM pg_catalog.pg_ts_template AS template
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = template.tmplnamespace
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_ts_template'::pg_catalog.regclass
 AND dependency.objid = template.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
 AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE namespace.nspname <> 'information_schema' AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C", template.tmplname COLLATE "C"
"#)?
        .into_iter()
        .map(|row| TextSearchTemplate {
            namespace: row.get(0), name: row.get(1), init_function: row.get(2),
            lexize_function: row.get(3), comment: row.get(4), extension: row.get(5),
        })
        .collect())
}

fn load_text_search_dictionaries(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<TextSearchDictionary>, IntrospectionError> {
    Ok(query(client, source_id, "text-search dictionaries", r#"
SELECT namespace.nspname, dictionary.dictname, owner.rolname,
       pg_catalog.format('%I.%I', template_namespace.nspname, template.tmplname),
       dictionary.dictinitoption,
       pg_catalog.obj_description(dictionary.oid, 'pg_ts_dict'), extension.extname
FROM pg_catalog.pg_ts_dict AS dictionary
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = dictionary.dictnamespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = dictionary.dictowner
JOIN pg_catalog.pg_ts_template AS template ON template.oid = dictionary.dicttemplate
JOIN pg_catalog.pg_namespace AS template_namespace ON template_namespace.oid = template.tmplnamespace
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_ts_dict'::pg_catalog.regclass
 AND dependency.objid = dictionary.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
 AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE namespace.nspname <> 'information_schema' AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C", dictionary.dictname COLLATE "C"
"#)?
        .into_iter()
        .map(|row| TextSearchDictionary {
            namespace: row.get(0), name: row.get(1), owner: row.get(2), template: row.get(3),
            options: row.get(4), comment: row.get(5), extension: row.get(6),
        })
        .collect())
}

fn load_text_search_configurations(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<TextSearchConfiguration>, IntrospectionError> {
    let rows = query(
        client,
        source_id,
        "text-search configurations",
        r#"
SELECT configuration.oid::bigint, namespace.nspname, configuration.cfgname,
       owner.rolname, pg_catalog.format('%I.%I', parser_namespace.nspname, parser.prsname),
       pg_catalog.obj_description(configuration.oid, 'pg_ts_config'), extension.extname
FROM pg_catalog.pg_ts_config AS configuration
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = configuration.cfgnamespace
JOIN pg_catalog.pg_roles AS owner ON owner.oid = configuration.cfgowner
JOIN pg_catalog.pg_ts_parser AS parser ON parser.oid = configuration.cfgparser
JOIN pg_catalog.pg_namespace AS parser_namespace ON parser_namespace.oid = parser.prsnamespace
LEFT JOIN pg_catalog.pg_depend AS dependency
  ON dependency.classid = 'pg_catalog.pg_ts_config'::pg_catalog.regclass
 AND dependency.objid = configuration.oid AND dependency.objsubid = 0
 AND dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
 AND dependency.deptype = 'e'
LEFT JOIN pg_catalog.pg_extension AS extension ON extension.oid = dependency.refobjid
WHERE namespace.nspname <> 'information_schema' AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C", configuration.cfgname COLLATE "C"
"#,
    )?;
    let mut configurations = Vec::with_capacity(rows.len());
    let mut configuration_by_oid = HashMap::with_capacity(rows.len());
    for row in rows {
        configuration_by_oid.insert(row.get::<_, i64>(0), configurations.len());
        configurations.push(TextSearchConfiguration {
            namespace: row.get(1),
            name: row.get(2),
            owner: row.get(3),
            parser: row.get(4),
            mappings: Vec::new(),
            comment: row.get(5),
            extension: row.get(6),
        });
    }
    for row in query(
        client,
        source_id,
        "text-search configuration mappings",
        r#"
SELECT mapping.mapcfg::bigint, token.alias,
       ARRAY(
           SELECT pg_catalog.format('%I.%I', dictionary_namespace.nspname, dictionary.dictname)
           FROM pg_catalog.pg_ts_config_map AS ordered_mapping
           JOIN pg_catalog.pg_ts_dict AS dictionary ON dictionary.oid = ordered_mapping.mapdict
           JOIN pg_catalog.pg_namespace AS dictionary_namespace ON dictionary_namespace.oid = dictionary.dictnamespace
           WHERE ordered_mapping.mapcfg = mapping.mapcfg
             AND ordered_mapping.maptokentype = mapping.maptokentype
           ORDER BY ordered_mapping.mapseqno
       )
FROM pg_catalog.pg_ts_config_map AS mapping
JOIN pg_catalog.pg_ts_config AS configuration ON configuration.oid = mapping.mapcfg
JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = configuration.cfgnamespace
JOIN LATERAL pg_catalog.ts_token_type(configuration.cfgparser) AS token
  ON token.tokid = mapping.maptokentype
WHERE namespace.nspname <> 'information_schema' AND namespace.nspname !~ '^pg_'
GROUP BY mapping.mapcfg, mapping.maptokentype, token.alias
ORDER BY mapping.mapcfg, mapping.maptokentype
"#,
    )? {
        if let Some(&index) = configuration_by_oid.get(&row.get::<_, i64>(0)) {
            configurations[index].mappings.push(TextSearchMapping {
                token_type: row.get(1),
                dictionaries: row.get(2),
            });
        }
    }
    Ok(configurations)
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
       constraint_record.conenforced,
       pg_catalog.pg_get_constraintdef(constraint_record.oid, true),
       constraint_record.convalidated,
       constraint_record.conislocal,
       constraint_record.connoinherit,
       constraint_record.conperiod,
       pg_catalog.obj_description(constraint_record.oid, 'pg_constraint'),
       COALESCE(ARRAY(
           SELECT (pg_catalog.pg_identify_object(
               'pg_catalog.pg_operator'::pg_catalog.regclass,
               exclusion_operator,
               0
           )).identity
           FROM unnest(constraint_record.conexclop) WITH ORDINALITY
                AS operator(exclusion_operator, position)
           ORDER BY position
       ), ARRAY[]::text[])
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
  AND constraint_record.contype IN ('c', 'f', 'n', 'p', 'u', 'x')
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
            Some(
                ForeignKeyReference::new(
                    row.get::<_, Option<String>>(5).unwrap_or_default(),
                    row.get::<_, Option<String>>(6).unwrap_or_default(),
                    row.get(7),
                )
                .with_actions(
                    foreign_key_action(source_id, &row.get::<_, String>(8))?,
                    foreign_key_action(source_id, &row.get::<_, String>(9))?,
                )
                .with_match_type(Some(foreign_key_match(
                    source_id,
                    &row.get::<_, String>(10),
                )?))
                .with_deferrability(ForeignKeyDeferrability::new(
                    row.get(11),
                    if row.get(12) {
                        ForeignKeyInitialTiming::Deferred
                    } else {
                        ForeignKeyInitialTiming::Immediate
                    },
                )),
            )
        } else {
            None
        };
        tables[table_index].constraints.push(Constraint {
            name: Some(row.get(1)),
            kind,
            columns: row.get(3),
            expression: row.get(4),
            references,
            definition: row.get(14),
            exclusion_operators: row.get(20),
            deferrable: row.get(11),
            initially_deferred: row.get(12),
            enforced: row.get(13),
            validated: row.get(15),
            locally_defined: row.get(16),
            no_inherit: row.get(17),
            temporal: row.get(18),
            comment: row.get(19),
        });
    }
    Ok(())
}

fn load_indexes(
    client: &mut Client,
    source_id: &SourceId,
    tables: &mut [Table],
    views: &mut [View],
) -> Result<(), IntrospectionError> {
    let table_by_name = tables
        .iter()
        .enumerate()
        .map(|(index, table)| ((table.namespace.clone(), table.name.clone()), index))
        .collect::<HashMap<_, _>>();
    let view_by_name = views
        .iter()
        .enumerate()
        .map(|(index, view)| ((view.namespace.clone(), view.name.clone()), index))
        .collect::<HashMap<_, _>>();
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
       index_record.indisreplident,
       (
           SELECT owning_extension.extname
           FROM pg_catalog.pg_depend AS extension_dependency
           JOIN pg_catalog.pg_extension AS owning_extension
             ON owning_extension.oid = extension_dependency.refobjid
           WHERE extension_dependency.classid = 'pg_catalog.pg_class'::pg_catalog.regclass
             AND extension_dependency.objid = index_relation.oid
             AND extension_dependency.objsubid = 0
             AND extension_dependency.refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass
             AND extension_dependency.deptype = 'e'
       ),
       owner.rolname, tablespace.spcname,
       COALESCE(index_relation.reloptions, ARRAY[]::text[]),
       index_relation.relkind = 'I',
       (
           SELECT pg_catalog.format('%I.%I', parent_namespace.nspname, parent_index.relname)
           FROM pg_catalog.pg_inherits AS inheritance
           JOIN pg_catalog.pg_class AS parent_index ON parent_index.oid = inheritance.inhparent
           JOIN pg_catalog.pg_namespace AS parent_namespace ON parent_namespace.oid = parent_index.relnamespace
           WHERE inheritance.inhrelid = index_relation.oid
           ORDER BY inheritance.inhseqno
           LIMIT 1
       ),
       (
           SELECT constraint_record.conname
           FROM pg_catalog.pg_constraint AS constraint_record
           WHERE constraint_record.conindid = index_relation.oid
           ORDER BY constraint_record.conname COLLATE "C"
           LIMIT 1
       ),
       pg_catalog.obj_description(index_relation.oid, 'pg_class'),
       namespace.nspname,
       table_relation.relname,
       ARRAY(
           SELECT COALESCE(
               pg_catalog.array_to_string(index_attribute.attoptions, pg_catalog.chr(31)),
               ''
           )
           FROM generate_series(1, index_record.indnkeyatts) AS position
           JOIN pg_catalog.pg_attribute AS index_attribute
             ON index_attribute.attrelid = index_record.indexrelid
            AND index_attribute.attnum = position
           ORDER BY position
       )
FROM pg_catalog.pg_index AS index_record
JOIN pg_catalog.pg_class AS table_relation
  ON table_relation.oid = index_record.indrelid
JOIN pg_catalog.pg_namespace AS namespace
  ON namespace.oid = table_relation.relnamespace
JOIN pg_catalog.pg_class AS index_relation
  ON index_relation.oid = index_record.indexrelid
JOIN pg_catalog.pg_am AS access_method
  ON access_method.oid = index_relation.relam
JOIN pg_catalog.pg_roles AS owner ON owner.oid = index_relation.relowner
LEFT JOIN pg_catalog.pg_tablespace AS tablespace
  ON tablespace.oid = NULLIF(index_relation.reltablespace, 0)
WHERE table_relation.relkind IN ('r', 'p', 'f', 'm')
  AND namespace.nspname <> 'information_schema'
  AND namespace.nspname !~ '^pg_'
ORDER BY namespace.nspname COLLATE "C",
         table_relation.relname COLLATE "C",
         index_relation.relname COLLATE "C"
"#,
    )?;

    for row in rows {
        let relation_key = (row.get::<_, String>(26), row.get::<_, String>(27));
        let table_index = table_by_name.get(&relation_key).copied();
        let view_index = view_by_name.get(&relation_key).copied();
        let parent_extension = table_index
            .and_then(|index| tables[index].extension.clone())
            .or_else(|| view_index.and_then(|index| views[index].extension.clone()));
        if table_index.is_none() && view_index.is_none() {
            continue;
        }
        let targets = row.get::<_, Vec<String>>(6);
        let column_flags = row.get::<_, Vec<bool>>(7);
        let descending = row.get::<_, Vec<bool>>(8);
        let collations = row.get::<_, Vec<Option<String>>>(9);
        let operator_classes = row.get::<_, Vec<Option<String>>>(11);
        let nulls_first = row.get::<_, Vec<bool>>(12);
        let operator_class_parameters =
            row.get::<_, Vec<String>>(28).into_iter().map(|parameters| {
                if parameters.is_empty() {
                    Vec::new()
                } else {
                    parameters.split('\u{1f}').map(str::to_owned).collect()
                }
            });
        let method = row.get::<_, String>(3);
        let terms = targets
            .into_iter()
            .zip(column_flags)
            .zip(descending)
            .zip(collations)
            .zip(operator_classes)
            .zip(nulls_first)
            .zip(operator_class_parameters)
            .map(
                |(
                    (((((target, is_column), descending), collation), operator_class), nulls_first),
                    operator_class_parameters,
                )| {
                    IndexTerm {
                        target: if is_column {
                            IndexTarget::Column(target)
                        } else {
                            IndexTarget::Expression(target)
                        },
                        collation,
                        operator_class,
                        operator_class_parameters,
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
        let extension = row.get::<_, Option<String>>(18).or(parent_extension);
        let index = Index {
            name: row.get(1),
            extension,
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
            owner: row.get(19),
            tablespace: row.get(20),
            options: row.get(21),
            partitioned: row.get(22),
            parent_index: row.get(23),
            constraint: row.get(24),
            comment: row.get(25),
        };
        if let Some(table_index) = table_index {
            tables[table_index].indexes.push(index);
        } else if let Some(view_index) = view_index {
            views[view_index].indexes.push(index);
        }
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
       pg_catalog.pg_get_expr(policy.polwithcheck, policy.polrelid, true),
       pg_catalog.obj_description(policy.oid, 'pg_policy')
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
            comment: row.get(7),
        });
    }
    Ok(())
}

fn load_object_privileges(
    client: &mut Client,
    source_id: &SourceId,
    include_cluster_objects: bool,
) -> Result<Vec<ObjectPrivilege>, IntrospectionError> {
    let mut privileges = object_privileges_from_rows(
        source_id,
        query(
            client,
            source_id,
            "object privileges",
            r#"
WITH acl_object AS (
    SELECT 'pg_catalog.pg_database'::pg_catalog.regclass::oid AS classid,
           database.oid AS objid, 0::integer AS objsubid, database.datacl AS acl
    FROM pg_catalog.pg_database AS database
    WHERE database.datname = pg_catalog.current_database()
    UNION ALL
    SELECT 'pg_catalog.pg_namespace'::pg_catalog.regclass::oid,
           namespace.oid, 0, namespace.nspacl
    FROM pg_catalog.pg_namespace AS namespace
    WHERE namespace.nspname <> 'information_schema'
      AND namespace.nspname !~ '^pg_'
    UNION ALL
    SELECT 'pg_catalog.pg_class'::pg_catalog.regclass::oid,
           relation.oid, 0, relation.relacl
    FROM pg_catalog.pg_class AS relation
    JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace
    WHERE relation.relkind IN ('r', 'p', 'v', 'm', 'S', 'f')
      AND namespace.nspname <> 'information_schema'
      AND namespace.nspname !~ '^pg_'
    UNION ALL
    SELECT 'pg_catalog.pg_class'::pg_catalog.regclass::oid,
           relation.oid, attribute.attnum::integer, attribute.attacl
    FROM pg_catalog.pg_attribute AS attribute
    JOIN pg_catalog.pg_class AS relation ON relation.oid = attribute.attrelid
    JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace
    WHERE attribute.attnum > 0
      AND NOT attribute.attisdropped
      AND namespace.nspname <> 'information_schema'
      AND namespace.nspname !~ '^pg_'
    UNION ALL
    SELECT 'pg_catalog.pg_proc'::pg_catalog.regclass::oid,
           routine.oid, 0, routine.proacl
    FROM pg_catalog.pg_proc AS routine
    JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = routine.pronamespace
    WHERE namespace.nspname <> 'information_schema'
      AND namespace.nspname !~ '^pg_'
    UNION ALL
    SELECT 'pg_catalog.pg_type'::pg_catalog.regclass::oid,
           type_record.oid, 0, type_record.typacl
    FROM pg_catalog.pg_type AS type_record
    JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = type_record.typnamespace
    WHERE type_record.typtype IN ('b', 'c', 'd', 'e', 'r', 'm')
      AND namespace.nspname <> 'information_schema'
      AND namespace.nspname !~ '^pg_'
    UNION ALL
    SELECT 'pg_catalog.pg_language'::pg_catalog.regclass::oid,
           language.oid, 0, language.lanacl
    FROM pg_catalog.pg_language AS language
    WHERE language.lanispl
    UNION ALL
    SELECT 'pg_catalog.pg_largeobject'::pg_catalog.regclass::oid,
           metadata.oid, 0, metadata.lomacl
    FROM pg_catalog.pg_largeobject_metadata AS metadata
    UNION ALL
    SELECT 'pg_catalog.pg_foreign_data_wrapper'::pg_catalog.regclass::oid,
           wrapper.oid, 0, wrapper.fdwacl
    FROM pg_catalog.pg_foreign_data_wrapper AS wrapper
    UNION ALL
    SELECT 'pg_catalog.pg_foreign_server'::pg_catalog.regclass::oid,
           server.oid, 0, server.srvacl
    FROM pg_catalog.pg_foreign_server AS server
    UNION ALL
    SELECT 'pg_catalog.pg_parameter_acl'::pg_catalog.regclass::oid,
           parameter_acl.oid, 0, parameter_acl.paracl
    FROM pg_catalog.pg_parameter_acl AS parameter_acl
)
SELECT identified.type,
       identified.identity,
       pg_catalog.pg_get_userbyid(privilege.grantor),
       CASE privilege.grantee
           WHEN 0 THEN 'PUBLIC'
           ELSE pg_catalog.pg_get_userbyid(privilege.grantee)
       END,
       privilege.privilege_type,
       privilege.is_grantable
FROM acl_object
CROSS JOIN LATERAL pg_catalog.aclexplode(acl_object.acl) AS privilege
CROSS JOIN LATERAL pg_catalog.pg_identify_object(
    acl_object.classid, acl_object.objid, acl_object.objsubid
) AS identified
ORDER BY identified.type COLLATE "C", identified.identity COLLATE "C",
         CASE privilege.grantee WHEN 0 THEN 'PUBLIC'
              ELSE pg_catalog.pg_get_userbyid(privilege.grantee) END COLLATE "C",
         privilege.privilege_type COLLATE "C", privilege.is_grantable,
         pg_catalog.pg_get_userbyid(privilege.grantor) COLLATE "C"
"#,
        )?,
    )?;

    if include_cluster_objects {
        privileges.extend(object_privileges_from_rows(
            source_id,
            query(
                client,
                source_id,
                "shared object privileges",
                r#"
WITH acl_object AS (
    SELECT 'pg_catalog.pg_database'::pg_catalog.regclass::oid AS classid,
           database.oid AS objid, 0::integer AS objsubid, database.datacl AS acl
    FROM pg_catalog.pg_database AS database
    WHERE database.datname <> pg_catalog.current_database()
    UNION ALL
    SELECT 'pg_catalog.pg_tablespace'::pg_catalog.regclass::oid,
           tablespace.oid, 0, tablespace.spcacl
    FROM pg_catalog.pg_tablespace AS tablespace
)
SELECT identified.type,
       identified.identity,
       pg_catalog.pg_get_userbyid(privilege.grantor),
       CASE privilege.grantee
           WHEN 0 THEN 'PUBLIC'
           ELSE pg_catalog.pg_get_userbyid(privilege.grantee)
       END,
       privilege.privilege_type,
       privilege.is_grantable
FROM acl_object
CROSS JOIN LATERAL pg_catalog.aclexplode(acl_object.acl) AS privilege
CROSS JOIN LATERAL pg_catalog.pg_identify_object(
    acl_object.classid, acl_object.objid, acl_object.objsubid
) AS identified
ORDER BY identified.type COLLATE "C", identified.identity COLLATE "C",
         CASE privilege.grantee WHEN 0 THEN 'PUBLIC'
              ELSE pg_catalog.pg_get_userbyid(privilege.grantee) END COLLATE "C",
         privilege.privilege_type COLLATE "C", privilege.is_grantable,
         pg_catalog.pg_get_userbyid(privilege.grantor) COLLATE "C"
"#,
            )?,
        )?);
        privileges.sort_by(|left, right| {
            (
                left.object_kind,
                &left.object_identity,
                &left.grantee,
                &left.privilege,
                left.grantable,
                &left.grantor,
            )
                .cmp(&(
                    right.object_kind,
                    &right.object_identity,
                    &right.grantee,
                    &right.privilege,
                    right.grantable,
                    &right.grantor,
                ))
        });
    }
    Ok(privileges)
}

fn object_privileges_from_rows(
    source_id: &SourceId,
    rows: Vec<Row>,
) -> Result<Vec<ObjectPrivilege>, IntrospectionError> {
    rows.into_iter()
        .map(|row| {
            Ok(ObjectPrivilege {
                object_kind: privilege_object_kind(source_id, &row.get::<_, String>(0))?,
                object_identity: row.get(1),
                grantor: row.get(2),
                grantee: row.get(3),
                privilege: privilege_kind(source_id, &row.get::<_, String>(4))?,
                grantable: row.get(5),
            })
        })
        .collect()
}

fn privilege_object_kind(
    source_id: &SourceId,
    value: &str,
) -> Result<PrivilegeObjectKind, IntrospectionError> {
    match value {
        "database" => Ok(PrivilegeObjectKind::Database),
        "schema" => Ok(PrivilegeObjectKind::Schema),
        "table" => Ok(PrivilegeObjectKind::Table),
        "table column" => Ok(PrivilegeObjectKind::TableColumn),
        "sequence" => Ok(PrivilegeObjectKind::Sequence),
        "view" => Ok(PrivilegeObjectKind::View),
        "materialized view" => Ok(PrivilegeObjectKind::MaterializedView),
        "foreign table" => Ok(PrivilegeObjectKind::ForeignTable),
        "function" => Ok(PrivilegeObjectKind::Function),
        "procedure" => Ok(PrivilegeObjectKind::Procedure),
        "aggregate" => Ok(PrivilegeObjectKind::Aggregate),
        "type" => Ok(PrivilegeObjectKind::Type),
        "domain" => Ok(PrivilegeObjectKind::Domain),
        "language" => Ok(PrivilegeObjectKind::Language),
        "large object" => Ok(PrivilegeObjectKind::LargeObject),
        "foreign-data wrapper" => Ok(PrivilegeObjectKind::ForeignDataWrapper),
        "server" => Ok(PrivilegeObjectKind::ForeignServer),
        "parameter ACL" => Ok(PrivilegeObjectKind::Parameter),
        "tablespace" => Ok(PrivilegeObjectKind::Tablespace),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_identify_object privilege type",
            other,
        )),
    }
}

fn privilege_kind(source_id: &SourceId, value: &str) -> Result<PrivilegeKind, IntrospectionError> {
    match value {
        "SELECT" => Ok(PrivilegeKind::Select),
        "INSERT" => Ok(PrivilegeKind::Insert),
        "UPDATE" => Ok(PrivilegeKind::Update),
        "DELETE" => Ok(PrivilegeKind::Delete),
        "TRUNCATE" => Ok(PrivilegeKind::Truncate),
        "REFERENCES" => Ok(PrivilegeKind::References),
        "TRIGGER" => Ok(PrivilegeKind::Trigger),
        "MAINTAIN" => Ok(PrivilegeKind::Maintain),
        "USAGE" => Ok(PrivilegeKind::Usage),
        "CREATE" => Ok(PrivilegeKind::Create),
        "CONNECT" => Ok(PrivilegeKind::Connect),
        "TEMPORARY" => Ok(PrivilegeKind::Temporary),
        "EXECUTE" => Ok(PrivilegeKind::Execute),
        "SET" => Ok(PrivilegeKind::Set),
        "ALTER SYSTEM" => Ok(PrivilegeKind::AlterSystem),
        other => Err(unsupported_catalog_value(
            source_id,
            "aclexplode.privilege_type",
            other,
        )),
    }
}

fn load_default_privileges(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<DefaultPrivilege>, IntrospectionError> {
    query(
        client,
        source_id,
        "default privileges",
        r#"
SELECT owner.rolname,
       namespace.nspname,
       default_acl.defaclobjtype::text,
       pg_catalog.pg_get_userbyid(privilege.grantor),
       CASE privilege.grantee
           WHEN 0 THEN 'PUBLIC'
           ELSE pg_catalog.pg_get_userbyid(privilege.grantee)
       END,
       privilege.privilege_type,
       privilege.is_grantable
FROM pg_catalog.pg_default_acl AS default_acl
JOIN pg_catalog.pg_roles AS owner ON owner.oid = default_acl.defaclrole
LEFT JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = default_acl.defaclnamespace
CROSS JOIN LATERAL pg_catalog.aclexplode(default_acl.defaclacl) AS privilege
WHERE namespace.oid IS NULL
   OR (namespace.nspname <> 'information_schema' AND namespace.nspname !~ '^pg_')
ORDER BY owner.rolname COLLATE "C", namespace.nspname COLLATE "C",
         default_acl.defaclobjtype::text COLLATE "C",
         CASE privilege.grantee WHEN 0 THEN 'PUBLIC'
              ELSE pg_catalog.pg_get_userbyid(privilege.grantee) END COLLATE "C",
         privilege.privilege_type COLLATE "C", privilege.is_grantable,
         pg_catalog.pg_get_userbyid(privilege.grantor) COLLATE "C"
"#,
    )?
    .into_iter()
    .map(|row| {
        Ok(DefaultPrivilege {
            owner: row.get(0),
            namespace: row.get(1),
            object_kind: default_privilege_object(source_id, &row.get::<_, String>(2))?,
            grantor: row.get(3),
            grantee: row.get(4),
            privilege: privilege_kind(source_id, &row.get::<_, String>(5))?,
            grantable: row.get(6),
        })
    })
    .collect()
}

fn default_privilege_object(
    source_id: &SourceId,
    code: &str,
) -> Result<DefaultPrivilegeObject, IntrospectionError> {
    match code {
        "r" => Ok(DefaultPrivilegeObject::Tables),
        "S" => Ok(DefaultPrivilegeObject::Sequences),
        "f" => Ok(DefaultPrivilegeObject::Routines),
        "T" => Ok(DefaultPrivilegeObject::Types),
        "n" => Ok(DefaultPrivilegeObject::Schemas),
        "L" => Ok(DefaultPrivilegeObject::LargeObjects),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_default_acl.defaclobjtype",
            other,
        )),
    }
}

fn load_security_labels(
    client: &mut Client,
    source_id: &SourceId,
    include_cluster_objects: bool,
) -> Result<Vec<SecurityLabel>, IntrospectionError> {
    let mut labels = security_labels_from_rows(
        source_id,
        query(
            client,
            source_id,
            "security labels",
            r#"
SELECT identified.type, identified.identity, label.provider, label.label
FROM pg_catalog.pg_seclabel AS label
CROSS JOIN LATERAL pg_catalog.pg_identify_object(
    label.classoid, label.objoid, label.objsubid
) AS identified
ORDER BY identified.type COLLATE "C", identified.identity COLLATE "C",
         label.provider COLLATE "C", label.label COLLATE "C"
"#,
        )?,
    )?;
    let shared_sql = if include_cluster_objects {
        r#"
SELECT identified.type, identified.identity, label.provider, label.label
FROM pg_catalog.pg_shseclabel AS label
CROSS JOIN LATERAL pg_catalog.pg_identify_object(label.classoid, label.objoid, 0) AS identified
ORDER BY identified.type COLLATE "C", identified.identity COLLATE "C",
         label.provider COLLATE "C", label.label COLLATE "C"
"#
    } else {
        r#"
SELECT identified.type, identified.identity, label.provider, label.label
FROM pg_catalog.pg_shseclabel AS label
JOIN pg_catalog.pg_database AS database
  ON label.classoid = 'pg_catalog.pg_database'::pg_catalog.regclass
 AND label.objoid = database.oid
CROSS JOIN LATERAL pg_catalog.pg_identify_object(label.classoid, label.objoid, 0) AS identified
WHERE database.datname = pg_catalog.current_database()
ORDER BY identified.type COLLATE "C", identified.identity COLLATE "C",
         label.provider COLLATE "C", label.label COLLATE "C"
"#
    };
    labels.extend(security_labels_from_rows(
        source_id,
        query(client, source_id, "shared security labels", shared_sql)?,
    )?);
    labels.sort_by(|left, right| {
        (
            left.object_kind,
            &left.object_identity,
            &left.provider,
            &left.label,
        )
            .cmp(&(
                right.object_kind,
                &right.object_identity,
                &right.provider,
                &right.label,
            ))
    });
    Ok(labels)
}

fn security_labels_from_rows(
    source_id: &SourceId,
    rows: Vec<Row>,
) -> Result<Vec<SecurityLabel>, IntrospectionError> {
    rows.into_iter()
        .map(|row| {
            Ok(SecurityLabel {
                object_kind: security_label_object_kind(source_id, &row.get::<_, String>(0))?,
                object_identity: row.get(1),
                provider: row.get(2),
                label: row.get(3),
            })
        })
        .collect()
}

fn security_label_object_kind(
    source_id: &SourceId,
    value: &str,
) -> Result<SecurityLabelObjectKind, IntrospectionError> {
    match value {
        "aggregate" => Ok(SecurityLabelObjectKind::Aggregate),
        "database" => Ok(SecurityLabelObjectKind::Database),
        "domain" => Ok(SecurityLabelObjectKind::Domain),
        "event trigger" => Ok(SecurityLabelObjectKind::EventTrigger),
        "foreign table" => Ok(SecurityLabelObjectKind::ForeignTable),
        "function" => Ok(SecurityLabelObjectKind::Function),
        "large object" => Ok(SecurityLabelObjectKind::LargeObject),
        "materialized view" => Ok(SecurityLabelObjectKind::MaterializedView),
        "procedure" => Ok(SecurityLabelObjectKind::Procedure),
        "publication" => Ok(SecurityLabelObjectKind::Publication),
        "role" => Ok(SecurityLabelObjectKind::Role),
        "schema" => Ok(SecurityLabelObjectKind::Schema),
        "sequence" => Ok(SecurityLabelObjectKind::Sequence),
        "subscription" => Ok(SecurityLabelObjectKind::Subscription),
        "table" => Ok(SecurityLabelObjectKind::Table),
        "table column" => Ok(SecurityLabelObjectKind::TableColumn),
        "tablespace" => Ok(SecurityLabelObjectKind::Tablespace),
        "type" => Ok(SecurityLabelObjectKind::Type),
        "view" => Ok(SecurityLabelObjectKind::View),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_identify_object security-label type",
            other,
        )),
    }
}

fn load_large_objects(
    client: &mut Client,
    source_id: &SourceId,
) -> Result<Vec<LargeObject>, IntrospectionError> {
    Ok(query(
        client,
        source_id,
        "large objects",
        r#"
SELECT metadata.oid,
       owner.rolname,
       pg_catalog.obj_description(metadata.oid, 'pg_largeobject')
FROM pg_catalog.pg_largeobject_metadata AS metadata
JOIN pg_catalog.pg_roles AS owner ON owner.oid = metadata.lomowner
ORDER BY metadata.oid
"#,
    )?
    .into_iter()
    .map(|row| LargeObject {
        oid: row.get(0),
        owner: row.get(1),
        comment: row.get(2),
        contents_omitted: true,
    })
    .collect())
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

fn relation_persistence(
    source_id: &SourceId,
    value: &str,
) -> Result<RelationPersistence, IntrospectionError> {
    match value {
        "p" => Ok(RelationPersistence::Permanent),
        "u" => Ok(RelationPersistence::Unlogged),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_class.relpersistence",
            other,
        )),
    }
}

fn replica_identity(
    source_id: &SourceId,
    value: &str,
) -> Result<ReplicaIdentity, IntrospectionError> {
    match value {
        "d" => Ok(ReplicaIdentity::Default),
        "n" => Ok(ReplicaIdentity::Nothing),
        "f" => Ok(ReplicaIdentity::Full),
        "i" => Ok(ReplicaIdentity::Index),
        other => Err(unsupported_catalog_value(
            source_id,
            "pg_class.relreplident",
            other,
        )),
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
        "n" => Ok(ConstraintKind::NotNull),
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

fn foreign_key_match(
    source_id: &SourceId,
    code: &str,
) -> Result<ForeignKeyMatch, IntrospectionError> {
    match code {
        "f" => Ok(ForeignKeyMatch::Full),
        "p" => Ok(ForeignKeyMatch::Partial),
        "s" => Ok(ForeignKeyMatch::Simple),
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

fn cast_context(source_id: &SourceId, code: &str) -> Result<CastContext, IntrospectionError> {
    match code {
        "e" => Ok(CastContext::Explicit),
        "a" => Ok(CastContext::Assignment),
        "i" => Ok(CastContext::Implicit),
        _ => Err(unsupported_catalog_value(
            source_id,
            "pg_cast.castcontext",
            code,
        )),
    }
}

fn cast_method(source_id: &SourceId, code: &str) -> Result<CastMethod, IntrospectionError> {
    match code {
        "f" => Ok(CastMethod::Function),
        "i" => Ok(CastMethod::InputOutput),
        "b" => Ok(CastMethod::Binary),
        _ => Err(unsupported_catalog_value(
            source_id,
            "pg_cast.castmethod",
            code,
        )),
    }
}

fn operator_purpose(
    source_id: &SourceId,
    code: &str,
) -> Result<OperatorPurpose, IntrospectionError> {
    match code {
        "s" => Ok(OperatorPurpose::Search),
        "o" => Ok(OperatorPurpose::Ordering),
        _ => Err(unsupported_catalog_value(
            source_id,
            "pg_amop.amoppurpose",
            code,
        )),
    }
}

fn access_method_kind(
    source_id: &SourceId,
    code: &str,
) -> Result<AccessMethodKind, IntrospectionError> {
    match code {
        "t" => Ok(AccessMethodKind::Table),
        "i" => Ok(AccessMethodKind::Index),
        _ => Err(unsupported_catalog_value(source_id, "pg_am.amtype", code)),
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
    /// PostgreSQL returned an internally inconsistent catalog row.
    #[error(
        "invalid PostgreSQL {catalog} state while introspecting source `{source_id}`: {detail}"
    )]
    CatalogInvariant {
        /// Stable source identity, safe for diagnostics.
        source_id: SourceId,
        /// Catalog field or relationship being validated.
        catalog: &'static str,
        /// Static invariant description with no source-provided secret material.
        detail: &'static str,
    },
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use dbmd_core::SourceId;

    use super::{
        default_privilege_object, policy_command, privilege_kind, privilege_object_kind,
        trigger_when_expression,
    };
    use crate::{DefaultPrivilegeObject, PolicyCommand, PrivilegeKind, PrivilegeObjectKind};

    #[test]
    fn decodes_access_control_catalog_values_into_semantic_enums() {
        let source = SourceId::from_str("app").expect("test source ID should be valid");

        assert_eq!(policy_command(&source, "r").unwrap(), PolicyCommand::Select);
        assert_eq!(policy_command(&source, "a").unwrap(), PolicyCommand::Insert);
        assert_eq!(policy_command(&source, "w").unwrap(), PolicyCommand::Update);
        assert_eq!(policy_command(&source, "d").unwrap(), PolicyCommand::Delete);
        assert_eq!(policy_command(&source, "*").unwrap(), PolicyCommand::All);
        assert_eq!(
            privilege_object_kind(&source, "table column").unwrap(),
            PrivilegeObjectKind::TableColumn
        );
        assert_eq!(
            privilege_kind(&source, "MAINTAIN").unwrap(),
            PrivilegeKind::Maintain
        );
        assert_eq!(
            default_privilege_object(&source, "L").unwrap(),
            DefaultPrivilegeObject::LargeObjects
        );
    }

    #[test]
    fn rejects_unknown_access_control_values_with_catalog_and_source_context() {
        let source = SourceId::from_str("app").expect("test source ID should be valid");

        for error in [
            policy_command(&source, "future").expect_err("unknown policy code should fail"),
            privilege_kind(&source, "SUPERUSER").expect_err("unknown privilege should fail"),
            privilege_object_kind(&source, "future object")
                .expect_err("unknown object family should fail"),
            default_privilege_object(&source, "?")
                .expect_err("unknown default privilege family should fail"),
        ] {
            let message = error.to_string();
            assert!(message.contains("source `app`"), "{message}");
            assert!(message.contains("unsupported PostgreSQL"), "{message}");
        }
    }

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
