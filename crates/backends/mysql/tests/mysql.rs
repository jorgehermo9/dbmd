use std::str::FromStr;

use dbmd_backend_mysql::{
    introspect, render_source, template_files, ConstraintKind, MysqlSource, PluginLoadOption,
    PluginStatus, PrivilegeObjectKind, ResourceGroupKind, RoutineDataAccess, RoutineKind,
    ScheduledEventKind, SqlSecurity, TriggerEvent, TriggerOrientation, TriggerTiming, ViewKind,
};
use dbmd_core::SourceId;
use dbmd_relational::{ForeignKeyAction, IndexSortOrder};
use dbmd_render::{OutputLayout, RenderContext, RenderOptions, RenderedArtifact, Renderer};
use dbmd_test_support::MysqlServer;

#[test]
fn introspects_and_renders_the_mysql_schema_surface_deterministically() {
    let server = MysqlServer::start(include_str!("fixtures/schema_surface.sql"))
        .expect("MySQL test container should start");
    let source = MysqlSource::new(
        SourceId::from_str("commerce").expect("test source ID should be valid"),
        server.url(),
    )
    .with_schema("test");

    let first = introspect(&source).expect("MySQL introspection should succeed");
    let second = introspect(&source).expect("repeat introspection should succeed");
    assert_eq!(first, second);
    assert_eq!(first.catalog().tables.len(), 6);
    let accounts = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "accounts")
        .expect("accounts table should be present");
    assert!(accounts
        .columns
        .iter()
        .any(|column| column.name == "normalized_email" && column.generation_expression.is_some()));
    assert!(accounts
        .columns
        .iter()
        .any(|column| column.name == "secret_token" && column.visible == Some(false)));
    assert!(accounts
        .constraints
        .iter()
        .any(|constraint| constraint.kind == ConstraintKind::ForeignKey
            && constraint.on_update == Some(ForeignKeyAction::Cascade)
            && constraint.on_delete == Some(ForeignKeyAction::Restrict)));
    assert!(accounts.indexes.iter().any(
        |index| index.name == "accounts_normalized_idx" && index.terms[0].expression.is_some()
    ));
    assert!(accounts
        .columns
        .iter()
        .any(|column| column.name == "embedding" && column.column_type == "vector(3)"));
    assert!(accounts.columns.iter().any(|column| {
        column.name == "default_embedding" && column.column_type == "vector(2048)"
    }));
    let inline_memberships = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "inline_memberships")
        .expect("inline foreign-key fixture should be present");
    assert!(inline_memberships.constraints.iter().any(|constraint| {
        constraint.kind == ConstraintKind::ForeignKey
            && constraint.referenced_table.as_deref() == Some("tenants")
            && constraint.referenced_columns == ["tenant_id"]
    }));
    assert_eq!(first.catalog().views.len(), 2);
    assert!(first.catalog().views.iter().any(|view| {
        view.name == "tenant_documents" && view.kind == ViewKind::JsonRelationalDuality
    }));
    assert_eq!(first.catalog().routines.len(), 3);
    assert_eq!(first.catalog().triggers.len(), 2);
    assert_eq!(first.catalog().events.len(), 2);
    assert_eq!(
        first
            .catalog()
            .tables
            .iter()
            .find(|table| table.name == "monthly_metrics")
            .expect("partitioned table should be present")
            .partitions
            .len(),
        4
    );
    assert!(accounts
        .indexes
        .iter()
        .any(|index| index.index_type == "FULLTEXT"));
    assert!(accounts.indexes.iter().any(|index| {
        index.name == "accounts_email_desc_idx"
            && index.terms[0].sort_order == Some(IndexSortOrder::Descending)
    }));
    assert!(accounts
        .indexes
        .iter()
        .any(|index| index.index_type == "SPATIAL"));
    assert!(accounts
        .constraints
        .iter()
        .any(|constraint| constraint.name == "accounts_status_check"
            && constraint.enforced == Some(false)));
    let generated_primary_key = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "generated_primary_key")
        .expect("generated invisible primary-key table should be present");
    assert!(generated_primary_key
        .columns
        .iter()
        .any(|column| column.name == "my_row_id" && column.visible == Some(false)));
    let memory_lookup = first
        .catalog()
        .tables
        .iter()
        .find(|table| table.name == "memory_lookup")
        .expect("MEMORY table should be present");
    assert!(memory_lookup
        .indexes
        .iter()
        .any(|index| index.index_type == "HASH"));
    assert!(first.catalog().routines.iter().any(|routine| {
        routine.name == "normalize_email"
            && routine.kind == RoutineKind::Function
            && routine.data_access == RoutineDataAccess::NoSql
            && routine.security == SqlSecurity::Definer
    }));
    assert!(first.catalog().triggers.iter().any(|trigger| {
        trigger.name == "accounts_updated"
            && trigger.event == TriggerEvent::Update
            && trigger.timing == TriggerTiming::Before
            && trigger.orientation == TriggerOrientation::Row
    }));
    assert!(first.catalog().events.iter().any(|event| {
        event.name == "purge_disabled_accounts" && event.kind == ScheduledEventKind::Recurring
    }));
    insta::assert_yaml_snapshot!("mysql_schema_surface", first);

    let context = RenderContext::new(vec![render_source(
        first.id(),
        first.display_name(),
        first.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files()).expect("MySQL templates should compile");
    let RenderedArtifact::SingleFile(markdown) = renderer
        .render(&context)
        .expect("MySQL catalog should render")
    else {
        panic!("default MySQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown).expect("Markdown should be UTF-8");
    assert!(markdown.contains("accounts_normalized_idx"));
    assert!(markdown.contains("purge_disabled_accounts"));
    assert!(markdown.contains("vector(3)"));
    assert!(markdown.contains("json_relational_duality"));
    insta::assert_snapshot!("mysql_markdown", markdown);
    assert_directory_render(&renderer, &context, "tables/test.accounts.md");
}

#[test]
fn introspects_global_objects_without_acquiring_secrets() {
    let server = MysqlServer::start(include_str!("fixtures/global_objects.sql"))
        .expect("MySQL global-object fixture should start");
    let source = MysqlSource::new(
        SourceId::from_str("global").expect("test source ID should be valid"),
        server.url(),
    )
    .with_schema("test")
    .with_global_objects(true);

    let first = introspect(&source).expect("MySQL global introspection should succeed");
    let second = introspect(&source).expect("repeat global introspection should succeed");
    assert_eq!(first, second);
    assert_eq!(first.catalog().servers.len(), 1);
    assert!(first.catalog().servers[0].password_configured);
    assert_eq!(first.catalog().spatial_reference_systems.len(), 1);
    assert!(first
        .catalog()
        .tablespaces
        .iter()
        .any(|tablespace| tablespace.name == "dbmd_general"));
    assert_eq!(first.catalog().resource_groups.len(), 1);
    assert_eq!(
        first.catalog().resource_groups[0].kind,
        ResourceGroupKind::User
    );
    assert!(first.catalog().accounts.iter().any(|account| {
        account.user == "dbmd_app"
            && account
                .authentication_factors
                .first()
                .is_some_and(|factor| factor.credential_configured)
            && account.comment.as_deref() == Some("dbmd application account")
            && account.attributes_configured
    }));
    assert!(first
        .catalog()
        .plugins
        .iter()
        .any(|plugin| plugin.name == "auth_socket"
            && plugin.status == PluginStatus::Active
            && plugin.load_option == PluginLoadOption::On));
    assert!(first
        .catalog()
        .components
        .iter()
        .any(|component| component.urn == "file://component_validate_password"));
    assert!(first
        .catalog()
        .role_grants
        .iter()
        .any(|grant| grant.role_user == "dbmd_reader" && grant.member_user == "dbmd_app"));
    assert!(first
        .catalog()
        .default_roles
        .iter()
        .any(|role| role.user == "dbmd_app" && role.role_user == "dbmd_reader"));
    assert!(first.catalog().privileges.iter().any(|privilege| {
        privilege.grantee.contains("dbmd_reader")
            && privilege.object_kind == PrivilegeObjectKind::Column
            && privilege.privilege == "UPDATE"
    }));
    assert!(first.catalog().privileges.iter().any(|privilege| {
        privilege.grantee.contains("dbmd_app")
            && privilege.object_kind == PrivilegeObjectKind::Proxy
            && privilege.privilege == "PROXY"
            && privilege.grantable
    }));

    let debug = format!("{first:?}");
    assert!(!debug.contains("dbmd-server-secret"));
    assert!(!debug.contains("dbmd-account-secret"));
    assert!(!debug.contains("dbmd-general.ibd"));
    insta::assert_yaml_snapshot!("mysql_global_objects", first);

    let context = RenderContext::new(vec![render_source(
        first.id(),
        first.display_name(),
        first.catalog(),
        false,
    )]);
    let renderer = Renderer::embedded(template_files()).expect("MySQL templates should compile");
    let RenderedArtifact::SingleFile(markdown) = renderer
        .render(&context)
        .expect("MySQL global catalog should render")
    else {
        panic!("default MySQL rendering should produce one file");
    };
    let markdown = String::from_utf8(markdown).expect("Markdown should be UTF-8");
    assert!(markdown.contains("dbmd_remote"));
    assert!(markdown.contains("dbmd geographic"));
    assert!(markdown.contains("dbmd_general"));
    assert!(markdown.contains("dbmd_app"));
    assert!(markdown.contains("<redacted>"));
    assert!(!markdown.contains("dbmd-server-secret"));
    assert!(!markdown.contains("dbmd-account-secret"));
    assert!(!markdown.contains("dbmd-general.ibd"));
    insta::assert_snapshot!("mysql_global_objects_markdown", markdown);
    assert_directory_render(&renderer, &context, "accounts/account.dbmd_app%40%25.md");
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
        .expect("MySQL directory profile should render")
    else {
        panic!("directory options should produce a directory artifact");
    };
    let index = String::from_utf8(files[&"index.md".parse().unwrap()].clone()).unwrap();
    assert!(index.contains("tables/test.accounts.md"));
    assert!(files.keys().any(|path| path.as_str() == object_path));
}
