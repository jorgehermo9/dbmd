mod support;

use dbmd_app::{render, RenderOutput, RenderRequest};
use dbmd_test_support::MysqlServer;
use support::TestProject;

#[test]
fn renders_mysql_through_the_application_operation() {
    let server = MysqlServer::start(include_str!(
        "../../backends/mysql/tests/fixtures/schema_surface.sql"
    ))
    .expect("MySQL fixture should start");
    let markdown = render_fixture("mysql", server.url());
    assert!(markdown.contains("accounts_normalized_idx"));
    assert!(markdown.contains("purge_disabled_accounts"));
}

fn render_fixture(backend: &str, url: &str) -> String {
    let project = TestProject::new();
    let config = project.config(format!(
        r#"[sources.commerce]
backend = "{backend}"
url = "{url}"
schema = "test"

[output]
path = "DATABASE.md"
"#
    ));
    let report = render(RenderRequest::new(config).to_stdout())
        .expect("MySQL-family application render should succeed");
    let RenderOutput::Stdout(markdown) = report.output else {
        panic!("stdout render should return Markdown bytes");
    };
    String::from_utf8(markdown).expect("Markdown should be UTF-8")
}
