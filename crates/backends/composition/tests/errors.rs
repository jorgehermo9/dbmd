use std::str::FromStr;

use dbmd_backend_clickhouse::ClickHouseSource;
use dbmd_backend_mariadb::MariaDbSource;
use dbmd_backend_mysql::MysqlSource;
use dbmd_backend_postgres::PostgresSource;
use dbmd_backends::{introspect, Source};
use dbmd_core::SourceId;

#[test]
fn composed_server_diagnostics_keep_source_identity_and_remove_credentials_from_error_chains() {
    let id = || SourceId::from_str("unavailable").expect("test source ID should be valid");
    let cases = [
        (
            "postgres",
            Source::from(PostgresSource::new(
                id(),
                "postgres://dbmd:sentinel-postgres@127.0.0.1:1/missing?connect_timeout=1",
            )),
            "sentinel-postgres",
        ),
        (
            "clickhouse",
            Source::from(
                ClickHouseSource::new(id(), "http://127.0.0.1:1").with_credentials(
                    Some("dbmd".to_string()),
                    Some("sentinel-clickhouse".to_string()),
                ),
            ),
            "sentinel-clickhouse",
        ),
        (
            "mysql",
            Source::from(MysqlSource::new(
                id(),
                "mysql://dbmd:sentinel-mysql@127.0.0.1:1/missing",
            )),
            "sentinel-mysql",
        ),
        (
            "mariadb",
            Source::from(MariaDbSource::new(
                id(),
                "mysql://dbmd:sentinel-mariadb@127.0.0.1:1/missing",
            )),
            "sentinel-mariadb",
        ),
    ];

    for (case, source, secret) in cases {
        let error = introspect(&source).expect_err("refused server connection should fail");
        let diagnostic = error.diagnostic();

        assert!(diagnostic.contains("`unavailable`"), "{case}: {diagnostic}");
        assert!(!diagnostic.contains(secret), "{case}: {diagnostic}");
    }
}
