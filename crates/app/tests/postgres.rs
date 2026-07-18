use std::{collections::BTreeMap, fs};

use dbmd_app::RenderRequest;
use dbmd_test_support::{run_postgres_cases, PostgresCase, PostgresServer, TestResult};

const CASES: &[PostgresCase] = &[PostgresCase {
    name: "render",
    run: renders_a_postgres_source_through_the_application_api,
}];

#[test]
fn postgres_application_fixtures() {
    run_postgres_cases(CASES);
}

fn renders_a_postgres_source_through_the_application_api(server: &PostgresServer) -> TestResult {
    let database = server.database(include_str!("fixtures/postgres/render/schema.sql"))?;
    let project = tempfile::tempdir()?;
    let config_path = project.path().join("dbmd.toml");
    fs::write(
        &config_path,
        r#"
[sources.catalog]
backend = "postgres"
url = "${DATABASE_URL}"

[output]
path = "DATABASE.md"
"#,
    )?;
    let environment = BTreeMap::from([("DATABASE_URL".to_string(), database.connection_string())]);

    let request = || RenderRequest::with_environment(&config_path, environment.clone());
    let report = dbmd_app::render(request())?;
    let markdown = fs::read_to_string(
        report
            .output
            .path()
            .expect("PostgreSQL render should write its configured path"),
    )?;
    let second_report = dbmd_app::render(request())?;
    let second_markdown = fs::read_to_string(
        second_report
            .output
            .path()
            .expect("second PostgreSQL render should write its configured path"),
    )?;

    assert_eq!(report, second_report);
    assert_eq!(markdown, second_markdown);
    assert_eq!(report.sources.len(), 1);
    assert_eq!(report.sources[0].as_str(), "catalog");
    assert!(markdown.contains("catalog.accounts"));
    assert!(markdown.contains("Application accounts"));
    assert!(markdown.contains("catalog.active_accounts"));
    assert!(markdown.contains("identity `always`"));
    assert!(markdown.contains("catalog.account_state"));
    assert!(markdown.contains("active, suspended"));
    insta::assert_snapshot!("postgres_render", markdown);
    Ok(())
}
