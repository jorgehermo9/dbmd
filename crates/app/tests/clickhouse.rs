use std::fs;

use dbmd_app::{render, RenderOutput, RenderRequest};
use dbmd_test_support::ClickHouseServer;

#[test]
fn renders_clickhouse_through_the_application_operation() {
    let server = ClickHouseServer::start_with_settings(
        include_str!("../../backends/clickhouse/tests/fixtures/schema_surface.sql"),
        &[
            ("allow_experimental_codecs", "1"),
            ("allow_experimental_window_view", "1"),
            ("allow_experimental_analyzer", "0"),
        ],
    )
    .expect("ClickHouse fixture should start");
    let project = tempfile::tempdir().expect("temporary project should exist");
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        format!(
            r#"[sources.analytics]
backend = "clickhouse"
url = "{}"
database = "analytics"

[output]
path = "DATABASE.md"
"#,
            server.endpoint()
        ),
    )
    .expect("config should be written");

    let report = render(RenderRequest::new(config).to_stdout())
        .expect("ClickHouse application render should succeed");
    let RenderOutput::Stdout(markdown) = report.output else {
        panic!("stdout render should return Markdown bytes");
    };
    let markdown = String::from_utf8(markdown).expect("Markdown should be UTF-8");
    assert!(markdown.contains("ReplacingMergeTree"));
    assert!(markdown.contains("analytics_normalize"));
}
