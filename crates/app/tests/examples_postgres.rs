#[path = "examples/support/mod.rs"]
mod examples;

use std::collections::BTreeMap;

use dbmd_test_support::PostgresServer;
use examples::{load_suite, run_example, schema_sql};

#[test]
fn postgres_example_is_exact_fresh_deterministic_and_safe() {
    let suite = load_suite();
    let example = suite.example("backends/postgres");
    let server = PostgresServer::start_initialized(
        "dbmd",
        "dbmd",
        "dbmd-example",
        &schema_sql(example, "catalog").expect("PostgreSQL schema should load"),
    )
    .expect("PostgreSQL example server should start");
    let environment = BTreeMap::from([(
        "POSTGRES_URL".to_string(),
        server.initial_connection_string(),
    )]);

    let result = run_example(example, environment);
    postgres_example_cleanup(&server);
    result.expect("PostgreSQL example should execute");
}

fn postgres_example_cleanup(server: &PostgresServer) {
    server
        .connect()
        .expect("PostgreSQL example database should reconnect for cleanup")
        .batch_execute("DROP SUBSCRIPTION advanced_subscription")
        .expect("disconnected example subscription should be dropped before its database");
}
