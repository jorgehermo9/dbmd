#[path = "examples/support/mod.rs"]
mod examples;

use std::collections::BTreeMap;

use dbmd_test_support::{ClickHouseServer, MariaDbServer, MysqlServer, PostgresServer};
use examples::{load_suite, run_example, schema_sql};

#[test]
fn full_example_composes_all_backends_exactly_and_safely() {
    let suite = load_suite();
    let example = suite.example("full");

    let postgres = PostgresServer::start_initialized(
        "dbmd",
        "dbmd",
        "dbmd-example",
        &schema_sql(example, "postgres_app").expect("PostgreSQL schema should load"),
    )
    .expect("PostgreSQL server should start");
    let clickhouse_sql =
        schema_sql(example, "clickhouse_events").expect("ClickHouse schema should load");
    let clickhouse =
        ClickHouseServer::start(&clickhouse_sql).expect("ClickHouse server should start");
    let mysql = MysqlServer::start(
        &schema_sql(example, "mysql_commerce").expect("MySQL schema should load"),
    )
    .expect("MySQL server should start");
    let mariadb = MariaDbServer::start(
        &schema_sql(example, "mariadb_commerce").expect("MariaDB schema should load"),
    )
    .expect("MariaDB server should start");

    let environment = BTreeMap::from([
        (
            "POSTGRES_URL".to_string(),
            postgres.initial_connection_string(),
        ),
        (
            "CLICKHOUSE_URL".to_string(),
            clickhouse.endpoint().to_string(),
        ),
        ("MYSQL_URL".to_string(), mysql.url().to_string()),
        ("MARIADB_URL".to_string(), mariadb.url().to_string()),
    ]);

    let result = run_example(example, environment);
    postgres
        .connect()
        .expect("PostgreSQL full-example database should reconnect for cleanup")
        .batch_execute("DROP SUBSCRIPTION advanced_subscription")
        .expect("disconnected example subscription should be dropped before container cleanup");
    result.expect("full example should execute");
}
