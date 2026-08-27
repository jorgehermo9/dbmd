//! Temporary project fixtures stay isolated, path-safe, and self-cleaning.

use std::{fs, panic};

use dbmd_test_support::TestProject;
use rstest::rstest;

#[test]
fn creates_distinct_existing_project_roots() {
    let first = TestProject::new();
    let second = TestProject::new();

    assert!(first.root().is_dir());
    assert!(second.root().is_dir());
    assert_ne!(first.root(), second.root());
}

#[test]
fn writes_exact_bytes_and_creates_missing_parents() {
    let project = TestProject::new();

    let path = project.write("nested/fixture.txt", b"fixture bytes\n");

    assert_eq!(
        fs::read(path).expect("fixture should remain readable"),
        b"fixture bytes\n"
    );
}

#[rstest]
#[case::absolute("/outside")]
#[case::parent_traversal("../outside")]
#[case::nested_parent_traversal("nested/../../outside")]
fn rejects_paths_that_can_escape_the_project(#[case] relative: &str) {
    let project = TestProject::new();

    let result = panic::catch_unwind(|| project.write(relative, b"unsafe"));

    assert!(result.is_err());
}

#[test]
fn removes_the_project_tree_when_dropped() {
    let root = {
        let project = TestProject::new();
        let root = project.root().to_path_buf();
        project.write("fixture.txt", b"fixture");
        root
    };

    assert!(!root.exists());
}
