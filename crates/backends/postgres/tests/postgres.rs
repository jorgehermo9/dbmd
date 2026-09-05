use std::str::FromStr;

use dbmd_backend_postgres::{
    introspect, render_source, template_files, AccessMethodKind, AggregateFinalModify,
    AggregateKind, CastContext, CastMethod, CollationProvider, ColumnCompression, ColumnStorage,
    ConstraintKind, DefaultPrivilegeObject, EventTriggerEvent, FunctionKind,
    FunctionParallel as PostgresFunctionParallel, FunctionVolatility as PostgresFunctionVolatility,
    GeneratedColumnKind, IdentityGeneration, IndexNullsOrder, IndexTarget, OperatorKind,
    OperatorPurpose, PolicyCommand as PostgresPolicyCommand, PostgresSource, PrivilegeKind,
    PrivilegeObjectKind, PublicationGeneratedColumns, RelationPersistence, ReplicaIdentity,
    RewriteRuleEvent, SequencePersistence, StatisticsKind, SubscriptionOrigin,
    SubscriptionStreaming, SubscriptionTwoPhase, SynchronousCommit,
    TriggerEnabled as PostgresTriggerEnabled, TriggerEvent, TriggerOrientation, TriggerTiming,
    TypeAlignment, TypeStorage, ViewCheckOption,
};
use dbmd_core::SourceId;
use dbmd_relational::{ForeignKeyAction, ForeignKeyInitialTiming, ForeignKeyMatch, IndexSortOrder};
use dbmd_render::{
    OutputLayout, RenderContext, RenderOptions, RenderedArtifact, Renderer, SourceLayout,
};
use dbmd_test_support::{run_postgres_cases, PostgresCase, PostgresServer, TestResult};
use postgres::{Client, NoTls};

#[test]
fn refused_connection_error_is_source_scoped_and_credential_free() {
    let source = PostgresSource::new(
        SourceId::from_str("unavailable").expect("test source ID should be valid"),
        "postgres://dbmd:sentinel-postgres-secret@127.0.0.1:1/missing?connect_timeout=1",
    );

    let error = introspect(&source).expect_err("refused PostgreSQL connection should fail");
    let diagnostic = format!("{error}\n{error:?}\n{source:?}");

    assert!(diagnostic.contains("PostgreSQL source `unavailable`"));
    assert!(!diagnostic.contains("sentinel-postgres-secret"));
}

const CASES: &[PostgresCase] = &[
    PostgresCase {
        name: "ordinary_table",
        run: introspects_an_ordinary_table,
    },
    PostgresCase {
        name: "relationships",
        run: introspects_composite_constraints_and_foreign_keys,
    },
    PostgresCase {
        name: "schema_objects",
        run: introspects_namespaces_enums_views_and_functions,
    },
    PostgresCase {
        name: "table_semantics",
        run: introspects_inheritance_partitioning_and_row_level_security,
    },
    PostgresCase {
        name: "indexes_and_constraints",
        run: introspects_postgres_index_and_constraint_semantics,
    },
    PostgresCase {
        name: "triggers",
        run: introspects_postgres_trigger_semantics,
    },
    PostgresCase {
        name: "sequences",
        run: introspects_postgres_sequences,
    },
    PostgresCase {
        name: "domains",
        run: introspects_postgres_domains,
    },
    PostgresCase {
        name: "composite_types",
        run: introspects_postgres_composite_types,
    },
    PostgresCase {
        name: "postgres_18",
        run: introspects_postgres_18_schema_semantics,
    },
    PostgresCase {
        name: "type_system",
        run: introspects_base_shell_range_and_multirange_types,
    },
    PostgresCase {
        name: "aggregates",
        run: introspects_normal_ordered_and_hypothetical_aggregates,
    },
    PostgresCase {
        name: "routines",
        run: introspects_function_execution_contracts,
    },
    PostgresCase {
        name: "table_properties",
        run: introspects_storage_typed_and_foreign_table_properties,
    },
    PostgresCase {
        name: "type_operator_infrastructure",
        run: introspects_type_and_operator_infrastructure,
    },
    PostgresCase {
        name: "advanced_schema_objects",
        run: introspects_rules_event_triggers_and_statistics,
    },
];

#[test]
fn postgres_schema_surface_fixtures() {
    run_postgres_cases(CASES);
}

#[test]
fn postgres_pgvector_extension_fixture() {
    introspects_pgvector_extension_ownership()
        .expect("pinned PostgreSQL and pgvector fixture should pass");
}

#[test]
fn postgres_cluster_objects_fixture() {
    introspects_opt_in_cluster_objects()
        .expect("pinned PostgreSQL cluster-object fixture should pass");
}

#[test]
fn postgres_access_control_fixture() {
    introspects_access_control_metadata()
        .expect("pinned PostgreSQL access-control fixture should pass");
}

fn introspects_access_control_metadata() -> TestResult {
    let server = PostgresServer::start()?;
    let database = server.database(include_str!("fixtures/access_control/schema.sql"))?;
    let connection_string = database.connection_string();
    let source = PostgresSource::new(
        SourceId::from_str("access-control").expect("test source ID should be valid"),
        connection_string.clone(),
    )
    .with_cluster_objects(true);

    let snapshot = introspect(&source)?;
    let has_privilege = |object_kind, privilege| {
        snapshot.catalog().privileges.iter().any(|grant| {
            grant.object_kind == object_kind
                && grant.grantee == "dbmd_acl_reader"
                && grant.privilege == privilege
        })
    };
    assert!(has_privilege(
        PrivilegeObjectKind::Table,
        PrivilegeKind::Maintain
    ));
    assert!(has_privilege(
        PrivilegeObjectKind::TableColumn,
        PrivilegeKind::Update
    ));
    assert!(has_privilege(
        PrivilegeObjectKind::Parameter,
        PrivilegeKind::AlterSystem
    ));
    assert!(has_privilege(
        PrivilegeObjectKind::LargeObject,
        PrivilegeKind::Select
    ));
    assert!(has_privilege(
        PrivilegeObjectKind::Database,
        PrivilegeKind::Connect
    ));
    assert!(has_privilege(
        PrivilegeObjectKind::Tablespace,
        PrivilegeKind::Create
    ));
    let default_object_kinds = snapshot
        .catalog()
        .default_privileges
        .iter()
        .filter(|grant| grant.grantee == "dbmd_acl_reader")
        .map(|grant| grant.object_kind)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        default_object_kinds,
        [
            DefaultPrivilegeObject::Tables,
            DefaultPrivilegeObject::Sequences,
            DefaultPrivilegeObject::Routines,
            DefaultPrivilegeObject::Types,
            DefaultPrivilegeObject::Schemas,
            DefaultPrivilegeObject::LargeObjects,
        ]
        .into_iter()
        .collect()
    );
    let database_setting = snapshot
        .catalog()
        .role_database_settings
        .iter()
        .find(|setting| setting.role == "dbmd_acl_reader")
        .expect("fixture database-scoped role setting should be present");
    assert_eq!(database_setting.settings, ["lock_timeout=3s"]);
    assert_eq!(
        snapshot
            .catalog()
            .sequences
            .iter()
            .find(|sequence| sequence.name == "event_sequence")
            .expect("fixture sequence should be present")
            .owner,
        "dbmd_acl_owner"
    );
    assert_eq!(
        snapshot
            .catalog()
            .enums
            .iter()
            .find(|enum_type| enum_type.name == "event_state")
            .expect("fixture enum should be present")
            .owner,
        "dbmd_acl_owner"
    );
    let large_object = snapshot
        .catalog()
        .large_objects
        .iter()
        .find(|large_object| large_object.oid == 424242)
        .expect("fixture large object should be present");
    assert!(large_object.contents_omitted);
    assert_eq!(
        large_object.comment.as_deref(),
        Some("Fixture document payload")
    );
    assert!(snapshot.catalog().security_labels.is_empty());
    assert!(!format!("{snapshot:?}").contains("large-object-secret"));

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    assert!(markdown.contains("## Object Privileges"));
    assert!(markdown.contains("## Default Privileges"));
    assert!(markdown.contains("## Role Database Settings"));
    assert!(markdown.contains("## Large Objects"));
    assert!(!markdown.contains("large-object-secret"));
    insta::assert_yaml_snapshot!("access_control", snapshot);
    insta::assert_snapshot!("access_control_markdown", markdown);
    assert_repeat_single_file_render(&renderer, &context, &markdown)?;

    let RenderedArtifact::Directory(files) = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            ..RenderOptions::default()
        },
    )?
    else {
        panic!("directory options should produce a directory artifact");
    };
    for directory in [
        "role-database-settings/",
        "privileges/",
        "default-privileges/",
        "large-objects/",
    ] {
        assert!(files
            .keys()
            .any(|path| path.as_str().starts_with(directory)));
    }
    assert_eq!(snapshot, introspect(&source)?);

    let mut cleanup = Client::connect(&connection_string, NoTls)?;
    cleanup.batch_execute(
        "REVOKE CREATE ON TABLESPACE pg_default FROM dbmd_acl_reader;
         SELECT pg_catalog.lo_unlink(424242);
         DROP OWNED BY dbmd_acl_reader CASCADE;
         DROP OWNED BY dbmd_acl_owner CASCADE;
         DROP ROLE dbmd_acl_reader, dbmd_acl_owner;",
    )?;
    Ok(())
}

