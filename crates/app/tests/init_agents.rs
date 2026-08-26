use std::fs;

use dbmd_app::{init_agents, InitAgentsRequest};

#[test]
fn prints_agent_instructions_for_the_canonical_artifact() {
    let project = tempfile::tempdir().expect("temporary project should exist");
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.app]
backend = "postgres"
url = "${DATABASE_URL}"

[output]
path = "docs/DATABASE.md"
"#,
    )
    .expect("config should be written");

    let report = init_agents(InitAgentsRequest::new(config))
        .expect("source credentials are not needed to generate instructions");

    assert!(report.instructions.contains("docs/DATABASE.md"));
    assert!(report.instructions.contains("dbmd verify"));
    assert!(report.instructions.contains("Do not edit"));
    assert!(report
        .instructions
        .contains("Prefer that artifact for structural questions"));
    assert!(report.instructions.contains("When freshness is uncertain"));
    assert!(report.instructions.contains("Query a live database only"));
    assert!(report.written_path.is_none());
}

#[test]
fn preserves_output_environment_references_without_expanding_them() {
    let project = tempfile::tempdir().expect("temporary project should exist");
    let config = project.path().join("dbmd.toml");
    fs::write(
        &config,
        r#"
[sources.app]
backend = "postgres"
url = "${DATABASE_URL}"

[output]
path = "${OUTPUT_ROOT}/DATABASE.md"
"#,
    )
    .expect("config should be written");

    let report = init_agents(InitAgentsRequest::new(config))
        .expect("instructions should not expand environment values");

    assert!(report.instructions.contains("${OUTPUT_ROOT}/DATABASE.md"));
}

#[test]
fn rejects_inline_duplicate_and_reversed_markers_without_modifying_the_file() {
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
    let cases = [
        "Keep `<!-- dbmd:begin -->` and `<!-- dbmd:end -->` documented.\n",
        "<!-- dbmd:begin -->\nmissing end\n",
        "missing begin\n<!-- dbmd:end -->\n",
        "<!-- dbmd:begin -->\n<!-- dbmd:begin -->\n<!-- dbmd:end -->\n",
        "<!-- dbmd:end -->\n<!-- dbmd:begin -->\n",
    ];

    for (index, original) in cases.into_iter().enumerate() {
        let path = project.path().join(format!("AGENTS-{index}.md"));
        fs::write(&path, original).expect("malformed fixture should be written");

        let error = init_agents(InitAgentsRequest::new(&config).with_file(&path))
            .expect_err("malformed markers must be rejected");

        assert!(error.to_string().contains("malformed dbmd marker block"));
        assert_eq!(
            fs::read_to_string(path).expect("rejected file should remain readable"),
            original
        );
    }
}

#[cfg(unix)]
#[test]
fn rejects_symlink_and_non_regular_instruction_destinations_without_touching_targets() {
    use std::os::unix::fs::symlink;

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
    let target = project.path().join("target.md");
    fs::write(&target, "user-owned\n").expect("symlink target should be written");
    let link = project.path().join("AGENTS-link.md");
    symlink(&target, &link).expect("instruction symlink should be created");
    let directory = project.path().join("AGENTS-directory.md");
    fs::create_dir(&directory).expect("non-regular destination should be created");

    for path in [&link, &directory] {
        let error = init_agents(InitAgentsRequest::new(&config).with_file(path))
            .expect_err("unsafe instruction destination must be rejected");
        assert!(
            error
                .to_string()
                .contains("must be a regular non-symlink file"),
            "{error}"
        );
    }
    assert_eq!(
        fs::read_to_string(target).expect("symlink target should remain readable"),
        "user-owned\n"
    );
}

#[test]
fn explicit_file_update_preserves_unrelated_content_and_is_idempotent() {
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
    let agents = project.path().join("AGENTS.md");
    fs::write(&agents, "# Existing guidance\n\nKeep this.\n")
        .expect("existing instructions should be written");

    let first = init_agents(InitAgentsRequest::new(&config).with_file(&agents))
        .expect("marked instructions should append safely");
    let first_contents = fs::read_to_string(&agents).expect("instructions should be readable");
    let second = init_agents(InitAgentsRequest::new(&config).with_file(&agents))
        .expect("second initialization should be idempotent");

    assert!(first.changed);
    assert!(!second.changed);
    assert!(first_contents.starts_with("# Existing guidance\n\nKeep this.\n"));
    assert_eq!(first_contents.matches("<!-- dbmd:begin -->").count(), 1);
    assert_eq!(
        fs::read_to_string(&agents).expect("instructions should remain readable"),
        first_contents
    );
}

#[cfg(unix)]
#[test]
fn explicit_file_update_preserves_existing_permissions() {
    use std::os::unix::fs::PermissionsExt;

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
    let agents = project.path().join("AGENTS.md");
    fs::write(&agents, "# Existing\n").expect("existing instructions should be written");
    fs::set_permissions(&agents, fs::Permissions::from_mode(0o640))
        .expect("fixture permissions should be set");

    init_agents(InitAgentsRequest::new(&config).with_file(&agents))
        .expect("instruction update should succeed");

    assert_eq!(
        fs::metadata(agents)
            .expect("updated file should exist")
            .permissions()
            .mode()
            & 0o777,
        0o640
    );
}
