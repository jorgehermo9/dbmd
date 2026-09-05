#[path = "examples/support/mod.rs"]
mod examples;

use std::{collections::BTreeMap, fs, path::Path};

use examples::{
    assert_inventory_conforms, assert_inventory_conforms_at, load_suite, run_example, Backend,
    Suite,
};

const VALID_MANIFEST: &str = r#"
version = 1
[[example]]
path = "registered"
[[example.source]]
id = "app"
backend = "sqlite"
schema_dir = "schema/app"
"#;

#[test]
fn valid_manifest_preserves_typed_backend_and_case_flags() {
    let suite = Suite::parse(VALID_MANIFEST).expect("valid suite should parse");
    let example = suite.example("registered");

    assert_eq!(example.source[0].backend, Backend::Sqlite);
    assert!(!example.cli);
    assert!(!example.drift_check);
}

#[test]
fn unknown_manifest_fields_are_rejected() {
    let error =
        Suite::parse(&VALID_MANIFEST.replace("version = 1", "version = 1\nmode = \"hidden\""))
            .expect_err("unknown fields should fail");

    assert!(error.to_string().contains("unknown field `mode`"));
}

#[test]
fn duplicate_example_paths_are_rejected() {
    let duplicate = format!(
        "{VALID_MANIFEST}\n{}",
        VALID_MANIFEST.trim_start_matches("\nversion = 1\n")
    );
    let error = Suite::parse(&duplicate).expect_err("duplicates should fail");

    assert!(error
        .to_string()
        .contains("duplicate example path `registered`"));
}

#[test]
fn unsafe_example_paths_are_rejected() {
    let error = Suite::parse(&VALID_MANIFEST.replace("registered", "../sqlite"))
        .expect_err("traversal should fail");

    assert!(error.to_string().contains("must not contain traversal"));
}

#[test]
fn duplicate_source_ids_are_rejected() {
    let duplicate_source = r#"
[[example.source]]
id = "app"
backend = "duckdb"
schema_dir = "schema/warehouse"
"#;
    let error = Suite::parse(&format!("{VALID_MANIFEST}{duplicate_source}"))
        .expect_err("duplicate source IDs should fail");

    assert!(error.to_string().contains("repeats source `app`"));
}

#[test]
fn unregistered_example_directories_are_rejected() {
    let root = tempfile::tempdir().expect("temporary examples root should be created");
    write_discovery_config(root.path(), "registered");
    write_discovery_config(root.path(), "unregistered");
    let suite = Suite::parse(VALID_MANIFEST).expect("manifest should parse");

    let error = assert_inventory_conforms_at(root.path(), &suite)
        .expect_err("unregistered example should fail");

    assert!(error.to_string().contains("unregistered"));
}

#[test]
fn stale_manifest_entries_are_rejected() {
    let root = tempfile::tempdir().expect("temporary examples root should be created");
    write_discovery_config(root.path(), "registered");
    let stale = format!(
        "{VALID_MANIFEST}\n{}",
        r#"
[[example]]
path = "missing"
[[example.source]]
id = "missing"
backend = "sqlite"
schema_dir = "schema/missing"
"#
    );
    let suite = Suite::parse(&stale).expect("manifest should parse");

    let error =
        assert_inventory_conforms_at(root.path(), &suite).expect_err("stale entry should fail");

    assert!(error.to_string().contains("missing"));
}

#[test]
fn structurally_incomplete_registered_examples_are_rejected() {
    let root = tempfile::tempdir().expect("temporary examples root should be created");
    write_discovery_config(root.path(), "registered");
    let suite = Suite::parse(VALID_MANIFEST).expect("manifest should parse");

    let error = assert_inventory_conforms_at(root.path(), &suite)
        .expect_err("incomplete example should fail");

    assert!(error.to_string().contains("README.md"));
}

fn write_discovery_config(root: &Path, example: &str) {
    let directory = root.join(example);
    fs::create_dir_all(&directory).expect("example directory should be created");
    fs::write(directory.join("dbmd.toml"), "").expect("discovery config should be written");
}

#[test]
fn example_inventory_is_complete_and_structurally_conformant() {
    assert_inventory_conforms(&load_suite()).expect("example inventory should conform");
}

#[test]
fn sqlite_quickstart_is_exact_fresh_deterministic_and_safe() {
    let suite = load_suite();
    run_example(suite.example("quickstart/sqlite"), BTreeMap::new())
        .expect("SQLite quickstart should execute");
}

#[test]
fn sqlite_backend_showcase_is_exact_fresh_deterministic_and_safe() {
    let suite = load_suite();
    run_example(suite.example("backends/sqlite"), BTreeMap::new())
        .expect("SQLite backend example should execute");
}

#[test]
fn duckdb_backend_showcase_is_exact_fresh_deterministic_and_safe() {
    let suite = load_suite();
    run_example(suite.example("backends/duckdb"), BTreeMap::new())
        .expect("DuckDB backend example should execute");
}

#[test]
fn embedded_multi_source_example_is_exact_fresh_deterministic_and_safe() {
    let suite = load_suite();
    run_example(
        suite.example("workflows/multi-source-embedded"),
        BTreeMap::new(),
    )
    .expect("embedded multi-source example should execute");
}

#[test]
fn layout_example_is_exact_fresh_deterministic_and_safe() {
    let suite = load_suite();
    run_example(suite.example("workflows/layouts"), BTreeMap::new())
        .expect("layout example should execute");
}

#[test]
fn custom_template_example_is_exact_fresh_deterministic_and_safe() {
    let suite = load_suite();
    run_example(suite.example("workflows/custom-templates"), BTreeMap::new())
        .expect("custom-template example should execute");
}

#[test]
fn canonical_lifecycle_example_is_exact_fresh_deterministic_and_safe() {
    let suite = load_suite();
    run_example(
        suite.example("workflows/canonical-lifecycle"),
        BTreeMap::new(),
    )
    .expect("canonical lifecycle example should execute");
}
