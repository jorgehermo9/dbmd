mod support;

use std::{collections::BTreeMap, fs};

use dbmd_app::{render, RenderOutput, RenderRequest};
use support::TestProject;

const CONFIG: &str = include_str!("fixtures/sqlite/full_schema/dbmd.toml");
const SCHEMA: &str = include_str!("fixtures/sqlite/full_schema/schema.sql");
const ANALYTICS_SCHEMA: &str = include_str!("fixtures/sqlite/full_schema/analytics.sql");
const MULTI_SOURCE_CONFIG: &str = include_str!("fixtures/sqlite/multi_source/dbmd.toml");
const DIRECTORY_CONFIG: &str = include_str!("fixtures/sqlite/directory/dbmd.toml");

#[test]
fn renders_the_complete_sqlite_schema_surface_deterministically() {
    let project = TestProject::from_fixture(CONFIG, SCHEMA, ANALYTICS_SCHEMA);

    let first_report = render(project.request()).expect("first render should succeed");
    let first = fs::read_to_string(project.output_path()).expect("artifact should exist");
    let second_report = render(project.request()).expect("second render should succeed");
    let second = fs::read_to_string(project.output_path()).expect("artifact should still exist");

    assert_eq!(first, second);
    assert_eq!(first_report, second_report);
    assert_eq!(first_report.output.bytes_written(), first.len());
    insta::assert_snapshot!("full_sqlite_render", first);
}