fn introspects_opt_in_cluster_objects() -> TestResult {
    let server = PostgresServer::start()?;
    let database = server.database(include_str!("fixtures/cluster/schema.sql"))?;
    let connection_string = database.connection_string();
    let source = PostgresSource::new(
        SourceId::from_str("cluster").expect("test source ID should be valid"),
        connection_string.clone(),
    )
    .with_cluster_objects(true);

    let snapshot = introspect(&source)?;
    assert!(snapshot.catalog().cluster_databases.len() >= 4);
    assert!(snapshot
        .catalog()
        .cluster_databases
        .iter()
        .any(|database| database.name == snapshot.catalog().database.name));
    let reader = snapshot
        .catalog()
        .roles
        .iter()
        .find(|role| role.name == "dbmd_cluster_reader")
        .expect("fixture reader role should be present");
    assert!(!reader.login);
    let application = snapshot
        .catalog()
        .roles
        .iter()
        .find(|role| role.name == "dbmd_cluster_app")
        .expect("fixture login role should be present");
    assert!(application.login);
    assert!(application.password_configured);
    assert_eq!(application.connection_limit, 5);
    assert_eq!(application.configuration, ["statement_timeout=5s"]);
    assert_eq!(snapshot.catalog().role_database_settings.len(), 1);
    assert_eq!(
        snapshot.catalog().role_database_settings[0].role,
        "dbmd_cluster_app"
    );
    assert_eq!(
        snapshot.catalog().role_database_settings[0].settings,
        ["lock_timeout=2s"]
    );
    assert_eq!(application.memberships.len(), 1);
    assert_eq!(application.memberships[0].role, "dbmd_cluster_reader");
    assert!(application.memberships[0].admin);
    assert!(!application.memberships[0].inherit);
    assert!(application.memberships[0].set);
    assert_eq!(
        application.comment.as_deref(),
        Some("Cluster fixture login role")
    );
    assert!(!format!("{snapshot:?}").contains("cluster-secret"));

    let default_tablespace = snapshot
        .catalog()
        .tablespaces
        .iter()
        .find(|tablespace| tablespace.name == "pg_default")
        .expect("default tablespace should be present");
    assert_eq!(default_tablespace.options, ["random_page_cost=1.1"]);
    assert!(default_tablespace.location_redacted);
    assert_eq!(
        default_tablespace.comment.as_deref(),
        Some("Cluster fixture default tablespace")
    );
    assert!(snapshot.catalog().privileges.iter().any(|privilege| {
        privilege.object_kind == PrivilegeObjectKind::Tablespace
            && privilege.object_identity == "pg_default"
            && privilege.grantee == "dbmd_cluster_reader"
            && privilege.privilege == PrivilegeKind::Create
    }));

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    assert!(markdown.contains("## Cluster Databases"));
    assert!(markdown.contains("## Tablespaces"));
    assert!(markdown.contains("## Roles"));
    assert!(markdown.contains("## Role Database Settings"));
    assert!(!markdown.contains("cluster-secret"));
    insta::assert_yaml_snapshot!("cluster_objects", snapshot);
    insta::assert_snapshot!("cluster_objects_markdown", markdown);
    assert_repeat_single_file_render(&renderer, &context, &markdown)?;

    let RenderedArtifact::Directory(files) = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            ..RenderOptions::default()
        },
    )?
    else {
        panic!("directory options should produce a directory artifact");
    };
    for directory in [
        "cluster-databases/",
        "tablespaces/",
        "roles/",
        "role-database-settings/",
    ] {
        assert!(files
            .keys()
            .any(|path| path.as_str().starts_with(directory)));
    }

    let repeated = introspect(&source)?;
    assert_eq!(snapshot, repeated);
    Client::connect(&connection_string, NoTls)?.batch_execute(
        "REVOKE CREATE ON TABLESPACE pg_default FROM dbmd_cluster_reader;
         ALTER TABLESPACE pg_default RESET (random_page_cost);
         COMMENT ON TABLESPACE pg_default IS NULL;
         DROP ROLE dbmd_cluster_app, dbmd_cluster_reader;",
    )?;
    Ok(())
}

#[test]
fn postgres_role_fixture() {
    introspects_roles_without_password_material()
        .expect("pinned PostgreSQL role fixture should pass");
}

fn introspects_roles_without_password_material() -> TestResult {
    let server = PostgresServer::start()?;
    let database = server.database(include_str!("fixtures/roles/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("access").expect("test source ID should be valid"),
        database.connection_string(),
    )
    .with_cluster_objects(true);

    let snapshot = introspect(&source)?;
    assert!(snapshot
        .catalog()
        .cluster_databases
        .iter()
        .any(|database| database.name == "postgres"));
    assert!(snapshot
        .catalog()
        .cluster_databases
        .iter()
        .any(|database| database.name == snapshot.catalog().database.name));
    assert_eq!(
        snapshot
            .catalog()
            .tablespaces
            .iter()
            .map(|tablespace| tablespace.name.as_str())
            .collect::<Vec<_>>(),
        ["pg_default", "pg_global"]
    );
    assert!(snapshot
        .catalog()
        .tablespaces
        .iter()
        .all(|tablespace| tablespace.location_redacted));
    assert_eq!(
        snapshot
            .catalog()
            .roles
            .iter()
            .map(|role| role.name.as_str())
            .collect::<Vec<_>>(),
        ["analyst", "reporting"]
    );
    let analyst = &snapshot.catalog().roles[0];
    assert!(analyst.login);
    assert!(!analyst.superuser);
    assert!(analyst.inherit);
    assert!(!analyst.create_role);
    assert!(!analyst.create_database);
    assert!(!analyst.replication);
    assert!(!analyst.bypass_row_level_security);
    assert_eq!(analyst.connection_limit, 3);
    assert!(analyst.password_configured);
    assert_eq!(analyst.configuration, ["statement_timeout=5s"]);
    assert_eq!(analyst.memberships.len(), 1);
    assert_eq!(analyst.memberships[0].role, "reporting");
    assert_eq!(analyst.memberships[0].grantor, "postgres");
    assert!(analyst.memberships[0].admin);
    assert!(!analyst.memberships[0].inherit);
    assert!(analyst.memberships[0].set);
    assert_eq!(analyst.comment.as_deref(), Some("Human analytics login"));
    let snapshot_debug = format!("{snapshot:?}");
    assert!(!snapshot_debug.contains("role-secret"));
    assert!(!snapshot_debug.contains("SCRAM-SHA-256"));

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    assert!(markdown.contains("## Roles"));
    assert!(markdown.contains("## Cluster Databases"));
    assert!(markdown.contains("## Tablespaces"));
    assert!(markdown.contains("**Location:** `<redacted>`"));
    assert!(markdown.contains("**Password configured:** yes"));
    assert!(markdown.contains("**Member of:** `reporting`"));
    assert!(!markdown.contains("role-secret"));
    assert!(!markdown.contains("SCRAM-SHA-256"));
    assert!(!markdown.contains("/var/lib/postgresql"));
    insta::assert_yaml_snapshot!("roles", snapshot);
    insta::assert_snapshot!("roles_markdown", markdown);
    assert_repeat_single_file_render(&renderer, &context, &markdown)?;

    let RenderedArtifact::Directory(files) = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            ..RenderOptions::default()
        },
    )?
    else {
        panic!("directory options should produce a directory artifact");
    };
    let role_path = "roles/role.analyst.md"
        .parse()
        .expect("static role artifact path should parse");
    let role_markdown = String::from_utf8(
        files
            .get(&role_path)
            .expect("role should have a directory artifact")
            .clone(),
    )?;
    assert!(role_markdown.contains("**Password configured:** yes"));
    assert!(!role_markdown.contains("role-secret"));
    assert!(files
        .keys()
        .any(|path| path.as_str().starts_with("cluster-databases/")));
    assert!(files
        .keys()
        .any(|path| path.as_str().starts_with("tablespaces/")));
    assert_eq!(snapshot, introspect(&source)?);
    Ok(())
}

