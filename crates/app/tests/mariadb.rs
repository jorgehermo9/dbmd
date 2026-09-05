mod support;

use dbmd_app::{render, RenderOutput, RenderRequest};
use dbmd_test_support::MariaDbServer;
use support::TestProject;

#[test]
fn renders_mariadb_through_the_application_operation() {
    let server = MariaDbServer::start(include_str!(
        "../../backends/mariadb/tests/fixtures/schema_surface.sql"
    ))
    .expect("MariaDB fixture should start");
    let project = TestProject::new();
    let config = project.config(format!(
        r#"[sources.commerce]
backend = "mariadb"
url = "{}"
schema = "test"

[output]
path = "DATABASE.md"
"#,
        server.url()
    ));

    let report = render(RenderRequest::new(config).to_stdout())
        .expect("MariaDB application render should succeed");
    let RenderOutput::Stdout(markdown) = report.output else {
        panic!("stdout render should return Markdown bytes");
    };
    let markdown = String::from_utf8(markdown).expect("Markdown should be UTF-8");
    assert!(markdown.contains("System versioning"));
    assert!(markdown.contains("order_number_seq"));
}
