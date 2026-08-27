#[path = "examples/support/mod.rs"]
mod examples;

use std::collections::BTreeMap;

use dbmd_test_support::MariaDbServer;
use examples::{load_suite, run_example, schema_sql};

#[test]
fn mariadb_example_is_exact_fresh_deterministic_and_safe() {
    let suite = load_suite();
    let example = suite.example("backends/mariadb");
    let server =
        MariaDbServer::start(&schema_sql(example, "commerce").expect("MariaDB schema should load"))
            .expect("MariaDB example server should start");
    let environment = BTreeMap::from([("MARIADB_URL".to_string(), server.url().to_string())]);

    run_example(example, environment).expect("MariaDB example should execute");
}