fn introspects_pgvector_extension_ownership() -> TestResult {
    let server = PostgresServer::start_pgvector()?;
    let database = server.database(include_str!("fixtures/extensions/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("vectors").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;
    let extension = snapshot
        .catalog()
        .extensions
        .iter()
        .find(|extension| extension.name == "vector")
        .expect("pgvector extension should be present");
    assert_eq!(extension.version, "0.8.2");
    assert_eq!(extension.namespace, "public");
    assert_eq!(
        extension.comment.as_deref(),
        Some("Vector similarity search support")
    );
    assert!(extension.configuration.is_empty());
    assert!(extension
        .members
        .iter()
        .any(|member| { member.object_type == "type" && member.names == ["public.vector"] }));
    assert!(extension
        .members
        .iter()
        .any(|member| { member.object_type == "access method" && member.names == ["hnsw"] }));
    assert!(extension
        .members
        .iter()
        .any(|member| { member.object_type == "access method" && member.names == ["ivfflat"] }));

    let fixture_extension = snapshot
        .catalog()
        .extensions
        .iter()
        .find(|extension| extension.name == "dbmd_fixture")
        .expect("configuration-table fixture extension should be present");
    assert_eq!(fixture_extension.version, "1.0");
    assert_eq!(fixture_extension.configuration.len(), 1);
    assert_eq!(
        fixture_extension.configuration[0].relation,
        "public.dbmd_extension_config"
    );
    assert_eq!(
        fixture_extension.configuration[0].condition.as_deref(),
        Some("WHERE enabled")
    );
    let configuration_table = snapshot
        .catalog()
        .tables
        .iter()
        .find(|table| table.qualified_name() == "public.dbmd_extension_config")
        .expect("extension configuration table should remain modeled");
    assert_eq!(
        configuration_table.extension.as_deref(),
        Some("dbmd_fixture")
    );
    let owned_by_fixture = |extension: Option<&str>| extension == Some("dbmd_fixture");
    assert!(snapshot.catalog().enums.iter().any(|object| {
        object.name == "dbmd_extension_state" && owned_by_fixture(object.extension.as_deref())
    }));
    assert!(snapshot.catalog().composite_types.iter().any(|object| {
        object.name == "dbmd_extension_pair" && owned_by_fixture(object.extension.as_deref())
    }));
    assert!(snapshot.catalog().domains.iter().any(|object| {
        object.name == "dbmd_extension_positive" && owned_by_fixture(object.extension.as_deref())
    }));
    assert!(snapshot.catalog().range_types.iter().any(|object| {
        object.name == "dbmd_extension_range" && owned_by_fixture(object.extension.as_deref())
    }));
    assert!(snapshot.catalog().sequences.iter().any(|object| {
        object.name == "dbmd_extension_sequence" && owned_by_fixture(object.extension.as_deref())
    }));
    assert!(configuration_table.indexes.iter().any(|object| {
        object.name == "dbmd_extension_config_enabled_idx"
            && owned_by_fixture(object.extension.as_deref())
    }));
    assert!(snapshot.catalog().views.iter().any(|object| {
        object.name == "dbmd_extension_enabled" && owned_by_fixture(object.extension.as_deref())
    }));
    assert!(snapshot.catalog().triggers.iter().any(|object| {
        object.name == "dbmd_extension_config_trigger"
            && owned_by_fixture(object.extension.as_deref())
    }));
    assert!(snapshot.catalog().functions.iter().any(|object| {
        object.name == "dbmd_extension_identity" && owned_by_fixture(object.extension.as_deref())
    }));
    assert!(snapshot.catalog().procedures.iter().any(|object| {
        object.name == "dbmd_extension_noop" && owned_by_fixture(object.extension.as_deref())
    }));
    assert!(snapshot.catalog().aggregates.iter().any(|object| {
        object.name == "dbmd_extension_sum" && owned_by_fixture(object.extension.as_deref())
    }));
    assert!(snapshot.catalog().collations.iter().any(|object| {
        object.name == "dbmd_extension_collation" && owned_by_fixture(object.extension.as_deref())
    }));

    let vector_type = snapshot
        .catalog()
        .base_types
        .iter()
        .find(|base_type| base_type.name == "vector")
        .expect("extension-owned vector base type should remain modeled");
    assert_eq!(vector_type.extension.as_deref(), Some("vector"));
    assert!(snapshot
        .catalog()
        .casts
        .iter()
        .any(|object| object.extension.as_deref() == Some("vector")));
    assert!(snapshot
        .catalog()
        .operators
        .iter()
        .any(|object| object.extension.as_deref() == Some("vector")));
    assert!(snapshot
        .catalog()
        .operator_families
        .iter()
        .any(|object| object.extension.as_deref() == Some("vector")));
    assert!(snapshot
        .catalog()
        .operator_classes
        .iter()
        .any(|object| object.extension.as_deref() == Some("vector")));
    for name in ["hnsw", "ivfflat"] {
        let access_method = snapshot
            .catalog()
            .access_methods
            .iter()
            .find(|method| method.name == name)
            .expect("pgvector access method should remain modeled");
        assert_eq!(access_method.extension.as_deref(), Some("vector"));
    }
    let plpgsql = snapshot
        .catalog()
        .languages
        .iter()
        .find(|language| language.name == "plpgsql")
        .expect("default extension language should remain modeled");
    assert_eq!(plpgsql.extension.as_deref(), Some("plpgsql"));

    let items = snapshot
        .catalog()
        .tables
        .iter()
        .find(|table| table.qualified_name() == "app.items")
        .expect("application table should be present");
    assert!(items.extension.is_none());
    let embedding = items
        .columns
        .iter()
        .find(|column| column.name == "embedding")
        .expect("vector column should be present");
    assert_eq!(embedding.data_type, "vector(3)");
    let hnsw = items
        .indexes
        .iter()
        .find(|index| index.name == "items_embedding_hnsw_idx")
        .expect("HNSW index should be present");
    assert!(hnsw.extension.is_none());
    assert_eq!(hnsw.method, "hnsw");
    assert_eq!(
        hnsw.terms[0].operator_class.as_deref(),
        Some("public.vector_l2_ops")
    );
    let ivfflat = items
        .indexes
        .iter()
        .find(|index| index.name == "items_embedding_ivfflat_idx")
        .expect("IVFFlat index should be present");
    assert!(ivfflat.extension.is_none());
    assert_eq!(ivfflat.method, "ivfflat");
    assert_eq!(
        ivfflat.terms[0].operator_class.as_deref(),
        Some("public.vector_cosine_ops")
    );

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    assert!(markdown.contains("Vector similarity search support"));
    assert!(markdown.contains("Owned objects"));
    assert!(markdown.contains("vector(3)"));
    assert!(markdown.contains("postgres `hnsw`"));
    assert!(markdown.contains("public.vector_l2_ops"));
    assert!(markdown.contains("postgres `ivfflat`"));
    assert!(markdown.contains("public.vector_cosine_ops"));
    assert!(!markdown.contains("vector_in"));
    assert!(!markdown.contains("### `public.dbmd_extension_config`"));
    insta::assert_snapshot!("extensions_markdown", markdown);
    assert_repeat_single_file_render(&renderer, &context, &markdown)?;

    let RenderedArtifact::Directory(files) = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            ..RenderOptions::default()
        },
    )?
    else {
        panic!("directory options should produce a directory artifact");
    };
    let extension_path = "extensions/extensions.vector.md"
        .parse()
        .expect("static artifact path should parse");
    let extension_markdown = String::from_utf8(
        files
            .get(&extension_path)
            .expect("pgvector extension should have a directory object")
            .clone(),
    )?;
    let table_path = "tables/app.items.md"
        .parse()
        .expect("static artifact path should parse");
    let table_markdown = String::from_utf8(
        files
            .get(&table_path)
            .expect("application vector table should have a directory object")
            .clone(),
    )?;
    assert!(!files.contains_key(
        &"tables/public.dbmd_extension_config.md"
            .parse()
            .expect("static artifact path should parse")
    ));
    insta::assert_snapshot!(
        "extensions_directory",
        format!("{extension_markdown}\n---\n{table_markdown}")
    );

    let repeated = introspect(&source)?;
    assert_eq!(snapshot, repeated);
    Ok(())
}

fn introspects_an_ordinary_table(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/ordinary_table/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;

    assert!(snapshot.catalog().cluster_databases.is_empty());
    assert!(snapshot.catalog().tablespaces.is_empty());
    assert!(snapshot.catalog().roles.is_empty());
    assert_eq!(
        snapshot
            .catalog()
            .tables
            .iter()
            .map(|table| table.qualified_name())
            .collect::<Vec<_>>(),
        ["app.accounts", "zeta.audit_log"]
    );
    let accounts = &snapshot.catalog().tables[0];
    assert_eq!(accounts.comment.as_deref(), Some("Application accounts"));
    assert_eq!(
        accounts.columns[1].comment.as_deref(),
        Some("Canonical login address")
    );
    assert_eq!(
        accounts.columns[0].identity,
        Some(IdentityGeneration::Always)
    );
    assert_eq!(
        accounts.columns[2]
            .generated
            .as_ref()
            .map(|generated| generated.expression.as_str()),
        Some("lower(email)")
    );
    assert_eq!(
        accounts.columns[2]
            .generated
            .as_ref()
            .map(|generated| generated.kind),
        Some(GeneratedColumnKind::Stored)
    );
    assert!(accounts
        .constraints
        .iter()
        .any(|constraint| constraint.kind == ConstraintKind::Check
            && constraint.name.as_deref() == Some("accounts_email_nonempty")
            && constraint.expression.as_deref() == Some("email <> ''::text")));
    let index = accounts
        .indexes
        .iter()
        .find(|index| index.name == "accounts_normalized_email_idx")
        .expect("expression index should be present");
    assert!(index.unique);
    assert_eq!(index.predicate.as_deref(), Some("email <> ''::text"));
    assert_eq!(index.method, "btree");
    assert!(matches!(
        (&index.terms[0].target, index.terms[0].order),
        (IndexTarget::Expression(expression), IndexSortOrder::Descending)
            if expression == "lower(email)"
    ));
    let repeated = introspect(&source)?;
    assert_eq!(snapshot, repeated);
    insta::assert_yaml_snapshot!("ordinary_table", snapshot);
    Ok(())
}

fn introspects_composite_constraints_and_foreign_keys(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/relationships/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("billing").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;

    let accounts = snapshot
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "accounts")
        .expect("accounts table should be present");
    assert!(accounts.constraints.iter().any(|constraint| {
        constraint.name.as_deref() == Some("accounts_pk")
            && constraint.kind == ConstraintKind::PrimaryKey
            && constraint.columns == ["tenant_id", "account_id"]
    }));
    assert!(accounts.constraints.iter().any(|constraint| {
        constraint.name.as_deref() == Some("accounts_tenant_email_unique")
            && constraint.kind == ConstraintKind::Unique
            && constraint.columns == ["tenant_id", "email"]
    }));

    let invoices = snapshot
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "invoices")
        .expect("invoices table should be present");
    let foreign_key = invoices
        .constraints
        .iter()
        .find(|constraint| constraint.kind == ConstraintKind::ForeignKey)
        .expect("invoices foreign key should be present");
    assert_eq!(foreign_key.columns, ["tenant_id", "account_id"]);
    let reference = foreign_key
        .references
        .as_ref()
        .expect("foreign key target should be present");
    assert_eq!(reference.namespace, "billing");
    assert_eq!(reference.table, "accounts");
    assert_eq!(reference.columns, ["tenant_id", "account_id"]);
    assert_eq!(reference.on_update, ForeignKeyAction::Cascade);
    assert_eq!(reference.on_delete, ForeignKeyAction::Restrict);
    assert_eq!(reference.match_type, Some(ForeignKeyMatch::Full));
    assert!(reference.deferrability.deferrable);
    assert_eq!(
        reference.deferrability.initially,
        ForeignKeyInitialTiming::Deferred
    );
    insta::assert_yaml_snapshot!("relationships", snapshot);
    Ok(())
}

