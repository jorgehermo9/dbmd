mod support;

use std::{collections::BTreeMap, fs};

use dbmd_app::{explain, DirectoryVariant, ExplainDestination, ExplainRequest, TemplateSource};
use dbmd_backends::Backend;
use support::TestProject;

#[test]
fn explains_resolution_without_connecting_or_exposing_credentials() {
    let project = TestProject::new();
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.local]
backend = "sqlite"
path = "missing.db"

[sources.production]
backend = "postgres"
url = "${DATABASE_URL}"

[output]
path = "DATABASE.md"
sources = ["production", "local"]
"#,
    )
    .expect("config should be written");
    let report = explain(ExplainRequest::with_environment(
        &config,
        BTreeMap::from([(
            "DATABASE_URL".to_string(),
            "postgres://secret:password@database/app".to_string(),
        )]),
    ))
    .expect("local resolution should not connect");

    assert_eq!(
        report
            .sources
            .iter()
            .map(|source| (source.id.as_str(), source.backend))
            .collect::<Vec<_>>(),
        [
            ("production", Backend::Postgres),
            ("local", Backend::Sqlite)
        ]
    );
    assert_eq!(report.required_environment, ["DATABASE_URL"]);
    assert_eq!(
        report.destination,
        ExplainDestination::Filesystem {
            display_path: project.path().join("DATABASE.md")
        }
    );
    assert_eq!(report.template_source, TemplateSource::Embedded);
    assert!(!format!("{report:?}").contains("password"));
}

#[test]
fn explains_every_composed_backend_without_connecting() {
    let project = TestProject::new();
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.clicks]
backend = "clickhouse"
url = "${CLICKHOUSE_URL}"
database = "analytics"

[sources.duck]
backend = "duckdb"
path = "warehouse.duckdb"

[sources.maria]
backend = "mariadb"
url = "${MARIADB_URL}"
schema = "${MARIADB_SCHEMA}"

[sources.mysql]
backend = "mysql"
url = "${MYSQL_URL}"
schema = "${MYSQL_SCHEMA}"

[output]
path = "DATABASE.md"
sources = ["clicks", "duck", "maria", "mysql"]
"#,
    )
    .expect("config should be written");
    let report = explain(ExplainRequest::with_environment(
        &config,
        BTreeMap::from([
            (
                "CLICKHOUSE_URL".to_string(),
                "http://database:8123".to_string(),
            ),
            (
                "MARIADB_URL".to_string(),
                "mysql://database/app".to_string(),
            ),
            ("MARIADB_SCHEMA".to_string(), "app".to_string()),
            ("MYSQL_URL".to_string(), "mysql://database/app".to_string()),
            ("MYSQL_SCHEMA".to_string(), "app".to_string()),
        ]),
    ))
    .expect("every backend config should resolve locally");

    assert_eq!(
        report
            .sources
            .iter()
            .map(|source| (source.id.as_str(), source.backend))
            .collect::<Vec<_>>(),
        [
            ("clicks", Backend::Clickhouse),
            ("duck", Backend::Duckdb),
            ("maria", Backend::Mariadb),
            ("mysql", Backend::Mysql),
        ]
    );
    assert_eq!(
        report.required_environment,
        [
            "CLICKHOUSE_URL",
            "MARIADB_SCHEMA",
            "MARIADB_URL",
            "MYSQL_SCHEMA",
            "MYSQL_URL",
        ]
    );
}

#[test]
fn resolves_every_backend_specific_optional_field_without_exposing_values() {
    let project = TestProject::new();
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.clicks]
backend = "clickhouse"
url = "${CLICKHOUSE_URL}"
database = "${CLICKHOUSE_DATABASE}"
username = "${CLICKHOUSE_USERNAME}"
password = "${CLICKHOUSE_PASSWORD}"
display_name = "Click analytics"

[sources.duck]
backend = "duckdb"
path = "${DUCKDB_PATH}"
secret_directory = "${DUCKDB_SECRETS}"
extension_directory = "${DUCKDB_EXTENSIONS}"
display_name = "Embedded analytics"

[sources.duck.attachments.raw]
path = "${DUCKDB_ATTACHMENT}"
read_only = false

[sources.maria]
backend = "mariadb"
url = "${MARIADB_URL}"
schema = "${MARIADB_SCHEMA}"
include_global_objects = true

[sources.mysql]
backend = "mysql"
url = "${MYSQL_URL}"
schema = "${MYSQL_SCHEMA}"
include_global_objects = true

[sources.postgres]
backend = "postgres"
url = "${POSTGRES_URL}"
include_cluster_objects = true

[sources.sqlite]
backend = "sqlite"
path = "${SQLITE_PATH}"

[sources.sqlite.attachments.analytics]
path = "${SQLITE_ATTACHMENT}"

