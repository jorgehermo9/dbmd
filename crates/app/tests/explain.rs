use std::{collections::BTreeMap, fs};

use dbmd_app::{explain, DirectoryVariant, ExplainDestination, ExplainRequest, TemplateSource};
use dbmd_backends::Backend;

#[test]
fn explains_resolution_without_connecting_or_exposing_credentials() {
    let project = tempfile::tempdir().expect("temporary project should exist");
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
fn explains_render_overrides_and_known_single_file_output() {
    let project = tempfile::tempdir().expect("temporary project should exist");
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
    let project = tempfile::tempdir().expect("temporary project should exist");
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
    let project = tempfile::tempdir().expect("temporary project should exist");
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
    let project = tempfile::tempdir().expect("temporary project should exist");
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