fn introspects_namespaces_enums_views_and_functions(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/schema_objects/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("catalog").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;

    assert_eq!(
        snapshot
            .catalog()
            .namespaces
            .iter()
            .map(|namespace| namespace.name.as_str())
            .collect::<Vec<_>>(),
        ["catalog", "empty_space", "public"]
    );
    assert_eq!(
        snapshot.catalog().namespaces[1].comment.as_deref(),
        Some("Reserved for future objects")
    );
    assert_eq!(snapshot.catalog().enums.len(), 1);
    assert_eq!(snapshot.catalog().enums[0].name, "account_state");
    assert_eq!(snapshot.catalog().enums[0].owner, "postgres");
    assert_eq!(
        snapshot.catalog().enums[0].values,
        ["invited", "active", "suspended"]
    );
    assert_eq!(snapshot.catalog().views.len(), 2);
    let active_accounts = snapshot
        .catalog()
        .views
        .iter()
        .find(|view| view.name == "active_accounts" && !view.materialized)
        .expect("ordinary fixture view should be present");
    assert!(active_accounts.security_barrier);
    assert!(active_accounts.security_invoker);
    assert_eq!(active_accounts.check_option, Some(ViewCheckOption::Local));
    let account_counts = snapshot
        .catalog()
        .views
        .iter()
        .find(|view| view.name == "account_counts" && view.materialized)
        .expect("materialized fixture view should be present");
    assert_eq!(account_counts.persistence, RelationPersistence::Permanent);
    assert_eq!(account_counts.access_method.as_deref(), Some("heap"));
    assert!(account_counts.tablespace.is_none());
    assert_eq!(account_counts.options, ["fillfactor=80"]);
    assert!(!account_counts.populated);
    assert_eq!(account_counts.indexes.len(), 1);
    assert_eq!(account_counts.indexes[0].name, "account_counts_state_idx");
    assert_eq!(
        account_counts.indexes[0].comment.as_deref(),
        Some("Supports state lookups on the account summary")
    );
    assert_eq!(snapshot.catalog().functions.len(), 1);
    let function = &snapshot.catalog().functions[0];
    assert_eq!(function.signature, "(value text)");
    assert_eq!(
        function.comment.as_deref(),
        Some("Normalizes an email address")
    );
    assert_eq!(function.volatility, PostgresFunctionVolatility::Immutable);
    assert_eq!(function.parallel, PostgresFunctionParallel::Safe);
    assert_eq!(snapshot.catalog().procedures.len(), 1);
    let procedure = &snapshot.catalog().procedures[0];
    assert_eq!(procedure.name, "archive_account");
    assert!(procedure.arguments.contains("INOUT archived boolean"));
    assert!(procedure.arguments.contains("DEFAULT 'manual'::text"));
    assert!(procedure.security_definer);
    assert_eq!(procedure.configuration, ["search_path=catalog, pg_temp"]);
    assert_eq!(
        procedure.comment.as_deref(),
        Some("Marks an account archive request")
    );
    insta::assert_yaml_snapshot!("schema_objects", snapshot);

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    assert!(markdown.contains("## Procedures"));
    assert!(markdown.contains("archive_account"));
    assert!(markdown.contains("search_path=catalog, pg_temp"));
    assert!(markdown.contains("**Security barrier:** yes"));
    assert!(markdown.contains("**Security invoker:** yes"));
    assert!(markdown.contains("**Check option:** `local`"));
    assert!(markdown.contains("**Populated:** no"));
    assert!(markdown.contains("**Option:** `fillfactor=80`"));
    insta::assert_snapshot!("schema_objects_markdown", markdown);
    assert_repeat_single_file_render(&renderer, &context, &markdown)?;

    let RenderedArtifact::Directory(files) = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            ..RenderOptions::default()
        },
    )?
    else {
        panic!("directory options should produce a directory artifact");
    };
    assert!(files.keys().any(|path| path
        .as_str()
        .starts_with("procedures/catalog.archive_account")));
    let active_accounts_path = "views/catalog.active_accounts.md"
        .parse()
        .expect("static view artifact path should parse");
    let active_accounts = String::from_utf8(
        files
            .get(&active_accounts_path)
            .expect("ordinary view should have a directory artifact")
            .clone(),
    )?;
    assert!(active_accounts.contains("**Security barrier:** yes"));
    assert!(active_accounts.contains("**Security invoker:** yes"));
    assert!(active_accounts.contains("**Check option:** `local`"));
    let account_counts_path = "views/catalog.account_counts.md"
        .parse()
        .expect("static materialized-view artifact path should parse");
    let account_counts = String::from_utf8(
        files
            .get(&account_counts_path)
            .expect("materialized view should have a directory artifact")
            .clone(),
    )?;
    assert!(account_counts.contains("**Populated:** no"));
    assert!(account_counts.contains("**Option:** `fillfactor=80`"));
    assert_eq!(snapshot, introspect(&source)?);
    Ok(())
}

fn introspects_normal_ordered_and_hypothetical_aggregates(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/aggregates/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("aggregates").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;
    assert_eq!(snapshot.catalog().aggregates.len(), 3);

    let integer_total = snapshot
        .catalog()
        .aggregates
        .iter()
        .find(|aggregate| aggregate.name == "integer_total")
        .expect("ordinary aggregate should be present");
    assert_eq!(integer_total.kind, AggregateKind::Normal);
    assert_eq!(integer_total.direct_arguments, 0);
    assert_eq!(integer_total.transition_type, "bigint");
    assert_eq!(
        integer_total.moving_transition_type.as_deref(),
        Some("bigint")
    );
    assert_eq!(integer_total.final_modify, AggregateFinalModify::Shareable);
    assert_eq!(
        integer_total.moving_final_modify,
        AggregateFinalModify::ReadWrite
    );
    assert!(integer_total.combine_function.is_some());
    assert!(integer_total.moving_inverse_function.is_some());
    assert_eq!(integer_total.initial_condition.as_deref(), Some("0"));
    assert_eq!(integer_total.moving_initial_condition.as_deref(), Some("0"));

    let ordered = snapshot
        .catalog()
        .aggregates
        .iter()
        .find(|aggregate| aggregate.name == "percentile_pick")
        .expect("ordered-set aggregate should be present");
    assert_eq!(ordered.kind, AggregateKind::OrderedSet);
    assert_eq!(ordered.direct_arguments, 1);
    assert!(ordered.arguments.contains("ORDER BY"));

    let hypothetical = snapshot
        .catalog()
        .aggregates
        .iter()
        .find(|aggregate| aggregate.name == "hypothetical_position")
        .expect("hypothetical-set aggregate should be present");
    assert_eq!(hypothetical.kind, AggregateKind::HypotheticalSet);
    assert_eq!(hypothetical.direct_arguments, 1);

    insta::assert_yaml_snapshot!("aggregates", snapshot);

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    assert!(markdown.contains("## Aggregates"));
    assert!(markdown.contains("integer_total"));
    assert!(markdown.contains("ordered_set"));
    assert!(markdown.contains("hypothetical_set"));
    insta::assert_snapshot!("aggregates_markdown", markdown);
    assert_repeat_single_file_render(&renderer, &context, &markdown)?;

    let RenderedArtifact::Directory(files) = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            ..RenderOptions::default()
        },
    )?
    else {
        panic!("directory options should produce a directory artifact");
    };
    assert_eq!(
        files
            .keys()
            .filter(|path| path.as_str().starts_with("aggregates/"))
            .count(),
        3
    );
    Ok(())
}

fn introspects_function_execution_contracts(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/routines/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("routines").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;
    assert_eq!(snapshot.catalog().functions.len(), 3);

    let starts_with = snapshot
        .catalog()
        .functions
        .iter()
        .find(|function| function.name == "starts_with")
        .expect("planner-supported function should be present");
    assert_eq!(starts_with.kind, FunctionKind::Ordinary);
    assert!(starts_with.arguments.contains("DEFAULT ''::text"));
    assert!(starts_with.strict);
    assert!(starts_with.leakproof);
    assert!(!starts_with.returns_set);
    assert_eq!(starts_with.cost, "3");
    assert!(starts_with
        .support_function
        .as_deref()
        .is_some_and(|support| support.contains("text_starts_with_support")));

    let set_returning = snapshot
        .catalog()
        .functions
        .iter()
        .find(|function| function.name == "range_values")
        .expect("set-returning function should be present");
    assert!(set_returning.returns_set);
    assert_eq!(set_returning.rows.as_deref(), Some("25"));
    assert_eq!(set_returning.cost, "7");
    assert_eq!(set_returning.configuration, ["search_path=pg_catalog"]);

    let window = snapshot
        .catalog()
        .functions
        .iter()
        .find(|function| function.name == "row_number_clone")
        .expect("window function should be present");
    assert_eq!(window.kind, FunctionKind::Window);

    insta::assert_yaml_snapshot!("routines", snapshot);
    Ok(())
}

fn introspects_storage_typed_and_foreign_table_properties(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/table_properties/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("storage").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;
    let table = |name: &str| {
        snapshot
            .catalog()
            .tables
            .iter()
            .find(|table| table.name == name)
            .unwrap_or_else(|| panic!("{name} table should be present"))
    };

    let typed = table("typed_devices");
    assert_eq!(typed.typed_table.as_deref(), Some("storage.device_row"));
    assert_eq!(typed.access_method.as_deref(), Some("heap"));

    let unlogged = table("event_payloads");
    assert_eq!(unlogged.persistence, RelationPersistence::Unlogged);
    assert_eq!(unlogged.replica_identity, ReplicaIdentity::Full);
    assert_eq!(unlogged.options, ["fillfactor=70"]);
    let payload = unlogged
        .columns
        .iter()
        .find(|column| column.name == "payload")
        .expect("configured payload column should be present");
    assert_eq!(payload.storage, ColumnStorage::External);
    assert_eq!(payload.compression, Some(ColumnCompression::Lz4));
    assert_eq!(payload.statistics_target, 777);
    assert_eq!(payload.options, ["n_distinct=-0.5"]);

    let foreign = table("remote_events");
    let details = foreign
        .foreign
        .as_ref()
        .expect("foreign-table details should be present");
    assert_eq!(details.server, "fixture_server");
    assert_eq!(details.wrapper, "fixture_wrapper");
    assert_eq!(details.options, ["schema_name=remote", "table_name=events"]);
    assert_eq!(
        foreign.columns[0].foreign_options,
        ["remote_name=external_id"]
    );

    let wrapper = snapshot
        .catalog()
        .foreign_data_wrappers
        .iter()
        .find(|wrapper| wrapper.name == "fixture_wrapper")
        .expect("fixture foreign-data wrapper should be present");
    assert_eq!(wrapper.owner, "postgres");
    assert_eq!(
        wrapper.options,
        ["api_token=<redacted>", "endpoint=catalog.example"]
    );
    assert_eq!(
        wrapper.comment.as_deref(),
        Some("Fixture foreign-data wrapper")
    );

    let server = snapshot
        .catalog()
        .foreign_servers
        .iter()
        .find(|server| server.name == "fixture_server")
        .expect("fixture foreign server should be present");
    assert_eq!(server.wrapper, "fixture_wrapper");
    assert_eq!(server.server_type.as_deref(), Some("catalog"));
    assert_eq!(server.version.as_deref(), Some("1.0"));
    assert_eq!(
        server.options,
        ["host=catalog.example", "password=<redacted>"]
    );
    assert_eq!(server.comment.as_deref(), Some("Fixture foreign server"));

    let mapping = snapshot
        .catalog()
        .user_mappings
        .iter()
        .find(|mapping| mapping.server == "fixture_server")
        .expect("fixture public user mapping should be present");
    assert_eq!(mapping.user, "PUBLIC");
    assert_eq!(
        mapping.options,
        ["user=catalog_reader", "password=<redacted>"]
    );
    let debug_snapshot = format!("{snapshot:?}");
    for secret in ["wrapper-secret", "server-secret", "mapping-secret"] {
        assert!(
            !debug_snapshot.contains(secret),
            "secret value reached the normalized catalog"
        );
    }

    insta::assert_yaml_snapshot!("table_properties", snapshot);

    let repeated = introspect(&source)?;
    assert_eq!(snapshot, repeated);

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    assert!(markdown.contains("## Foreign-Data Wrappers"));
    assert!(markdown.contains("## Foreign Servers"));
    assert!(markdown.contains("## User Mappings"));
    assert!(markdown.contains("<redacted>"));
    for secret in ["wrapper-secret", "server-secret", "mapping-secret"] {
        assert!(!markdown.contains(secret));
    }
    insta::assert_snapshot!("table_properties_markdown", markdown);
    assert_repeat_single_file_render(&renderer, &context, &markdown)?;

    let RenderedArtifact::Directory(files) = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            ..RenderOptions::default()
        },
    )?
    else {
        panic!("directory options should produce a directory artifact");
    };
    for path in [
        "foreign-data-wrappers/foreign-data-wrappers.fixture_wrapper.md",
        "foreign-servers/foreign-servers.fixture_server.md",
        "user-mappings/fixture_server.PUBLIC.md",
    ] {
        let path = path
            .parse()
            .expect("static external-data artifact path should parse");
        let rendered = String::from_utf8(
            files
                .get(&path)
                .expect("external-data object should have a directory artifact")
                .clone(),
        )?;
        assert!(rendered.contains("<redacted>") || !rendered.contains("Option"));
        for secret in ["wrapper-secret", "server-secret", "mapping-secret"] {
            assert!(!rendered.contains(secret));
        }
    }
    let index_path = "index.md"
        .parse()
        .expect("static artifact path should parse");
    let index = String::from_utf8(files[&index_path].clone())?;
    assert!(index.contains("foreign-data-wrappers/"));
    assert!(index.contains("foreign-servers/"));
    assert!(index.contains("user-mappings/"));
    Ok(())
}