[output]
path = "DATABASE.md"
sources = ["clicks", "duck", "maria", "mysql", "postgres", "sqlite"]
"#,
    )
    .expect("config should be written");
    let names = [
        "CLICKHOUSE_URL",
        "CLICKHOUSE_DATABASE",
        "CLICKHOUSE_USERNAME",
        "CLICKHOUSE_PASSWORD",
        "DUCKDB_PATH",
        "DUCKDB_SECRETS",
        "DUCKDB_EXTENSIONS",
        "DUCKDB_ATTACHMENT",
        "MARIADB_URL",
        "MARIADB_SCHEMA",
        "MYSQL_URL",
        "MYSQL_SCHEMA",
        "POSTGRES_URL",
        "SQLITE_PATH",
        "SQLITE_ATTACHMENT",
    ];
    let environment = names
        .iter()
        .map(|name| ((*name).to_string(), format!("sentinel-value-{name}")))
        .collect::<BTreeMap<_, _>>();

    let report = explain(ExplainRequest::with_environment(&config, environment))
        .expect("all backend-specific fields should resolve locally");
    let debug = format!("{report:?}");

    assert_eq!(
        report
            .sources
            .iter()
            .map(|source| source.backend)
            .collect::<Vec<_>>(),
        [
            Backend::Clickhouse,
            Backend::Duckdb,
            Backend::Mariadb,
            Backend::Mysql,
            Backend::Postgres,
            Backend::Sqlite,
        ]
    );
    assert_eq!(report.required_environment.len(), names.len());
    for name in names {
        assert!(report.required_environment.iter().any(|item| item == name));
        assert!(!debug.contains(&format!("sentinel-value-{name}")));
    }
}

#[test]
fn rejects_backend_specific_attachment_invariants_during_local_resolution() {
    let project = TestProject::new();
    let cases = [
        (
            "SQLite reserved namespace",
            r#"
[sources.app]
backend = "sqlite"
path = "app.db"
[sources.app.attachments.main]
path = "other.db"
[output]
path = "DATABASE.md"
"#,
            "SQLite namespace `main` is reserved",
        ),
        (
            "DuckDB reserved attachment",
            r#"
[sources.app]
backend = "duckdb"
path = "app.duckdb"
[sources.app.attachments.temp]
path = "other.duckdb"
[output]
path = "DATABASE.md"
"#,
            "DuckDB attachment name `temp` is reserved",
        ),
    ];

    for (case, contents, expected) in cases {
        let config = project
            .path()
            .join(format!("{}.toml", case.replace(' ', "-")));
        fs::write(&config, contents).expect("invalid config fixture should be written");

        let error = explain(ExplainRequest::new(config)).expect_err(case);

        assert!(error.to_string().contains(expected), "{case}: {error}");
    }
}

#[test]
fn explains_render_overrides_and_known_single_file_output() {
    let project = TestProject::new();
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.first]
backend = "sqlite"
path = "first.db"

[sources.second]
backend = "sqlite"
path = "second.db"

[output]
path = "DATABASE.md"
sources = ["first"]
"#,
    )
    .expect("config should be written");

    let report = explain(
        ExplainRequest::new(&config)
            .with_sources(["second"])
            .with_output_path("alternate/SCHEMA.md")
            .with_template_root("templates/dbmd"),
    )
    .expect("overridden plan should resolve");

    assert_eq!(report.sources[0].id.as_str(), "second");
    assert_eq!(
        report.canonical_output_display_path,
        project.path().join("DATABASE.md")
    );
    assert_eq!(
        report.destination,
        ExplainDestination::Filesystem {
            display_path: project.path().join("alternate/SCHEMA.md")
        }
    );
    assert!(report.output_overridden);
    assert_eq!(
        report.planned_display_files,
        Some(vec![project.path().join("alternate/SCHEMA.md")])
    );
    assert_eq!(
        report.template_source,
        TemplateSource::Custom {
            display_root: project.path().join("templates/dbmd")
        }
    );
    assert_eq!(
        report.required_template_entrypoints.len(),
        dbmd_render::embedded_template_files().len() + dbmd_backends::all_template_files().len()
    );
}

#[test]
fn preserves_environment_names_in_display_paths_without_exposing_values() {
    let project = TestProject::new();
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.app]
backend = "sqlite"
path = "app.db"

[output]
path = "${OUTPUT_SECRET}/DATABASE.md"

[templates]
dir = "${TEMPLATE_SECRET}/dbmd"
"#,
    )
    .expect("config should be written");
    let report = explain(ExplainRequest::with_environment(
        config,
        BTreeMap::from([
            ("OUTPUT_SECRET".to_string(), "private-output".to_string()),
            (
                "TEMPLATE_SECRET".to_string(),
                "private-template".to_string(),
            ),
        ]),
    ))
    .expect("plan should resolve");
    let debug = format!("{report:?}");

    assert!(debug.contains("${OUTPUT_SECRET}"));
    assert!(debug.contains("${TEMPLATE_SECRET}"));
    assert!(!debug.contains("private-output"));
    assert!(!debug.contains("private-template"));
}

#[test]
fn reports_only_effective_destination_changes_as_overrides() {
    let project = TestProject::new();
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.app]
backend = "sqlite"
path = "app.db"

[output]
path = "DATABASE.md"
"#,
    )
    .expect("config should be written");

    let report = explain(ExplainRequest::new(config).with_output_path("DATABASE.md"))
        .expect("equivalent destination should resolve");

    assert!(!report.output_overridden);
}

#[test]
fn reports_the_implemented_directory_variant() {
    let project = TestProject::new();
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.app]
backend = "sqlite"
path = "app.db"

[output]
path = "database"

[output.layout]
kind = "directory"
"#,
    )
    .expect("config should be written");

    let report = explain(ExplainRequest::new(config)).expect("directory plan should resolve");

    assert_eq!(report.directory_variant, Some(DirectoryVariant::Objects));
    assert!(report.planned_display_files.is_none());
}

#[test]
fn request_debug_lists_environment_names_but_not_values() {
    let request = ExplainRequest::with_environment(
        "dbmd.toml",
        BTreeMap::from([("DATABASE_URL".to_string(), "sentinel-secret".to_string())]),
    );

    let debug = format!("{request:?}");

    assert!(debug.contains("DATABASE_URL"));
    assert!(!debug.contains("sentinel-secret"));
}
