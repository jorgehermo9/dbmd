mod support;

use std::fs;

use dbmd_app::{init_ci, InitCiRequest};
use support::TestProject;

#[test]
fn initializes_a_pinned_github_actions_verify_workflow() {
    let project = TestProject::new();
    let path = project.path().join(".github/workflows/dbmd.yml");

    let report = init_ci(InitCiRequest::new(&path)).expect("CI workflow should initialize");
    let workflow = fs::read_to_string(&path).expect("workflow should exist");

    assert_eq!(report.workflow_path, path);
    assert!(workflow.contains("uses: actions/checkout@v4"));
    assert!(workflow.contains("cargo install dbmd --locked --version 0.1.0"));
    assert!(workflow.contains("run: dbmd verify"));
    assert!(workflow.contains("DATABASE_URL: ${{ secrets.DATABASE_URL }}"));
}

#[test]
fn ci_initialization_requires_explicit_overwrite_for_an_existing_workflow() {
    let project = TestProject::new();
    let path = project.path().join("dbmd.yml");
    fs::write(&path, "user owned\n").expect("existing workflow should be written");

    let error = init_ci(InitCiRequest::new(&path))
        .expect_err("existing workflow must be preserved by default");
    assert!(error.to_string().contains("already exists"));
    assert_eq!(
        fs::read_to_string(&path).expect("workflow should remain"),
        "user owned\n"
    );

    init_ci(InitCiRequest::new(&path).with_overwrite(true))
        .expect("explicit overwrite should replace the workflow");
    assert!(fs::read_to_string(path)
        .expect("replacement workflow should exist")
        .contains("dbmd verify"));
}