fn introspects_inheritance_partitioning_and_row_level_security(
    server: &PostgresServer,
) -> TestResult {
    let database = server.database(include_str!("fixtures/table_semantics/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("tenancy").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;

    let table = |name: &str| {
        snapshot
            .catalog()
            .tables
            .iter()
            .find(|table| table.name == name)
            .unwrap_or_else(|| panic!("{name} table should be present"))
    };
    let base = table("base_events");
    assert!(base.row_level_security);
    assert!(base.force_row_level_security);
    assert_eq!(base.policies.len(), 1);
    assert_eq!(base.policies[0].name, "tenant_events");
    assert!(!base.policies[0].permissive);
    assert_eq!(base.policies[0].command, PostgresPolicyCommand::Select);
    assert_eq!(base.policies[0].roles, ["PUBLIC"]);

    let inherited = table("special_events");
    assert_eq!(inherited.inherits, ["tenancy.base_events"]);
    let inherited_tenant = inherited
        .columns
        .iter()
        .find(|column| column.name == "tenant_id")
        .expect("inherited tenant column should be present");
    assert!(!inherited_tenant.locally_defined);
    assert_eq!(inherited_tenant.inheritance_count, 1);

    let parent = table("events");
    assert_eq!(parent.partition_key.as_deref(), Some("RANGE (created_at)"));
    let parent_index = parent
        .indexes
        .iter()
        .find(|index| index.name == "events_created_idx")
        .expect("partitioned parent index should be present");
    assert!(parent_index.partitioned);
    assert!(parent_index.parent_index.is_none());

    let partition = table("events_2025");
    assert_eq!(
        partition.partition_parent.as_deref(),
        Some("tenancy.events")
    );
    assert_eq!(
        partition.partition_bound.as_deref(),
        Some("FOR VALUES FROM ('2025-01-01') TO ('2026-01-01')")
    );
    let partition_index = partition
        .indexes
        .iter()
        .find(|index| index.name == "events_2025_created_idx")
        .expect("attached partition index should be present");
    assert!(!partition_index.partitioned);
    assert_eq!(
        partition_index.parent_index.as_deref(),
        Some("tenancy.events_created_idx")
    );
    assert_eq!(partition_index.options, ["fillfactor=76"]);
    insta::assert_yaml_snapshot!("table_semantics", snapshot);

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    assert!(markdown.contains("partitioned"));
    assert!(markdown.contains("parent `tenancy.events_created_idx`"));
    assert!(markdown.contains("option `fillfactor=76`"));
    assert_eq!(snapshot, introspect(&source)?);
    Ok(())
}

fn introspects_postgres_index_and_constraint_semantics(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/indexes_and_constraints/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("search").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;
    let documents = snapshot
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "documents")
        .expect("documents table should be present");

    let check = documents
        .constraints
        .iter()
        .find(|constraint| constraint.name.as_deref() == Some("documents_title_check"))
        .expect("check constraint should be present");
    assert!(!check.validated);
    assert!(check.definition.ends_with("NOT VALID"));

    let unique = documents
        .constraints
        .iter()
        .find(|constraint| constraint.name.as_deref() == Some("documents_title_unique"))
        .expect("unique constraint should be present");
    assert!(unique.deferrable);
    assert!(unique.initially_deferred);
    assert!(unique.definition.contains("NULLS NOT DISTINCT"));
    let exclusion = documents
        .constraints
        .iter()
        .find(|constraint| constraint.name.as_deref() == Some("documents_active_window_exclude"))
        .expect("exclusion constraint should be present");
    assert_eq!(exclusion.kind, ConstraintKind::Exclusion);
    assert_eq!(exclusion.exclusion_operators.len(), 1);
    assert!(exclusion.exclusion_operators[0].contains("&&"));

    let lookup = documents
        .indexes
        .iter()
        .find(|index| index.name == "documents_lookup_idx")
        .expect("covering expression index should be present");
    assert_eq!(lookup.terms.len(), 2);
    assert_eq!(
        lookup.terms[0].operator_class.as_deref(),
        Some("pg_catalog.int8_ops")
    );
    assert_eq!(lookup.terms[0].nulls_order, Some(IndexNullsOrder::Last));
    assert_eq!(
        lookup.terms[1].collation.as_deref(),
        Some("pg_catalog.\"C\"")
    );
    assert_eq!(
        lookup.terms[1].operator_class.as_deref(),
        Some("pg_catalog.text_ops")
    );
    assert_eq!(lookup.terms[1].nulls_order, Some(IndexNullsOrder::First));
    assert_eq!(lookup.included_columns, ["body"]);
    assert!(lookup.nulls_not_distinct);
    assert!(lookup.valid);
    assert!(lookup.ready);
    assert_eq!(lookup.owner, "postgres");
    assert!(lookup.tablespace.is_none());
    assert_eq!(lookup.options, ["fillfactor=75"]);
    assert!(!lookup.partitioned);

    let brin = documents
        .indexes
        .iter()
        .find(|index| index.name == "documents_brin_idx")
        .expect("parameterized BRIN index should be present");
    assert_eq!(brin.method, "brin");
    assert_eq!(
        brin.terms[0].operator_class_parameters,
        ["n_distinct_per_range=32", "false_positive_rate=0.05"]
    );

    let unique_index = documents
        .indexes
        .iter()
        .find(|index| index.name == "documents_title_unique")
        .expect("constraint backing index should be present");
    assert_eq!(
        unique_index.constraint.as_deref(),
        Some("documents_title_unique")
    );

    let clustered = documents
        .indexes
        .iter()
        .find(|index| index.name == "documents_cluster_idx")
        .expect("clustered index should be present");
    assert!(clustered.clustered);
    let replica = documents
        .indexes
        .iter()
        .find(|index| index.name == "documents_replica_idx")
        .expect("replica identity index should be present");
    assert!(replica.replica_identity);

    insta::assert_yaml_snapshot!("indexes_and_constraints", snapshot);
    Ok(())
}

fn introspects_postgres_trigger_semantics(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/triggers/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("audit").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;

    assert_eq!(snapshot.catalog().triggers.len(), 7);
    assert_eq!(
        snapshot
            .catalog()
            .triggers
            .iter()
            .map(|trigger| (trigger.target.as_str(), trigger.name.as_str()))
            .collect::<Vec<_>>(),
        [
            ("account_emails", "account_emails_write"),
            ("accounts", "accounts_balance_constraint"),
            ("accounts", "accounts_transition"),
            ("accounts", "accounts_truncate"),
            ("accounts", "zz_accounts_change"),
            ("partitioned_events", "partitioned_events_change"),
            ("partitioned_events_2026", "partitioned_events_change"),
        ]
    );

    let row_change = snapshot
        .catalog()
        .triggers
        .iter()
        .find(|trigger| trigger.name == "zz_accounts_change")
        .expect("row-change trigger should be present");
    assert_eq!(
        row_change.comment.as_deref(),
        Some("Captures relevant account row changes")
    );
    assert_eq!(row_change.timing, TriggerTiming::Before);
    assert_eq!(row_change.orientation, TriggerOrientation::Row);
    assert_eq!(
        row_change.events,
        [
            TriggerEvent::Insert,
            TriggerEvent::Update {
                columns: vec!["email".to_string(), "balance".to_string()],
            },
            TriggerEvent::Delete,
        ]
    );
    assert_eq!(
        row_change.when_expression.as_deref(),
        Some("pg_trigger_depth() = 0")
    );
    assert_eq!(row_change.function, "audit.capture_row_change()");
    assert_eq!(row_change.arguments, ["history", "full"]);
    assert_eq!(row_change.enabled, PostgresTriggerEnabled::Always);
    assert!(row_change.constraint.is_none());

    let transition = snapshot
        .catalog()
        .triggers
        .iter()
        .find(|trigger| trigger.name == "accounts_transition")
        .expect("transition-table trigger should be present");
    assert_eq!(transition.orientation, TriggerOrientation::Statement);
    assert_eq!(transition.enabled, PostgresTriggerEnabled::Disabled);
    assert_eq!(
        transition.old_transition_table.as_deref(),
        Some("previous_rows")
    );
    assert_eq!(
        transition.new_transition_table.as_deref(),
        Some("current_rows")
    );

    let constraint = snapshot
        .catalog()
        .triggers
        .iter()
        .find(|trigger| trigger.name == "accounts_balance_constraint")
        .expect("constraint trigger should be present");
    let constraint = constraint
        .constraint
        .as_ref()
        .expect("constraint-trigger metadata should be present");
    assert_eq!(
        constraint.referenced_table.as_deref(),
        Some("audit.account_limits")
    );
    assert!(constraint.deferrable);
    assert!(constraint.initially_deferred);

    let truncate = snapshot
        .catalog()
        .triggers
        .iter()
        .find(|trigger| trigger.name == "accounts_truncate")
        .expect("truncate trigger should be present");
    assert_eq!(truncate.events, [TriggerEvent::Truncate]);
    assert_eq!(truncate.enabled, PostgresTriggerEnabled::Replica);

    let view = snapshot
        .catalog()
        .triggers
        .iter()
        .find(|trigger| trigger.name == "account_emails_write")
        .expect("view trigger should be present");
    assert_eq!(view.timing, TriggerTiming::InsteadOf);
    assert_eq!(view.orientation, TriggerOrientation::Row);

    let cloned = snapshot
        .catalog()
        .triggers
        .iter()
        .find(|trigger| trigger.target == "partitioned_events_2026")
        .expect("cloned partition trigger should be present");
    assert_eq!(
        cloned.parent_trigger.as_deref(),
        Some("audit.partitioned_events.partitioned_events_change")
    );

    let source = render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    );
    let context = RenderContext::new(vec![source]);
    insta::assert_yaml_snapshot!("triggers_render_context", context);
    let renderer = Renderer::embedded(template_files())?;
    let artifact = renderer.render(&context)?;
    let RenderedArtifact::SingleFile(markdown) = artifact else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    assert!(markdown.contains("BEFORE INSERT OR UPDATE OF email, balance OR DELETE"));
    assert!(markdown.contains("**Orientation:** `row`"));
    assert!(markdown.contains("**Enabled:** `always`"));
    assert!(markdown.contains("**Old transition table:** `previous_rows`"));
    assert!(markdown.contains("**Constraint trigger:** deferrable initially deferred"));

    let directory = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            source_layout: SourceLayout::Auto,
        },
    )?;
    let RenderedArtifact::Directory(files) = directory else {
        panic!("directory rendering should produce a file map");
    };
    assert!(files
        .keys()
        .any(|path| path.as_str() == "triggers/audit.accounts%2Ezz_accounts_change.md"));

    insta::assert_yaml_snapshot!("triggers", snapshot);
    Ok(())
}

