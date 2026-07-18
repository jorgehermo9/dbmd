use std::str::FromStr;

use dbmd_core::{
    ColumnBackend, ConstraintBackend, ConstraintKind, ForeignKeyAction, ForeignKeyInitialTiming,
    IndexBackend, IndexNullsOrder, IndexSortOrder, IndexTarget, PostgresFunctionParallel,
    PostgresFunctionVolatility, PostgresPolicyCommand, SourceId, TableBackend,
};
use dbmd_introspect::postgres::{introspect, PostgresSource};
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
            .tables
            .iter()
            .map(|table| table.qualified_name())
            .collect::<Vec<_>>(),
        ["app.accounts", "zeta.audit_log"]
    );
    let accounts = &snapshot.tables[0];
    assert_eq!(accounts.comment.as_deref(), Some("Application accounts"));
    assert_eq!(
        accounts.columns[1].comment.as_deref(),
        Some("Canonical login address")
    );
    assert!(matches!(
        &accounts.columns[0].backend,
        ColumnBackend::Postgres(column) if column.identity.as_deref() == Some("always")
    ));
    assert!(matches!(
        &accounts.columns[2].backend,
        ColumnBackend::Postgres(column)
            if column.generated.as_deref() == Some("lower(email)")
    ));
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
    assert!(matches!(
        &index.backend,
        IndexBackend::Postgres(index) if index.method == "btree"
    ));
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
            .namespaces
            .iter()
            .map(|namespace| namespace.name.as_str())
            .collect::<Vec<_>>(),
        ["catalog", "empty_space", "public"]
    );
    assert_eq!(
        snapshot.namespaces[1].comment.as_deref(),
        Some("Reserved for future objects")
    );
    assert_eq!(snapshot.enums.len(), 1);
    assert_eq!(snapshot.enums[0].name, "account_state");
    assert_eq!(snapshot.enums[0].values, ["invited", "active", "suspended"]);
    assert_eq!(snapshot.views.len(), 2);
    assert!(snapshot
        .views
        .iter()
        .any(|view| view.name == "active_accounts" && !view.materialized));
    assert!(snapshot
        .views
        .iter()
        .any(|view| view.name == "account_counts" && view.materialized));
    assert_eq!(snapshot.functions.len(), 1);
    let function = &snapshot.functions[0];
    assert_eq!(function.signature, "(value text)");
    assert_eq!(
        function.comment.as_deref(),
        Some("Normalizes an email address")
    );
    assert_eq!(
        function.backend.volatility(),
        Some(PostgresFunctionVolatility::Immutable)
    );
    assert_eq!(
        function.backend.parallel(),
        Some(PostgresFunctionParallel::Safe)
    );
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
            .tables
            .iter()
            .find(|table| table.name == name)
            .unwrap_or_else(|| panic!("{name} table should be present"))
    };
    let TableBackend::Postgres(base) = &table("base_events").backend else {
        panic!("base_events should carry PostgreSQL semantics");
    };
    assert!(base.row_level_security);
    assert!(base.force_row_level_security);
    assert_eq!(base.policies.len(), 1);
    assert_eq!(base.policies[0].name, "tenant_events");
    assert!(!base.policies[0].permissive);
    assert_eq!(base.policies[0].command, PostgresPolicyCommand::Select);
    assert_eq!(base.policies[0].roles, ["PUBLIC"]);

    let TableBackend::Postgres(inherited) = &table("special_events").backend else {
        panic!("special_events should carry PostgreSQL semantics");
    };
    assert_eq!(inherited.inherits, ["tenancy.base_events"]);

    let TableBackend::Postgres(parent) = &table("events").backend else {
        panic!("events should carry PostgreSQL semantics");
    };
    assert_eq!(parent.partition_key.as_deref(), Some("RANGE (created_at)"));

    let TableBackend::Postgres(partition) = &table("events_2025").backend else {
        panic!("events_2025 should carry PostgreSQL semantics");
    };
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
        .tables
        .iter()
        .find(|table| table.name == "documents")
        .expect("documents table should be present");

    let check = documents
        .constraints
        .iter()
        .find(|constraint| constraint.name.as_deref() == Some("documents_title_check"))
        .expect("check constraint should be present");
    let ConstraintBackend::Postgres(check) = &check.backend else {
        panic!("check constraint should carry PostgreSQL semantics");
    };
    assert!(!check.validated);
    assert!(check.definition.ends_with("NOT VALID"));

    let unique = documents
        .constraints
        .iter()
        .find(|constraint| constraint.name.as_deref() == Some("documents_title_unique"))
        .expect("unique constraint should be present");
    let ConstraintBackend::Postgres(unique) = &unique.backend else {
        panic!("unique constraint should carry PostgreSQL semantics");
    };
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
    let IndexBackend::Postgres(lookup) = &lookup.backend else {
        panic!("lookup index should carry PostgreSQL semantics");
    };
    assert_eq!(lookup.included_columns, ["body"]);
    assert!(lookup.nulls_not_distinct);
    assert!(lookup.valid);
    assert!(lookup.ready);

    let clustered = documents
        .indexes
        .iter()
        .find(|index| index.name == "documents_cluster_idx")
        .expect("clustered index should be present");
    assert!(matches!(
        &clustered.backend,
        IndexBackend::Postgres(index) if index.clustered
    ));
    let replica = documents
        .indexes
        .iter()
        .find(|index| index.name == "documents_replica_idx")
        .expect("replica identity index should be present");
    assert!(matches!(
        &replica.backend,
        IndexBackend::Postgres(index) if index.replica_identity
    ));

    insta::assert_yaml_snapshot!("indexes_and_constraints", snapshot);
    Ok(())
}
