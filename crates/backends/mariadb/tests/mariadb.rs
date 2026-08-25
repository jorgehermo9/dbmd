use std::str::FromStr;

use dbmd_backend_mariadb::{
    introspect, render_source, template_files, CheckConstraintLevel, ConstraintKind,
    GeneratedColumnStorage, MariaDbSource, ParameterMode, PartitionMethod, PluginKind,
    PluginLicense, PluginLoadOption, PluginMaturity, PluginStatus, PrivilegeObjectKind,
    RoutineDataAccess, RoutineKind, ScheduledEventCompletion, ScheduledEventKind,
    ScheduledEventStatus, ScheduledIntervalUnit, SqlSecurity, TlsRequirement, TriggerEvent,
    TriggerOrientation, TriggerTiming, ViewAlgorithm, ViewCheckOption,
};
use dbmd_core::SourceId;
use dbmd_relational::{ForeignKeyAction, IndexSortOrder};
use dbmd_render::{OutputLayout, RenderContext, RenderOptions, RenderedArtifact, Renderer};
use dbmd_test_support::MariaDbServer;

#[test]
fn introspects_and_renders_the_mariadb_schema_surface_deterministically() {
    let server = MariaDbServer::start(include_str!("fixtures/schema_surface.sql"))
        .expect("MariaDB test container should start");
    let source = MariaDbSource::new(
        SourceId::from_str("commerce").expect("test source ID should be valid"),
        server.url(),
    )
    .with_schema("test")
    .with_global_objects(true);

    let first = introspect(&source).expect("MariaDB introspection should succeed");
    let second = introspect(&source).expect("repeat introspection should succeed");
    assert_eq!(first, second);
    assert_eq!(first.catalog().tables.len(), 6);
    assert_eq!(
        first.catalog().schemas[0].comment.as_deref(),
        Some("Commerce schema fixture")
    );
    assert!(first.catalog().schemas[0]
        .definition
        .starts_with("CREATE DATABASE"));
    assert_eq!(first.catalog().sequences.len(), 2);
    let sequence = first
        .catalog()
        .sequences
        .iter()
        .find(|sequence| sequence.name == "order_number_seq")
        .expect("ascending sequence should be present");
    assert_eq!(sequence.data_type, "bigint");
    assert_eq!(sequence.numeric_precision, 64);
    assert_eq!(sequence.numeric_precision_radix, 2);
    assert_eq!(sequence.numeric_scale, 0);
    assert_eq!(sequence.start_value, "1000");
    assert_eq!(sequence.increment, "10");
    assert_eq!(sequence.cache, Some(20));
    assert!(!sequence.cycle);
    assert_eq!(sequence.engine.as_deref(), Some("InnoDB"));
    let descending_sequence = first
        .catalog()
        .sequences
        .iter()
        .find(|sequence| sequence.name == "descending_order_seq")
        .expect("descending sequence should be present");
    assert_eq!(descending_sequence.increment, "-2");
    assert_eq!(descending_sequence.cache, Some(0));
    assert!(descending_sequence.cycle);
    let accounts = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "accounts")
        .expect("accounts table should be present");
    assert!(accounts.system_versioned);
    assert_eq!(
        accounts
            .system_time_period
            .as_ref()
            .map(|period| period.start_column.as_str()),
        Some("row_start")
    );
    assert!(accounts
        .columns
        .iter()
        .any(|column| column.name == "normalized_email" && column.generation_expression.is_some()));
    assert!(accounts.columns.iter().any(|column| {
        column.name == "normalized_email"
            && column.generated_storage == Some(GeneratedColumnStorage::Stored)
    }));
    assert!(accounts
        .columns
        .iter()
        .any(|column| column.name == "profile_document" && column.column_type == "xmltype"));
    assert!(accounts
        .columns
        .iter()
        .any(|column| column.name == "secret_token" && !column.visible));
    assert!(accounts
        .columns
        .iter()
        .any(|column| column.name == "row_start" && column.system_time_period_start));
    assert!(accounts
        .constraints
        .iter()
        .any(|constraint| constraint.kind == ConstraintKind::ForeignKey));
    assert!(accounts.constraints.iter().any(|constraint| {
        constraint.name == "accounts_email_check"
            && constraint.check_level == Some(CheckConstraintLevel::Table)
    }));
    assert!(accounts.indexes.iter().any(|index| {
        index.name == "accounts_status_desc_idx"
            && index.comment.as_deref() == Some("Status lookup ordering")
    }));
    let accounts_tenant_fk = accounts
        .constraints
        .iter()
        .find(|constraint| constraint.name == "accounts_tenant_fk")
        .expect("account foreign key should be present");
    assert_eq!(accounts_tenant_fk.match_type, None);
    assert_eq!(
        accounts_tenant_fk.on_update,
        Some(ForeignKeyAction::Cascade)
    );
    assert_eq!(
        accounts_tenant_fk.on_delete,
        Some(ForeignKeyAction::Restrict)
    );
    assert!(accounts
        .indexes
        .iter()
        .any(|index| index.name == "accounts_email_ignored_idx" && index.ignored == Some(true)));
    let descending_term = accounts
        .indexes
        .iter()
        .find(|index| index.name == "accounts_status_desc_idx")
        .and_then(|index| index.terms.first())
        .expect("descending index term should be present");
    assert_eq!(descending_term.sort_order, Some(IndexSortOrder::Descending));
    let tenant_embeddings = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "tenant_embeddings")
        .expect("bitemporal vector table should be present");
    assert!(tenant_embeddings.system_versioned);
    assert_eq!(tenant_embeddings.application_time_periods.len(), 1);
    assert_eq!(
        tenant_embeddings.application_time_periods[0].name,
        "validity"
    );
    let temporal_constraint = tenant_embeddings
        .constraints
        .iter()
        .find(|constraint| constraint.name == "tenant_validity_uq")
        .expect("temporal unique constraint should be present");
    assert_eq!(temporal_constraint.period.as_deref(), Some("validity"));
    let vector_index = tenant_embeddings
        .indexes
        .iter()
        .find(|index| index.name == "embedding_vector_idx")
        .expect("vector index should be present");
    assert_eq!(vector_index.index_type, "VECTOR");
    let vector_options = vector_index
        .vector_options
        .as_ref()
        .expect("vector index options should be structured");
    assert_eq!(vector_options.m, Some(8));
    assert_eq!(vector_options.distance.as_deref(), Some("cosine"));
    let first_subpartition = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "monthly_metrics")
        .and_then(|table| table.partitions.first())
        .expect("subpartition metadata should be present");
    assert_eq!(first_subpartition.method, Some(PartitionMethod::Range));
    assert_eq!(
        first_subpartition.subpartition_method,
        Some(PartitionMethod::Hash)
    );
    assert_eq!(first.catalog().views.len(), 1);
    assert_eq!(first.catalog().views[0].algorithm, ViewAlgorithm::Merge);
    assert_eq!(
        first.catalog().views[0].check_option,
        ViewCheckOption::Cascaded
    );
    assert_eq!(first.catalog().views[0].security, SqlSecurity::Invoker);
    assert_eq!(first.catalog().routines.len(), 3);
    assert_eq!(first.catalog().packages.len(), 1);
    let package = &first.catalog().packages[0];
    assert_eq!(package.name, "analytics_tools");
    assert_eq!(package.security, SqlSecurity::Invoker);
    assert_eq!(package.comment.as_deref(), Some("Analytics package"));
    assert!(package
        .specification
        .definition
        .contains("CREATE DEFINER=`root`@`localhost` PACKAGE"));
    assert!(package
        .body
        .as_ref()
        .expect("package body should be present")
        .definition
        .contains("PACKAGE BODY"));
    assert!(first
        .catalog()
        .routines
        .iter()
        .all(|routine| routine.create_statement.starts_with("CREATE DEFINER")));
    assert_eq!(first.catalog().triggers.len(), 3);
    let updated_trigger = first
        .catalog()
        .triggers
        .iter()
        .find(|trigger| trigger.name == "accounts_updated")
        .expect("update-of trigger should be present");
    assert_eq!(updated_trigger.update_columns, ["email", "status"]);
    assert_eq!(updated_trigger.events, [TriggerEvent::Update]);
    assert_eq!(updated_trigger.timing, TriggerTiming::Before);
    assert_eq!(updated_trigger.orientation, TriggerOrientation::Row);
    let changed_trigger = first
        .catalog()
        .triggers
        .iter()
        .find(|trigger| trigger.name == "accounts_changed")
        .expect("multi-event trigger should be present");
    assert_eq!(
        changed_trigger.events,
        [
            TriggerEvent::Insert,
            TriggerEvent::Update,
            TriggerEvent::Delete
        ]
    );
    let normalize_email = first
        .catalog()
        .routines
        .iter()
        .find(|routine| routine.name == "normalize_email")
        .expect("defaulted function should be present");
    assert_eq!(
        normalize_email.parameters[1].default.as_deref(),
        Some("'fallback@example.invalid'")
    );
    assert_eq!(normalize_email.kind, RoutineKind::Function);
    assert_eq!(normalize_email.data_access, RoutineDataAccess::NoSql);
    assert_eq!(normalize_email.security, SqlSecurity::Definer);
    assert_eq!(normalize_email.parameters[1].mode, Some(ParameterMode::In));
    let duplicate_foreign_keys = first
        .catalog()
        .tables
        .iter()
        .flat_map(|table| {
            table
                .constraints
                .iter()
                .filter(|constraint| constraint.name == "accounts_tenant_fk")
        })
        .count();
    assert_eq!(duplicate_foreign_keys, 2);
    assert_eq!(first.catalog().events.len(), 2);
    assert!(first
        .catalog()
        .events
        .iter()
        .all(|event| event.create_statement.starts_with("CREATE DEFINER")));
    let recurring_event = first
        .catalog()
        .events
        .iter()
        .find(|event| event.name == "purge_disabled_accounts")
        .expect("recurring event should be present");
    assert_eq!(recurring_event.kind, ScheduledEventKind::Recurring);
    assert_eq!(
        recurring_event.interval_unit,
        Some(ScheduledIntervalUnit::Day)
    );
    assert_eq!(recurring_event.status, ScheduledEventStatus::Disabled);
    assert_eq!(
        recurring_event.completion,
        ScheduledEventCompletion::Preserve
    );
    assert_eq!(first.catalog().servers.len(), 1);
    let server = &first.catalog().servers[0];
    assert_eq!(server.name, "analytics_remote");
    assert_eq!(server.wrapper, "mariadb");
    assert_eq!(server.host.as_deref(), Some("db.internal"));
    assert_eq!(server.database.as_deref(), Some("analytics"));
    assert_eq!(server.username.as_deref(), Some("reader"));
    assert_eq!(server.port, Some(3307));
    assert_eq!(server.owner.as_deref(), Some("platform"));
    let region = server
        .options
        .iter()
        .find(|option| option.name == "REGION")
        .expect("custom server option should be present");
    assert_eq!(region.value.as_deref(), Some("eu-west-1"));
    assert!(!region.sensitive);
    let password = server
        .options
        .iter()
        .find(|option| option.name == "PASSWORD")
        .expect("credential option identity should be present");
    assert!(password.sensitive);
    assert_eq!(password.value, None);
    assert!(!format!("{first:?}").contains("dbmd-mariadb-server-secret-sentinel"));
    assert!(first.catalog().loadable_functions.is_empty());
    let blackhole_plugin = first
        .catalog()
        .plugins
        .iter()
        .find(|plugin| {
            plugin.name.eq_ignore_ascii_case("BLACKHOLE")
                && plugin.kind == PluginKind::StorageEngine
                && plugin.library.as_deref() == Some("ha_blackhole.so")
        })
        .expect("fixture plugin should be present");
    assert_eq!(blackhole_plugin.status, PluginStatus::Active);
    assert_eq!(blackhole_plugin.load_option, PluginLoadOption::On);
    assert_eq!(blackhole_plugin.maturity, PluginMaturity::Stable);
    assert_eq!(blackhole_plugin.license, PluginLicense::Gpl);
    let analytics_service = first
        .catalog()
        .accounts
        .iter()
        .find(|account| account.name == "analytics_service" && account.host == "localhost")
        .expect("fixture account should be present");
    assert_eq!(
        analytics_service.authentication_plugins,
        ["caching_sha2_password"]
    );
    assert_eq!(analytics_service.password_lifetime_days, Some(90));
    assert!(analytics_service.account_locked);
    assert_eq!(
        analytics_service.default_role.as_deref(),
        Some("analytics_reader")
    );
    assert_eq!(analytics_service.tls_requirement, TlsRequirement::Specified);
    assert_eq!(
        analytics_service.tls_cipher.as_deref(),
        Some("TLS_AES_256_GCM_SHA384")
    );
    assert_eq!(
        analytics_service.x509_issuer.as_deref(),
        Some("/CN=dbmd-ca")
    );
    assert_eq!(
        analytics_service.x509_subject.as_deref(),
        Some("/CN=dbmd-client")
    );
    assert_eq!(analytics_service.max_queries_per_hour, Some(17));
    assert_eq!(analytics_service.max_user_connections, Some(3));
    assert!(first
        .catalog()
        .role_memberships
        .iter()
        .any(|membership| membership.user == "analytics_service"
            && membership.host == "localhost"
            && membership.role == "analytics_reader"
            && membership.admin_option));
    assert!(first.catalog().privileges.iter().any(|privilege| {
        privilege.grantee == "'analytics_reader'@''"
            && privilege.privilege == "SELECT"
            && privilege.schema.as_deref() == Some("test")
    }));
    assert!(first.catalog().privileges.iter().any(|privilege| {
        privilege.grantee == "'analytics_reader'@''"
            && privilege.privilege == "EXECUTE"
            && privilege.object_kind == PrivilegeObjectKind::Function
            && privilege.object.as_deref() == Some("normalize_email")
    }));
    assert!(first.catalog().privileges.iter().any(|privilege| {
        privilege.grantee == "'analytics_reader'@''"
            && privilege.privilege == "SHOW CREATE ROUTINE"
            && privilege.object_kind == PrivilegeObjectKind::Schema
            && privilege.schema.as_deref() == Some("test")
            && privilege.object.is_none()
    }));
    assert!(first.catalog().privileges.iter().any(|privilege| {
        privilege.grantee == "'analytics_reader'@''"
            && privilege.privilege == "EXECUTE"
            && privilege.object_kind == PrivilegeObjectKind::Package
            && privilege.object.as_deref() == Some("analytics_tools")
    }));
    assert!(first.catalog().privileges.iter().any(|privilege| {
        privilege.grantee == "'analytics_service'@'localhost'"
            && privilege.object_kind == PrivilegeObjectKind::Proxy
            && privilege.object.as_deref() == Some("'proxy_target'@'localhost'")
            && privilege.grantable
    }));
    assert!(!format!("{first:?}").contains("$A$005$"));
    insta::assert_yaml_snapshot!("mariadb_schema_surface", first);

    let context = RenderContext::new(vec![render_source(
        first.id(),
        first.display_name(),
        first.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files()).expect("MariaDB templates should compile");
    let RenderedArtifact::SingleFile(markdown) = renderer
        .render(&context)
        .expect("MariaDB catalog should render")
    else {
        panic!("default MariaDB rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown).expect("Markdown should be UTF-8");
    assert!(markdown.contains("System versioning"));
    assert!(markdown.contains("Commerce schema fixture"));
    assert!(markdown.contains("order_number_seq"));
    assert!(markdown.contains("**Cache:** `20`"));
    assert!(markdown.contains("xmltype"));
    assert!(markdown.contains("Update columns"));
    assert!(markdown.contains("default 'fallback@example.invalid'"));
    assert!(markdown.contains("Application-time period"));
    assert!(markdown.contains("period validity without overlaps"));
    assert!(markdown.contains("M=8"));
    assert!(markdown.contains("distance=cosine"));
    assert!(markdown.contains("analytics_remote"));
    assert!(markdown.contains("analytics_tools"));
    assert!(markdown.contains("PACKAGE BODY"));
    assert!(markdown.contains("eu-west-1"));
    assert!(markdown.contains("[redacted]"));
    assert!(!markdown.contains("dbmd-mariadb-server-secret-sentinel"));
    assert!(markdown.contains("analytics_service@localhost"));
    assert!(markdown.contains("caching_sha2_password"));
    assert!(markdown.contains("Account locked"));
    assert!(!markdown.contains("$A$005$"));
    insta::assert_snapshot!("mariadb_markdown", markdown);
    assert_directory_render(&renderer, &context, "objects/test.order_number_seq.md");
    assert_directory_render(&renderer, &context, "objects/server.analytics_remote.md");
    assert_directory_render(&renderer, &context, "objects/test.analytics_tools.md");
    assert_directory_render(
        &renderer,
        &context,
        "objects/account.analytics_service%40localhost.md",
    );
}

fn assert_directory_render(renderer: &Renderer, context: &RenderContext, object_path: &str) {
    let RenderedArtifact::Directory(files) = renderer
        .render_with_options(
            context,
            RenderOptions {
                layout: OutputLayout::Directory,
                ..RenderOptions::default()
            },
        )
        .expect("MariaDB directory profile should render")
    else {
        panic!("directory options should produce a directory artifact");
    };
    let index_path = "index.md"
        .parse()
        .expect("static artifact path should parse");
    let index = String::from_utf8(files[&index_path].clone()).expect("index should be UTF-8");
    assert!(index.contains("objects/test.order_number_seq.md"));
    assert!(files.keys().any(|path| path.as_str() == object_path));
}