fn introspects_postgres_sequences(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/sequences/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("automation").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;

    assert_eq!(snapshot.catalog().sequences.len(), 1);
    let sequence = &snapshot.catalog().sequences[0];
    assert_eq!(sequence.namespace, "automation");
    assert_eq!(sequence.name, "invoice_number");
    assert_eq!(sequence.data_type, "bigint");
    assert_eq!(sequence.start, 1000);
    assert_eq!(sequence.minimum, 1000);
    assert_eq!(sequence.maximum, 999_999);
    assert_eq!(sequence.increment, 5);
    assert_eq!(sequence.cache, 20);
    assert!(sequence.cycle);
    assert_eq!(sequence.persistence, SequencePersistence::Unlogged);
    assert_eq!(sequence.owned_by.as_deref(), Some("automation.invoices.id"));
    assert_eq!(
        sequence.comment.as_deref(),
        Some("Stable invoice number allocator")
    );
    insta::assert_yaml_snapshot!("sequences", snapshot);

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    assert!(markdown.contains("automation.invoice_number"));
    assert!(markdown.contains("Stable invoice number allocator"));
    assert!(markdown.contains("OWNED BY automation.invoices.id"));
    insta::assert_snapshot!("sequences_markdown", markdown);
    assert_repeat_single_file_render(&renderer, &context, &markdown)?;

    let RenderedArtifact::Directory(files) = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            ..RenderOptions::default()
        },
    )?
    else {
        panic!("directory options should produce a directory artifact");
    };
    let sequence_path = "sequences/automation.invoice_number.md"
        .parse()
        .expect("static artifact path should parse");
    assert!(files.contains_key(&sequence_path));
    let index_path = "index.md"
        .parse()
        .expect("static artifact path should parse");
    let index = String::from_utf8(files[&index_path].clone())?;
    assert!(index.contains("sequences/automation.invoice_number.md"));
    Ok(())
}

fn introspects_postgres_domains(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/domains/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("types").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;

    assert_eq!(snapshot.catalog().domains.len(), 1);
    let domain = &snapshot.catalog().domains[0];
    assert_eq!(domain.namespace, "types");
    assert_eq!(domain.name, "email_address");
    assert_eq!(domain.base_type, "text");
    assert_eq!(domain.collation.as_deref(), Some("pg_catalog.\"C\""));
    assert_eq!(domain.default.as_deref(), Some("''::text"));
    assert!(domain.not_null);
    assert_eq!(domain.owner, "domain_owner");
    assert_eq!(
        domain.comment.as_deref(),
        Some("Canonical application email address")
    );
    assert_eq!(domain.constraints.len(), 2);
    assert_eq!(domain.constraints[0].name, "email_not_blocked");
    assert!(!domain.constraints[0].validated);
    assert_eq!(domain.constraints[1].name, "email_shape");
    assert!(domain.constraints[1].validated);
    assert!(domain.definition.contains("CREATE DOMAIN"));
    assert!(domain.definition.contains("CONSTRAINT \"email_shape\""));
    assert!(domain.definition.contains("NOT VALID"));
    assert_eq!(
        snapshot.catalog().tables[0].columns[0].data_type,
        "types.email_address"
    );
    insta::assert_yaml_snapshot!("domains", snapshot);

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    assert!(markdown.contains("types.email_address"));
    assert!(markdown.contains("Canonical application email address"));
    assert!(markdown.contains("email_not_blocked"));
    insta::assert_snapshot!("domains_markdown", markdown);
    assert_repeat_single_file_render(&renderer, &context, &markdown)?;

    let RenderedArtifact::Directory(files) = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            ..RenderOptions::default()
        },
    )?
    else {
        panic!("directory options should produce a directory artifact");
    };
    let domain_path = "domains/types.email_address.md"
        .parse()
        .expect("static artifact path should parse");
    assert!(files.contains_key(&domain_path));
    let index_path = "index.md"
        .parse()
        .expect("static artifact path should parse");
    let index = String::from_utf8(files[&index_path].clone())?;
    assert!(index.contains("domains/types.email_address.md"));
    Ok(())
}

fn introspects_postgres_composite_types(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/composite_types/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("types").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;

    assert_eq!(snapshot.catalog().composite_types.len(), 1);
    let composite = &snapshot.catalog().composite_types[0];
    assert_eq!(composite.namespace, "types");
    assert_eq!(composite.name, "postal_address");
    assert_eq!(composite.owner, "type_owner");
    assert_eq!(
        composite.comment.as_deref(),
        Some("Reusable postal address")
    );
    assert_eq!(composite.attributes.len(), 3);
    assert_eq!(composite.attributes[0].name, "street");
    assert_eq!(composite.attributes[0].data_type, "text");
    assert_eq!(
        composite.attributes[0].collation.as_deref(),
        Some("pg_catalog.\"C\"")
    );
    assert_eq!(
        composite.attributes[1].comment.as_deref(),
        Some("Postal locality")
    );
    assert_eq!(composite.attributes[2].data_type, "character varying(12)");
    assert!(composite.definition.contains("CREATE TYPE"));
    assert!(composite
        .definition
        .contains("\"postal_code\" character varying(12)"));
    insta::assert_yaml_snapshot!("composite_types", snapshot);

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    assert!(markdown.contains("types.postal_address"));
    assert!(markdown.contains("Reusable postal address"));
    assert!(markdown.contains("Postal locality"));
    insta::assert_snapshot!("composite_types_markdown", markdown);
    assert_repeat_single_file_render(&renderer, &context, &markdown)?;

    let RenderedArtifact::Directory(files) = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            ..RenderOptions::default()
        },
    )?
    else {
        panic!("directory options should produce a directory artifact");
    };
    let composite_path = "composite-types/types.postal_address.md"
        .parse()
        .expect("static artifact path should parse");
    assert!(files.contains_key(&composite_path));
    let index_path = "index.md"
        .parse()
        .expect("static artifact path should parse");
    let index = String::from_utf8(files[&index_path].clone())?;
    assert!(index.contains("composite-types/types.postal_address.md"));
    Ok(())
}