#[test]
fn preserves_the_previous_artifact_when_introspection_fails() {
    let project = TestProject::from_fixture(CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    fs::write(project.output_path(), "previous artifact\n")
        .expect("old artifact should be written");
    let request = RenderRequest::with_environment(
        project.path().join("dbmd.toml"),
        BTreeMap::from([
            (
                "DBMD_TEST_DATABASE".to_string(),
                project
                    .path()
                    .join("missing.db")
                    .to_string_lossy()
                    .into_owned(),
            ),
            (
                "DBMD_TEST_ANALYTICS_DATABASE".to_string(),
                project
                    .path()
                    .join("analytics.db")
                    .to_string_lossy()
                    .into_owned(),
            ),
        ]),
    );

    let error = render(request).expect_err("missing database should fail introspection");

    assert!(error.to_string().contains("failed to open SQLite source"));
    assert_eq!(
        fs::read_to_string(project.output_path()).expect("old artifact should remain"),
        "previous artifact\n"
    );
}

#[test]
fn renders_selected_sources_in_configured_order_with_source_sections() {
    let project = TestProject::from_fixture(MULTI_SOURCE_CONFIG, SCHEMA, ANALYTICS_SCHEMA);

    let report = render(project.request()).expect("multiple SQLite sources should render");
    let markdown = fs::read_to_string(project.output_path()).expect("artifact should exist");

    assert_eq!(
        report
            .sources
            .iter()
            .map(dbmd_core::SourceId::as_str)
            .collect::<Vec<_>>(),
        ["analytics", "app"]
    );
    insta::assert_snapshot!("multiple_sqlite_sources", markdown);
}

#[test]
fn request_source_override_replaces_config_selection_and_preserves_order() {
    let project = TestProject::from_fixture(MULTI_SOURCE_CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    let request = project.request().with_sources(["app", "analytics"]);

    let report = render(request).expect("request source override should render");

    assert_eq!(
        report
            .sources
            .iter()
            .map(dbmd_core::SourceId::as_str)
            .collect::<Vec<_>>(),
        ["app", "analytics"]
    );
}

#[test]
fn atomically_renders_a_directory_artifact_without_stale_files() {
    let project = TestProject::from_fixture(DIRECTORY_CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    let output = project.path().join("database");
    fs::create_dir_all(output.join("tables")).expect("old artifact tree should be created");
    fs::write(output.join("tables/stale.md"), "stale\n").expect("stale artifact should be created");

    let report = render(project.request()).expect("directory artifact should render");
    let index = fs::read_to_string(output.join("index.md")).expect("index should exist");
    let table = fs::read_to_string(output.join("tables/main.accounts.md"))
        .expect("table artifact should exist");

    assert_eq!(report.output.path(), Some(output.as_path()));
    assert!(!output.join("tables/stale.md").exists());
    insta::assert_snapshot!("directory_index", index);
    insta::assert_snapshot!("directory_table", table);
}

#[test]
fn configured_stdout_returns_single_file_without_writing_the_canonical_artifact() {
    let project = TestProject::from_fixture(CONFIG, SCHEMA, ANALYTICS_SCHEMA);

    let report = render(project.request().to_stdout()).expect("stdout render should succeed");

    let RenderOutput::Stdout(contents) = report.output else {
        panic!("stdout request should return in-memory output");
    };
    let markdown = String::from_utf8(contents).expect("rendered Markdown should be UTF-8");
    assert!(markdown.contains("`main.accounts`"));
    assert!(!project.output_path().exists());
}

#[test]
fn configless_sqlite_request_renders_without_reading_a_project_config() {
    let project = TestProject::from_fixture(CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    fs::remove_file(project.path().join("dbmd.toml")).expect("config should be removed");
    let output = project.path().join("ONE_OFF.md");
    let request = RenderRequest::sqlite(project.path().join("app.db")).with_output_path(&output);

    let report = render(request).expect("one-off SQLite render should succeed");

    assert_eq!(report.output.path(), Some(output.as_path()));
    assert!(fs::read_to_string(output)
        .expect("one-off artifact should exist")
        .contains("`main.accounts`"));
}

#[test]
fn output_override_does_not_require_environment_used_only_by_canonical_path() {
    let config = r#"
[sources.app]
backend = "sqlite"
path = "app.db"

[output]
path = "${CANONICAL_OUTPUT}"
"#;
    let project = TestProject::from_fixture(config, SCHEMA, ANALYTICS_SCHEMA);
    let output = project.path().join("ALTERNATE.md");
    let request =
        RenderRequest::with_environment(project.path().join("dbmd.toml"), BTreeMap::new())
            .with_output_path(&output);

    let report = render(request).expect("overridden canonical output should not be resolved");

    assert_eq!(report.output.path(), Some(output.as_path()));
}

#[test]
fn directory_output_refuses_to_replace_the_project_root() {
    let config = r#"
[sources.app]
backend = "sqlite"
path = "${DBMD_TEST_DATABASE}"

[output]
path = "."

[output.layout]
kind = "directory"
"#;
    let project = TestProject::from_fixture(config, SCHEMA, ANALYTICS_SCHEMA);
    let marker = project.path().join("user-owned.txt");
    fs::write(&marker, "preserve me\n").expect("marker should be written");

    let error = render(project.request()).expect_err("project root output must be rejected");

    assert!(error.to_string().contains("unsafe artifact output path"));
    assert_eq!(
        fs::read_to_string(marker).expect("project contents should remain"),
        "preserve me\n"
    );
}

#[test]
fn nested_config_refuses_to_replace_the_discovered_repository_root() {
    let project = TestProject::from_fixture(CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    fs::create_dir(project.path().join(".git")).expect("repository marker should be created");
    let nested = project.path().join("config");
    fs::create_dir(&nested).expect("nested config directory should be created");
    let config_path = nested.join("dbmd.toml");
    fs::write(
        &config_path,
        format!(
            r#"
[sources.app]
backend = "sqlite"
path = "{}"

[output]
path = "{}"

[output.layout]
kind = "directory"
"#,
            project.path().join("app.db").display(),
            project.path().display()
        ),
    )
    .expect("nested config should be written");
    let marker = project.path().join("user-owned.txt");
    fs::write(&marker, "preserve me\n").expect("marker should be written");

    let error = render(RenderRequest::with_environment(
        config_path,
        BTreeMap::new(),
    ))
    .expect_err("repository root output must be rejected");

    assert!(error.to_string().contains("unsafe artifact output path"));
    assert_eq!(
        fs::read_to_string(marker).expect("repository should remain"),
        "preserve me\n"
    );
}

#[test]
fn directory_output_refuses_paths_inside_git_metadata() {
    let config = r#"
[sources.app]
backend = "sqlite"
path = "${DBMD_TEST_DATABASE}"

[output]
path = ".git/generated-schema"

[output.layout]
kind = "directory"
"#;
    let project = TestProject::from_fixture(config, SCHEMA, ANALYTICS_SCHEMA);

    let error = render(project.request()).expect_err(".git output must be rejected");

    assert!(error.to_string().contains("unsafe artifact output path"));
    assert!(!project.path().join(".git/generated-schema").exists());
}

#[cfg(unix)]
#[test]
fn directory_output_refuses_a_symlink_root_without_touching_its_target() {
    use std::os::unix::fs::symlink;

    let project = TestProject::from_fixture(DIRECTORY_CONFIG, SCHEMA, ANALYTICS_SCHEMA);
    let target = project.path().join("user-owned");
    fs::create_dir(&target).expect("symlink target should be created");
    fs::write(target.join("marker.txt"), "preserve me\n").expect("marker should be written");
    symlink(&target, project.path().join("database")).expect("output symlink should be created");

    let error = render(project.request()).expect_err("symlink output root must be rejected");

    assert!(error.to_string().contains("unsafe artifact output path"));
    assert_eq!(
        fs::read_to_string(target.join("marker.txt")).expect("target should remain readable"),
        "preserve me\n"
    );
}

#[cfg(unix)]
#[test]
fn directory_output_refuses_a_symlinked_ancestor() {
    use std::os::unix::fs::symlink;

    let config = r#"
[sources.app]
backend = "sqlite"
path = "${DBMD_TEST_DATABASE}"

[output]
path = "linked/generated"

[output.layout]
kind = "directory"
"#;
    let project = TestProject::from_fixture(config, SCHEMA, ANALYTICS_SCHEMA);
    let target = project.path().join("user-owned");
    fs::create_dir(&target).expect("symlink target should be created");
    fs::write(target.join("marker.txt"), "preserve me\n").expect("marker should be written");
    symlink(&target, project.path().join("linked")).expect("ancestor symlink should be created");

    let error = render(project.request()).expect_err("symlinked ancestor must be rejected");

    assert!(error.to_string().contains("unsafe artifact output path"));
    assert!(!target.join("generated").exists());
}

#[test]
fn configured_custom_template_profile_controls_rendering() {
    let config = r#"
[sources.app]
backend = "sqlite"
path = "${DBMD_TEST_DATABASE}"

[output]
path = "DATABASE.md"
profile = "agent"

[templates]
dir = "templates/dbmd"
"#;
    let project = TestProject::from_fixture(config, SCHEMA, ANALYTICS_SCHEMA);
    let root = project.path().join("templates/dbmd");
    write_complete_template_profile(&root);
    fs::write(
        root.join("agent/single_file/database.md.j2"),
        "# Project-owned template for `{{ context.sources[0].id }}`\n",
    )
    .expect("custom entrypoint should be replaced");

    render(project.request()).expect("complete custom profile should render");

    assert_eq!(
        fs::read_to_string(project.output_path()).expect("custom output should exist"),
        "# Project-owned template for `app`"
    );
}

#[test]
fn custom_template_preflight_fails_before_database_introspection() {
    let config = r#"
[sources.app]
backend = "sqlite"
path = "missing.db"

[output]
path = "DATABASE.md"

[templates]
dir = "templates/dbmd"
"#;
    let project = TestProject::from_fixture(config, SCHEMA, ANALYTICS_SCHEMA);
    let entrypoint = project
        .path()
        .join("templates/dbmd/agent/single_file/database.md.j2");
    fs::create_dir_all(
        entrypoint
            .parent()
            .expect("entrypoint should have a parent"),
    )
    .expect("template directory should be created");
    fs::write(entrypoint, "# Incomplete\n").expect("template should be written");

    let error = render(project.request()).expect_err("incomplete profile should fail preflight");

    assert!(error.to_string().contains("directory/enum.md.j2"));
    assert!(!error.to_string().contains("failed to open SQLite source"));
}

#[test]
fn stdout_layout_preflight_fails_before_database_introspection() {
    let config = r#"
[sources.app]
backend = "sqlite"
path = "missing.db"

[output]
path = "database"

[output.layout]
kind = "directory"
"#;
    let project = TestProject::from_fixture(config, SCHEMA, ANALYTICS_SCHEMA);

    let error = render(project.request().to_stdout())
        .expect_err("directory stdout should fail during local preflight");

    assert!(error.to_string().contains("stdout is available only"));
    assert!(!error.to_string().contains("failed to open SQLite source"));
}

#[test]
fn unsafe_output_preflight_fails_before_database_introspection() {
    let config = r#"
[sources.app]
backend = "sqlite"
path = "missing.db"

[output]
path = "."

[output.layout]
kind = "directory"
"#;
    let project = TestProject::from_fixture(config, SCHEMA, ANALYTICS_SCHEMA);

    let error = render(project.request()).expect_err("unsafe output should fail local preflight");

    assert!(error.to_string().contains("unsafe artifact output path"));
    assert!(!error.to_string().contains("failed to open SQLite source"));
}

fn write_complete_template_profile(root: &std::path::Path) {
    let backend_templates = dbmd_backends::all_template_files();
    for file in dbmd_render::embedded_template_files()
        .iter()
        .chain(&backend_templates)
    {
        let path = root.join("agent").join(file.relative_path);
        fs::create_dir_all(path.parent().expect("template should have a parent"))
            .expect("template directory should be created");
        fs::write(path, file.contents).expect("template should be written");
    }
}
