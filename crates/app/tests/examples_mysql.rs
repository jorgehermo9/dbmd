#[path = "examples/support/mod.rs"]
mod examples;

use std::collections::BTreeMap;

use dbmd_test_support::MysqlServer;
use examples::{load_suite, run_example, schema_sql};

#[test]
fn mysql_example_is_exact_fresh_deterministic_and_safe() {
    let suite = load_suite();
    let example = suite.example("backends/mysql");
    let server =
        MysqlServer::start(&schema_sql(example, "commerce").expect("MySQL schema should load"))
            .expect("MySQL example server should start");
    let environment = BTreeMap::from([("MYSQL_URL".to_string(), server.url().to_string())]);

    run_example(example, environment).expect("MySQL example should execute");
}