fn introspects_postgres_18_schema_semantics(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/postgres_18/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("postgres18").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;
    let accounts = snapshot
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "accounts")
        .expect("accounts table should be present");
    let virtual_column = accounts
        .columns
        .iter()
        .find(|column| column.name == "virtual_amount")
        .expect("virtual generated column should be present")
        .generated
        .as_ref()
        .expect("virtual column should retain generation metadata");
    assert_eq!(virtual_column.kind, GeneratedColumnKind::Virtual);
    assert_eq!(virtual_column.expression, "base_amount * 2");
    let stored_column = accounts
        .columns
        .iter()
        .find(|column| column.name == "stored_amount")
        .expect("stored generated column should be present")
        .generated
        .as_ref()
        .expect("stored column should retain generation metadata");
    assert_eq!(stored_column.kind, GeneratedColumnKind::Stored);
    let not_null = accounts
        .constraints
        .iter()
        .find(|constraint| constraint.name.as_deref() == Some("accounts_email_required"))
        .expect("named not-null constraint should be present");
    assert_eq!(not_null.kind, ConstraintKind::NotNull);
    assert!(not_null.enforced);
    let check = accounts
        .constraints
        .iter()
        .find(|constraint| constraint.name.as_deref() == Some("accounts_amount_nonnegative"))
        .expect("unenforced check should be present");
    assert!(!check.enforced);
    assert_eq!(
        accounts
            .columns
            .iter()
            .find(|column| column.name == "email")
            .and_then(|column| column.collation.as_deref()),
        Some("temporal.unicode_fast")
    );
    let unicode_fast = snapshot
        .catalog()
        .collations
        .iter()
        .find(|collation| collation.name == "unicode_fast")
        .expect("builtin-provider collation should be present");
    assert_eq!(unicode_fast.provider, CollationProvider::Builtin);
    assert_eq!(unicode_fast.locale.as_deref(), Some("PG_UNICODE_FAST"));

    let temporal_unique = snapshot
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "plan_versions")
        .and_then(|table| {
            table
                .constraints
                .iter()
                .find(|constraint| constraint.name.as_deref() == Some("plan_versions_identity"))
        })
        .expect("WITHOUT OVERLAPS unique constraint should be present");
    assert!(temporal_unique.temporal);
    let period_fk = snapshot
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "plan_assignments")
        .and_then(|table| {
            table
                .constraints
                .iter()
                .find(|constraint| constraint.name.as_deref() == Some("assignments_plan_period"))
        })
        .expect("PERIOD foreign key should be present");
    assert!(period_fk.temporal);
    assert!(!period_fk.enforced);

    let temporal_changes = snapshot
        .catalog()
        .publications
        .iter()
        .find(|publication| publication.name == "temporal_changes")
        .expect("table publication should be present");
    assert_eq!(
        temporal_changes.generated_columns,
        PublicationGeneratedColumns::Stored
    );
    assert!(temporal_changes.publish_insert);
    assert!(!temporal_changes.publish_update);
    assert_eq!(temporal_changes.tables.len(), 1);
    assert_eq!(
        temporal_changes.tables[0]
            .columns
            .as_deref()
            .expect("explicit column list should be retained"),
        ["account_id", "stored_amount"]
    );
    assert_eq!(
        temporal_changes.tables[0].row_filter.as_deref(),
        Some("base_amount >= 0")
    );
    let schema_publication = snapshot
        .catalog()
        .publications
        .iter()
        .find(|publication| publication.name == "temporal_schema")
        .expect("schema publication should be present");
    assert_eq!(schema_publication.schemas, ["temporal"]);
    assert!(snapshot
        .catalog()
        .publications
        .iter()
        .any(|publication| publication.name == "all_tables" && publication.all_tables));
    let extension = snapshot
        .catalog()
        .extensions
        .iter()
        .find(|extension| extension.name == "btree_gist")
        .expect("btree_gist extension should be present");
    assert_eq!(
        extension.comment.as_deref(),
        Some("Temporal exclusion operator support")
    );
    assert!(
        extension.members.iter().any(|member| {
            member.object_type == "type" && member.names == ["public.gbtreekey16"]
        }),
        "btree_gist members: {:#?}",
        extension.members
    );

    insta::assert_yaml_snapshot!("postgres_18_schema_semantics", snapshot);

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    assert!(markdown.contains("generated"));
    assert!(markdown.contains("virtual"));
    assert!(markdown.contains("base_amount * 2"));
    assert!(markdown.contains("accounts_email_required"));
    assert!(markdown.contains("not enforced"));
    assert!(markdown.contains("temporal"));
    assert!(markdown.contains("temporal_changes"));
    assert!(markdown.contains("stored_amount"));
    assert!(markdown.contains("Generated columns"));
    assert!(markdown.contains("unicode_fast"));
    assert!(markdown.contains("PG_UNICODE_FAST"));
    insta::assert_snapshot!("postgres_18_schema_semantics_markdown", markdown);
    assert_repeat_single_file_render(&renderer, &context, &markdown)?;

    let RenderedArtifact::Directory(files) = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            ..RenderOptions::default()
        },
    )?
    else {
        panic!("directory options should produce a directory artifact");
    };
    let table_path = "tables/temporal.accounts.md"
        .parse()
        .expect("static artifact path should parse");
    let table = String::from_utf8(
        files
            .get(&table_path)
            .expect("PostgreSQL 18 table should have a directory object")
            .clone(),
    )?;
    assert!(table.contains("virtual"));
    assert!(table.contains("base_amount * 2"));
    assert!(table.contains("accounts_email_required"));
    let publication_path = "publications/publications.temporal_changes.md"
        .parse()
        .expect("static artifact path should parse");
    let publication = String::from_utf8(
        files
            .get(&publication_path)
            .expect("PostgreSQL publication should have a directory object")
            .clone(),
    )?;
    assert!(publication.contains("stored_amount"));
    assert!(publication.contains("base_amount"));
    let collation_path = "collations/temporal.unicode_fast.md"
        .parse()
        .expect("static artifact path should parse");
    let collation = String::from_utf8(
        files
            .get(&collation_path)
            .expect("PostgreSQL collation should have a directory object")
            .clone(),
    )?;
    assert!(collation.contains("builtin"));
    assert!(collation.contains("PG_UNICODE_FAST"));
    Ok(())
}

fn introspects_base_shell_range_and_multirange_types(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/type_system/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("types").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;
    assert_eq!(snapshot.catalog().base_types.len(), 2);
    let shell = snapshot
        .catalog()
        .base_types
        .iter()
        .find(|value| value.name == "pending_value")
        .expect("shell type should be present");
    assert!(!shell.defined);
    assert!(shell.details.is_none());
    assert_eq!(
        shell.definition,
        "CREATE TYPE \"type_system\".\"pending_value\";"
    );

    let base = snapshot
        .catalog()
        .base_types
        .iter()
        .find(|value| value.name == "scalar_token")
        .expect("base type should be present");
    let details = base
        .details
        .as_ref()
        .expect("defined base type should have implementation details");
    assert_eq!(details.internal_length, 4);
    assert!(details.passed_by_value);
    assert_eq!(details.category, "N");
    assert!(details.preferred);
    assert_eq!(details.alignment, TypeAlignment::Int);
    assert_eq!(details.storage, TypeStorage::Plain);
    assert_eq!(details.default.as_deref(), Some("0"));
    assert_eq!(
        base.comment.as_deref(),
        Some("Integer-backed application token")
    );

    assert_eq!(snapshot.catalog().range_types.len(), 1);
    let range = &snapshot.catalog().range_types[0];
    assert_eq!(range.name, "measurement_range");
    assert_eq!(range.subtype, "double precision");
    assert_eq!(range.subtype_operator_class, "pg_catalog.float8_ops");
    assert_eq!(range.subtype_diff.as_deref(), Some("pg_catalog.float8mi"));
    assert_eq!(range.multirange.name, "measurement_ranges");
    assert_eq!(
        range.multirange.comment.as_deref(),
        Some("Disjoint measurement intervals")
    );
    insta::assert_yaml_snapshot!("type_system", snapshot);

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    assert!(markdown.contains("Base and Shell Types"));
    assert!(markdown.contains("scalar_token_in"));
    assert!(markdown.contains("Range and Multirange Types"));
    assert!(markdown.contains("measurement_ranges"));
    insta::assert_snapshot!("type_system_markdown", markdown);
    assert_repeat_single_file_render(&renderer, &context, &markdown)?;

    let RenderedArtifact::Directory(files) = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            ..RenderOptions::default()
        },
    )?
    else {
        panic!("directory options should produce a directory artifact");
    };
    assert!(files
        .keys()
        .any(|path| { path.as_str() == "base-types/type_system.scalar_token.md" }));
    assert!(files
        .keys()
        .any(|path| { path.as_str() == "range-types/type_system.measurement_range.md" }));
    Ok(())
}

fn introspects_type_and_operator_infrastructure(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!(
        "fixtures/type_operator_infrastructure/schema.sql"
    ))?;
    let source = PostgresSource::new(
        SourceId::from_str("infrastructure").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;
    assert_eq!(snapshot.catalog().casts.len(), 3);
    assert!(snapshot.catalog().casts.iter().any(|cast| {
        cast.source_type == "infrastructure.label_a"
            && cast.target_type == "infrastructure.label_b"
            && cast.context == CastContext::Implicit
            && cast.method == CastMethod::Function
            && cast.function.as_deref()
                == Some("infrastructure.label_a_to_b(infrastructure.label_a)")
    }));
    assert!(snapshot.catalog().casts.iter().any(|cast| {
        cast.source_type == "infrastructure.label_b"
            && cast.target_type == "infrastructure.label_c"
            && cast.context == CastContext::Assignment
            && cast.method == CastMethod::InputOutput
    }));
    assert!(snapshot.catalog().casts.iter().any(|cast| {
        cast.source_type == "infrastructure.label_c"
            && cast.target_type == "integer"
            && cast.context == CastContext::Explicit
            && cast.method == CastMethod::Binary
    }));

    let conversion = snapshot
        .catalog()
        .conversions
        .iter()
        .find(|conversion| conversion.name == "utf8_to_latin1")
        .expect("fixture conversion should be present");
    assert_eq!(conversion.namespace, "infrastructure");
    assert_eq!(conversion.source_encoding, "UTF8");
    assert_eq!(conversion.target_encoding, "LATIN1");
    assert!(conversion.default);
    assert_eq!(
        conversion.comment.as_deref(),
        Some("Fixture default encoding conversion")
    );

    let binary = snapshot
        .catalog()
        .operators
        .iter()
        .find(|operator| operator.name == "===")
        .expect("binary fixture operator should be present");
    assert_eq!(binary.kind, OperatorKind::Binary);
    assert_eq!(binary.left_type.as_deref(), Some("integer"));
    assert_eq!(binary.right_type, "integer");
    assert!(binary.can_hash);
    assert!(binary.can_merge);
    assert!(binary.commutator.is_some());
    assert!(binary.restriction_selectivity.is_some());
    assert!(binary.join_selectivity.is_some());
    assert_eq!(binary.comment.as_deref(), Some("Fixture equality operator"));
    let prefix = snapshot
        .catalog()
        .operators
        .iter()
        .find(|operator| operator.name == "!!")
        .expect("prefix fixture operator should be present");
    assert_eq!(prefix.kind, OperatorKind::Prefix);
    assert!(prefix.left_type.is_none());

    let family = snapshot
        .catalog()
        .operator_families
        .iter()
        .find(|family| family.name == "integer_family")
        .expect("fixture operator family should be present");
    assert_eq!(family.access_method, "btree");
    assert_eq!(family.operators.len(), 5);
    assert!(family
        .operators
        .iter()
        .all(|operator| operator.purpose == OperatorPurpose::Search));
    assert_eq!(family.functions.len(), 1);
    assert_eq!(family.functions[0].number, 1);

    let class = snapshot
        .catalog()
        .operator_classes
        .iter()
        .find(|class| class.name == "integer_class")
        .expect("fixture operator class should be present");
    assert_eq!(class.access_method, "btree");
    assert_eq!(class.family, "infrastructure.integer_family");
    assert_eq!(class.input_type, "integer");
    assert!(!class.default);

    let access_method = snapshot
        .catalog()
        .access_methods
        .iter()
        .find(|method| method.name == "fixture_btree")
        .expect("fixture access method should be present");
    assert_eq!(access_method.kind, AccessMethodKind::Index);
    assert!(access_method.handler.contains("fixture_btree_handler"));

    let language = snapshot
        .catalog()
        .languages
        .iter()
        .find(|language| language.name == "fixture_pl")
        .expect("fixture language should be present");
    assert!(language.procedural);
    assert!(language.trusted);
    assert_eq!(
        language.comment.as_deref(),
        Some("Fixture procedural language")
    );

    let transform = snapshot
        .catalog()
        .transforms
        .iter()
        .find(|transform| transform.language == "fixture_pl")
        .expect("fixture transform should be present");
    assert_eq!(transform.data_type, "integer");
    assert!(transform
        .from_sql
        .as_deref()
        .is_some_and(|function| { function.contains("textlike_support") }));
    assert!(transform
        .to_sql
        .as_deref()
        .is_some_and(|function| function.contains("int4recv")));
    let procedure = snapshot
        .catalog()
        .procedures
        .iter()
        .find(|procedure| procedure.name == "accept_integer")
        .expect("fixture transform-aware procedure should be present");
    assert_eq!(procedure.transforms, ["integer"]);

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    for heading in [
        "## Casts",
        "## Encoding Conversions",
        "## Operators",
        "## Operator Families",
        "## Operator Classes",
        "## Access Methods",
        "## Procedural Languages",
        "## Transforms",
    ] {
        assert!(
            markdown.contains(heading),
            "missing rendered heading {heading}"
        );
    }
    insta::assert_yaml_snapshot!("type_operator_infrastructure", snapshot);
    insta::assert_snapshot!("type_operator_infrastructure_markdown", markdown);
    assert_repeat_single_file_render(&renderer, &context, &markdown)?;

    let RenderedArtifact::Directory(files) = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            ..RenderOptions::default()
        },
    )?
    else {
        panic!("directory options should produce a directory artifact");
    };
    for directory in [
        "casts/",
        "conversions/",
        "operators/",
        "operator-families/",
        "operator-classes/",
        "access-methods/",
        "languages/",
        "transforms/",
    ] {
        assert!(
            files
                .keys()
                .any(|path| path.as_str().starts_with(directory)),
            "missing directory artifact under {directory}"
        );
    }
    let index_path = "index.md"
        .parse()
        .expect("static artifact path should parse");
    insta::assert_snapshot!(
        "type_operator_infrastructure_directory",
        String::from_utf8(files[&index_path].clone())?
    );

    assert_eq!(snapshot, introspect(&source)?);
    Ok(())
}

