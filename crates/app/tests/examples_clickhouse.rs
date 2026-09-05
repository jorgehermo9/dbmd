#[path = "examples/support/mod.rs"]
mod examples;

use std::collections::BTreeMap;

use dbmd_test_support::ClickHouseServer;
use examples::{load_suite, run_example, schema_sql};

#[test]
fn clickhouse_example_is_exact_fresh_deterministic_and_safe() {
    let suite = load_suite();
    let example = suite.example("backends/clickhouse");
    let sql = schema_sql(example, "analytics").expect("ClickHouse schema should load");
    let server = ClickHouseServer::start(&sql).expect("ClickHouse example server should start");
    let environment =
        BTreeMap::from([("CLICKHOUSE_URL".to_string(), server.endpoint().to_string())]);

    run_example(example, environment).expect("ClickHouse example should execute");
}
