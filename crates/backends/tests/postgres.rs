use std::str::FromStr;

use dbmd_backends::postgres::{
    introspect, FunctionParallel as PostgresFunctionParallel,
    FunctionVolatility as PostgresFunctionVolatility, PolicyCommand as PostgresPolicyCommand,
    PostgresSource, TriggerEnabled as PostgresTriggerEnabled, TriggerEvent, TriggerOrientation,
    TriggerTiming,
};
use dbmd_backends::relational::{
    ConstraintKind, ForeignKeyAction, ForeignKeyInitialTiming, IndexNullsOrder, IndexSortOrder,
    IndexTarget,
};
use dbmd_backends::{all_template_files, render_context, Catalog, DatabaseContext};
use dbmd_core::{SourceId, SourceSnapshot};
use dbmd_render::{OutputLayout, RenderOptions, RenderedArtifact, Renderer, SourceLayout};
use dbmd_test_support::{run_postgres_cases, PostgresCase, PostgresServer, TestResult};

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
];

#[test]
fn postgres_schema_surface_fixtures() {
    run_postgres_cases(CASES);
}

fn introspects_an_ordinary_table(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/postgres/ordinary_table/schema.sql"))?;
    let source = PostgresSource::new(
        SourceId::from_str("app").expect("test source ID should be valid"),
        database.connection_string(),
    );

    let snapshot = introspect(&source)?;

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
    assert_eq!(accounts.columns[0].identity.as_deref(), Some("always"));
    assert_eq!(
        accounts.columns[2].generated.as_deref(),
        Some("lower(email)")
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
    insta::assert_yaml_snapshot!("ordinary_table", snapshot);
    Ok(())
}

fn introspects_composite_constraints_and_foreign_keys(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/postgres/relationships/schema.sql"))?;
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
    assert_eq!(reference.match_name.as_deref(), Some("FULL"));
    assert!(reference.deferrability.deferrable);
    assert_eq!(
        reference.deferrability.initially,
        ForeignKeyInitialTiming::Deferred
    );
    insta::assert_yaml_snapshot!("relationships", snapshot);
    Ok(())
}

fn introspects_namespaces_enums_views_and_functions(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/postgres/schema_objects/schema.sql"))?;
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
    assert_eq!(
        snapshot.catalog().enums[0].values,
        ["invited", "active", "suspended"]
    );
    assert_eq!(snapshot.catalog().views.len(), 2);
    assert!(snapshot
        .catalog()
        .views
        .iter()
        .any(|view| view.name == "active_accounts" && !view.materialized));
    assert!(snapshot
        .catalog()
        .views
        .iter()
        .any(|view| view.name == "account_counts" && view.materialized));
    assert_eq!(snapshot.catalog().functions.len(), 1);
    let function = &snapshot.catalog().functions[0];
    assert_eq!(function.signature, "(value text)");
    assert_eq!(
        function.comment.as_deref(),
        Some("Normalizes an email address")
    );
    assert_eq!(function.volatility, PostgresFunctionVolatility::Immutable);
    assert_eq!(function.parallel, PostgresFunctionParallel::Safe);
    insta::assert_yaml_snapshot!("schema_objects", snapshot);
    Ok(())
}

fn introspects_inheritance_partitioning_and_row_level_security(
    server: &PostgresServer,
) -> TestResult {
    let database = server.database(include_str!("fixtures/postgres/table_semantics/schema.sql"))?;
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

    let parent = table("events");
    assert_eq!(parent.partition_key.as_deref(), Some("RANGE (created_at)"));

    let partition = table("events_2025");
    assert_eq!(
        partition.partition_parent.as_deref(),
        Some("tenancy.events")
    );
    assert_eq!(
        partition.partition_bound.as_deref(),
        Some("FOR VALUES FROM ('2025-01-01') TO ('2026-01-01')")
    );
    insta::assert_yaml_snapshot!("table_semantics", snapshot);
    Ok(())
}

fn introspects_postgres_index_and_constraint_semantics(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!(
        "fixtures/postgres/indexes_and_constraints/schema.sql"
    ))?;
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
    let database = server.database(include_str!("fixtures/postgres/triggers/schema.sql"))?;
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

    let composed = SourceSnapshot::new(
        snapshot.id().clone(),
        Catalog::Postgres(snapshot.catalog().clone()),
    );
    let database = DatabaseContext::new(vec![composed])?;
    let context = render_context(&database, false);
    insta::assert_yaml_snapshot!("triggers_render_context", context);
    let templates = all_template_files();
    let renderer = Renderer::embedded(&templates)?;
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