fn introspects_rules_event_triggers_and_statistics(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/advanced_schema_objects/schema.sql"))?;
    let connection_string = database.connection_string();
    let source = PostgresSource::new(
        SourceId::from_str("advanced").expect("test source ID should be valid"),
        connection_string.clone(),
    );

    let snapshot = introspect(&source)?;
    let rule = snapshot
        .catalog()
        .rules
        .iter()
        .find(|rule| rule.name == "archive_order_delete")
        .expect("fixture rewrite rule should be present");
    assert_eq!(rule.namespace, "advanced");
    assert_eq!(rule.target, "orders");
    assert_eq!(rule.event, RewriteRuleEvent::Delete);
    assert!(!rule.instead);
    assert_eq!(rule.enabled, PostgresTriggerEnabled::Replica);
    assert_eq!(rule.comment.as_deref(), Some("Archives replicated deletes"));
    assert!(rule.definition.contains("CREATE RULE"));

    let event_trigger = snapshot
        .catalog()
        .event_triggers
        .iter()
        .find(|trigger| trigger.name == "capture_schema_change")
        .expect("fixture event trigger should be present");
    assert_eq!(event_trigger.event, EventTriggerEvent::DdlCommandEnd);
    assert_eq!(event_trigger.tags, ["CREATE TABLE", "ALTER TABLE"]);
    assert_eq!(event_trigger.enabled, PostgresTriggerEnabled::Always);
    assert!(event_trigger.function.contains("capture_schema_change"));
    assert_eq!(
        event_trigger.comment.as_deref(),
        Some("Captures selected schema changes")
    );

    let statistics = snapshot
        .catalog()
        .statistics
        .iter()
        .find(|statistics| statistics.name == "orders_dependencies")
        .expect("fixture extended statistics should be present");
    assert_eq!(statistics.target, 500);
    assert_eq!(statistics.columns, ["customer_id", "region"]);
    assert_eq!(
        statistics.kinds,
        [
            StatisticsKind::NdDistinct,
            StatisticsKind::Dependencies,
            StatisticsKind::MostCommonValues,
        ]
    );
    assert_eq!(
        statistics.comment.as_deref(),
        Some("Cross-column order distribution")
    );
    let expression_statistics = snapshot
        .catalog()
        .statistics
        .iter()
        .find(|statistics| statistics.name == "orders_expression")
        .expect("fixture expression statistics should be present");
    assert_eq!(expression_statistics.kinds, [StatisticsKind::Expressions]);
    assert_eq!(expression_statistics.expressions, ["lower(region)"]);

    let parser = snapshot
        .catalog()
        .text_search_parsers
        .iter()
        .find(|parser| parser.name == "default_parser")
        .expect("fixture text-search parser should be present");
    assert!(parser.start_function.contains("prsd_start"));
    assert_eq!(
        parser.comment.as_deref(),
        Some("Fixture parser backed by PostgreSQL defaults")
    );
    let template = snapshot
        .catalog()
        .text_search_templates
        .iter()
        .find(|template| template.name == "simple_template")
        .expect("fixture text-search template should be present");
    assert!(template
        .init_function
        .as_deref()
        .is_some_and(|function| function.contains("dsimple_init")));
    let dictionary = snapshot
        .catalog()
        .text_search_dictionaries
        .iter()
        .find(|dictionary| dictionary.name == "simple_dictionary")
        .expect("fixture text-search dictionary should be present");
    assert_eq!(dictionary.template, "advanced.simple_template");
    assert!(dictionary
        .options
        .as_deref()
        .is_some_and(|options| options.contains("stopwords")));
    let configuration = snapshot
        .catalog()
        .text_search_configurations
        .iter()
        .find(|configuration| configuration.name == "search_configuration")
        .expect("fixture text-search configuration should be present");
    assert_eq!(configuration.parser, "advanced.default_parser");
    let asciiword = configuration
        .mappings
        .iter()
        .find(|mapping| mapping.token_type == "asciiword")
        .expect("fixture asciiword mapping should be present");
    assert_eq!(
        asciiword.dictionaries,
        ["advanced.simple_dictionary", "pg_catalog.english_stem"]
    );

    let subscription = snapshot
        .catalog()
        .subscriptions
        .iter()
        .find(|subscription| subscription.name == "advanced_subscription")
        .expect("fixture subscription should be present");
    assert!(!subscription.enabled);
    assert!(subscription.binary);
    assert_eq!(subscription.streaming, SubscriptionStreaming::Parallel);
    assert_eq!(subscription.two_phase, SubscriptionTwoPhase::Pending);
    assert!(subscription.disable_on_error);
    assert!(!subscription.password_required);
    assert!(subscription.run_as_owner);
    assert!(subscription.failover);
    assert!(subscription.slot_name.is_none());
    assert_eq!(
        subscription.synchronous_commit,
        SynchronousCommit::RemoteApply
    );
    assert_eq!(subscription.publications, ["advanced_publication"]);
    assert_eq!(subscription.origin, SubscriptionOrigin::None);
    assert_eq!(subscription.skip_lsn.as_deref(), Some("0/16B6C50"));
    assert!(subscription.connection_redacted);
    assert_eq!(
        subscription.comment.as_deref(),
        Some("Disconnected fixture subscription")
    );
    assert!(!format!("{snapshot:?}").contains("subscription-secret"));

    let context = RenderContext::new(vec![render_source(
        snapshot.id(),
        snapshot.display_name(),
        snapshot.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files())?;
    let RenderedArtifact::SingleFile(markdown) = renderer.render(&context)? else {
        panic!("default PostgreSQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown)?;
    for heading in [
        "## Rewrite Rules",
        "## Event Triggers",
        "## Extended Statistics",
        "## Text Search Parsers",
        "## Text Search Templates",
        "## Text Search Dictionaries",
        "## Text Search Configurations",
        "## Subscriptions",
    ] {
        assert!(
            markdown.contains(heading),
            "missing rendered heading {heading}"
        );
    }
    assert!(markdown.contains("**Connection:** `<redacted>`"));
    assert!(markdown.contains("**Skip LSN:** `0/16B6C50`"));
    assert!(!markdown.contains("subscription-secret"));
    insta::assert_yaml_snapshot!("advanced_schema_objects", snapshot);
    insta::assert_snapshot!("advanced_schema_objects_markdown", markdown);
    assert_repeat_single_file_render(&renderer, &context, &markdown)?;

    let RenderedArtifact::Directory(files) = renderer.render_with_options(
        &context,
        RenderOptions {
            layout: OutputLayout::Directory,
            ..RenderOptions::default()
        },
    )?
    else {
        panic!("directory options should produce a directory artifact");
    };
    for directory in [
        "rules/",
        "event-triggers/",
        "statistics/",
        "text-search-parsers/",
        "text-search-templates/",
        "text-search-dictionaries/",
        "text-search-configurations/",
        "subscriptions/",
    ] {
        assert!(
            files
                .keys()
                .any(|path| path.as_str().starts_with(directory)),
            "missing directory artifact under {directory}"
        );
    }
    let subscription_path = "subscriptions/subscription.advanced_subscription.md"
        .parse()
        .expect("static subscription artifact path should parse");
    let subscription_markdown = String::from_utf8(
        files
            .get(&subscription_path)
            .expect("subscription should have a directory artifact")
            .clone(),
    )?;
    assert!(subscription_markdown.contains("**Connection:** `<redacted>`"));
    assert!(subscription_markdown.contains("**Skip LSN:** `0/16B6C50`"));
    assert!(!subscription_markdown.contains("subscription-secret"));
    insta::assert_snapshot!(
        "advanced_schema_objects_directory",
        String::from_utf8(
            files[&"index.md"
                .parse()
                .expect("static artifact path should parse")]
                .clone()
        )?
    );
    assert_eq!(snapshot, introspect(&source)?);
    Client::connect(&connection_string, NoTls)?
        .batch_execute("DROP SUBSCRIPTION advanced_subscription")?;
    Ok(())
}

fn assert_repeat_single_file_render(
    renderer: &Renderer,
    context: &RenderContext,
    expected: &str,
) -> TestResult {
    let RenderedArtifact::SingleFile(repeated) = renderer.render(context)? else {
        panic!("repeat PostgreSQL rendering should produce one file");
    };

    assert_eq!(repeated, expected.as_bytes());
    Ok(())
}
