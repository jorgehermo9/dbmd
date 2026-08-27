use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{ensure, Context, Result};
use dbmd_app::{render, verify, ArtifactChangeKind, RenderRequest, VerifyRequest};
use dbmd_test_support::TestProject;
use duckdb::Connection as DuckDbConnection;
use rusqlite::Connection as SqliteConnection;

use super::manifest::{Backend, Example, Suite};

const SUITE: &str = include_str!("../suite.toml");

pub fn load_suite() -> Suite {
    Suite::parse(SUITE).expect("committed example suite manifest should be valid")
}

pub fn assert_inventory_conforms(suite: &Suite) -> Result<()> {
    assert_inventory_conforms_at(&examples_root(), suite)
}

pub fn assert_inventory_conforms_at(examples_root: &Path, suite: &Suite) -> Result<()> {
    let discovered = discover_example_roots(examples_root)?;
    let registered = suite
        .example
        .iter()
        .map(|example| PathBuf::from(&example.path))
        .collect::<BTreeSet<_>>();

    ensure!(
        discovered == registered,
        "example inventory differs:\n  unregistered: {:?}\n  missing: {:?}",
        discovered.difference(&registered).collect::<Vec<_>>(),
        registered.difference(&discovered).collect::<Vec<_>>()
    );

    for example in &suite.example {
        assert_example_structure(examples_root, example)?;
    }
    Ok(())
}

pub fn schema_sql(example: &Example, source_id: &str) -> Result<String> {
    let source = example
        .source
        .iter()
        .find(|source| source.id == source_id)
        .with_context(|| format!("example `{}` has no source `{source_id}`", example.path))?;
    read_sql_directory(&examples_root().join(&example.path).join(&source.schema_dir))
}

pub fn run_example(example: &Example, environment: BTreeMap<String, String>) -> Result<()> {
    let source_root = examples_root().join(&example.path);
    let project = TestProject::new();
    copy_directory(&source_root, project.root())?;
    initialize_embedded_sources(project.root(), example)?;

    for config_name in config_names(&source_root)? {
        let config_path = project.root().join(&config_name);
        let output = configured_output(&config_path)?;
        let expected = source_root.join(&output);
        let original = read_artifact(&expected).ok();

        render(RenderRequest::with_environment(
            &config_path,
            environment.clone(),
        ))?;
        let actual_path = project.root().join(&output);
        let first = read_artifact(&actual_path)?;

        if should_update_examples() {
            replace_artifact(&actual_path, &expected)?;
        } else {
            let original = original.with_context(|| {
                format!(
                    "example `{}` is missing `{}`",
                    example.path,
                    output.display()
                )
            })?;
            ensure!(
                first == original,
                "example `{}` config `{}` rendered artifact differs from `{}`",
                example.path,
                config_name.display(),
                output.display()
            );
        }

        let report = verify(VerifyRequest::with_environment(
            &config_path,
            environment.clone(),
        ))?;
        ensure!(
            report.is_fresh(),
            "example `{}` config `{}` reports drift: {:?}",
            example.path,
            config_name.display(),
            report.changes
        );

        render(RenderRequest::with_environment(
            &config_path,
            environment.clone(),
        ))?;
        let second = read_artifact(&actual_path)?;
        ensure!(
            first == second,
            "example `{}` is not deterministic",
            example.path
        );
        assert_no_runtime_values(&first, project.root(), &environment)?;
    }
    if example.drift_check {
        assert_drift_is_detected_without_rewrite(project.root(), &environment)?;
    }
    Ok(())
}

fn assert_drift_is_detected_without_rewrite(
    project_root: &Path,
    environment: &BTreeMap<String, String>,
) -> Result<()> {
    let config_path = project_root.join("dbmd.toml");
    let output_path = project_root.join(configured_output(&config_path)?);
    ensure!(
        output_path.is_file(),
        "drift example must own a single-file artifact"
    );
    let mut file = fs::OpenOptions::new().append(true).open(&output_path)?;
    file.write_all(b"\nmanual example drift\n")?;
    file.sync_all()?;
    let drifted = fs::read(&output_path)?;

    let report =
        verify(VerifyRequest::with_environment(&config_path, environment.clone()).with_diff(true))?;

    ensure!(
        !report.is_fresh(),
        "drift example unexpectedly remained fresh"
    );
    ensure!(
        report.changes.len() == 1 && report.changes[0].kind == ArtifactChangeKind::Modified,
        "drift example returned unexpected changes: {:?}",
        report.changes
    );
    ensure!(
        report.diff.is_some(),
        "drift example did not produce its requested diff"
    );
    ensure!(
        fs::read(&output_path)? == drifted,
        "verify rewrote the drifted artifact"
    );
    Ok(())
}

fn assert_example_structure(examples_root: &Path, example: &Example) -> Result<()> {
    let root = examples_root.join(&example.path);
    for required in ["README.md", "justfile"] {
        ensure!(
            root.join(required).is_file(),
            "example `{}` is missing `{required}`",
            example.path
        );
    }

    let recipes = just_recipes(&root.join("justfile"))?;
    for recipe in ["render", "verify", "down"] {
        ensure!(
            recipes.contains(recipe),
            "example `{}` justfile is missing `{recipe}`",
            example.path
        );
    }

    let configs = config_names(&root)?;
    ensure!(
        !configs.is_empty(),
        "example `{}` has no dbmd config",
        example.path
    );
    for config in &configs {
        assert_config_sources(&root.join(config), example)?;
        let output = configured_output(&root.join(config))?;
        if !should_update_examples() {
            ensure!(
                root.join(&output).exists(),
                "example `{}` config `{}` is missing artifact `{}`",
                example.path,
                config.display(),
                output.display()
            );
        }
    }

    for source in &example.source {
        let schema_dir = root.join(&source.schema_dir);
        ensure!(
            schema_dir.is_dir(),
            "example `{}` source `{}` is missing schema directory `{}`",
            example.path,
            source.id,
            source.schema_dir
        );
        ensure!(
            !sql_files(&schema_dir)?.is_empty(),
            "example `{}` source `{}` has no SQL files",
            example.path,
            source.id
        );
    }

    let embedded_sources = example
        .source
        .iter()
        .filter(|source| source.backend.is_embedded())
        .collect::<Vec<_>>();
    if !embedded_sources.is_empty() {
        let justfile = fs::read_to_string(root.join("justfile"))?;
        for source in embedded_sources {
            let initializer = match source.backend {
                Backend::Sqlite => "sqlite3",
                Backend::Duckdb => "duckdb",
                _ => unreachable!("only embedded sources were selected"),
            };
            ensure!(
                justfile.contains(initializer) && justfile.contains(&source.schema_dir),
                "example `{}` does not initialize source `{}` from its schema in the justfile",
                example.path,
                source.id
            );
        }
    }

    let server_sources = example
        .source
        .iter()
        .filter(|source| !source.backend.is_embedded())
        .collect::<Vec<_>>();
    if !server_sources.is_empty() {
        assert_compose_contract(&root, &server_sources)?;
    }
    Ok(())
}

fn assert_compose_contract(root: &Path, sources: &[&super::manifest::ExampleSource]) -> Result<()> {
    let compose_path = root.join("compose.yaml");
    ensure!(
        compose_path.is_file(),
        "server example `{}` is missing compose.yaml",
        root.display()
    );
    let compose = fs::read_to_string(&compose_path)?;
    ensure!(
        compose.contains("healthcheck:"),
        "compose file `{}` has no health check",
        compose_path.display()
    );

    for source in sources {
        let image = match source.backend {
            Backend::Postgres => "postgres:18.4-alpine",
            Backend::Clickhouse => "clickhouse/clickhouse-server:26.6.1.1193",
            Backend::Mysql => "mysql:9.7.1",
            Backend::Mariadb => "mariadb:12.3.2",
            Backend::Duckdb | Backend::Sqlite => unreachable!("embedded sources were filtered out"),
        };
        ensure!(
            compose.contains(&format!("image: {image}")),
            "compose file `{}` does not pin `{image}`",
            compose_path.display()
        );
        ensure!(
            compose.contains(&format!(
                "./{}:/docker-entrypoint-initdb.d:ro",
                source.schema_dir
            )),
            "compose file `{}` does not mount source `{}` schema read-only",
            compose_path.display(),
            source.id
        );
        if source.backend == Backend::Clickhouse {
            ensure!(
                compose.contains("system.workloads") && compose.contains("analytics_interactive"),
                "compose file `{}` health check does not wait for ClickHouse initialization",
                compose_path.display()
            );
        }
    }
    Ok(())
}

fn assert_config_sources(config_path: &Path, example: &Example) -> Result<()> {
    let config = parse_toml(config_path)?;
    let sources = config
        .get("sources")
        .and_then(toml::Value::as_table)
        .context("config has no [sources] table")?;
    let expected = example
        .source
        .iter()
        .map(|source| source.id.as_str())
        .collect::<BTreeSet<_>>();
    let actual = sources.keys().map(String::as_str).collect::<BTreeSet<_>>();
    ensure!(
        actual == expected,
        "config `{}` sources differ from manifest: expected {expected:?}, found {actual:?}",
        config_path.display()
    );

    for source in &example.source {
        let backend = sources
            .get(&source.id)
            .and_then(toml::Value::as_table)
            .and_then(|value| value.get("backend"))
            .and_then(toml::Value::as_str)
            .with_context(|| format!("source `{}` has no backend", source.id))?;
        ensure!(
            backend == source.backend.as_str(),
            "config `{}` source `{}` backend `{backend}` differs from manifest `{}`",
            config_path.display(),
            source.id,
            source.backend.as_str()
        );
    }
    Ok(())
}

fn initialize_embedded_sources(project_root: &Path, example: &Example) -> Result<()> {
    let config_path = project_root.join("dbmd.toml");
    let config = parse_toml(&config_path)?;
    let configured_sources = config
        .get("sources")
        .and_then(toml::Value::as_table)
        .context("config has no sources")?;

    for source in example
        .source
        .iter()
        .filter(|source| source.backend.is_embedded())
    {
        let path = configured_sources
            .get(&source.id)
            .and_then(toml::Value::as_table)
            .and_then(|source| source.get("path"))
            .and_then(toml::Value::as_str)
            .with_context(|| format!("embedded source `{}` has no path", source.id))?;
        ensure!(
            !path.contains("${"),
            "embedded example source `{}` must use a project-relative path",
            source.id
        );
        let database_path = project_root.join(path);
        if let Some(parent) = database_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let sql = read_sql_directory(&project_root.join(&source.schema_dir))?;
        match source.backend {
            Backend::Sqlite => SqliteConnection::open(&database_path)?.execute_batch(&sql)?,
            Backend::Duckdb => DuckDbConnection::open(&database_path)?.execute_batch(&sql)?,
            _ => unreachable!("only embedded sources are selected"),
        }
    }
    Ok(())
}

fn discover_example_roots(root: &Path) -> Result<BTreeSet<PathBuf>> {
    fn visit(root: &Path, current: &Path, found: &mut BTreeSet<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, found)?;
            } else if is_dbmd_config(&path) {
                let parent = path.parent().context("config should have a parent")?;
                found.insert(parent.strip_prefix(root)?.to_path_buf());
            }
        }
        Ok(())
    }

    let mut found = BTreeSet::new();
    visit(root, root, &mut found)?;
    Ok(found)
}

fn config_names(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(root)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| is_dbmd_config(path))
        .map(|path| {
            path.file_name()
                .map(PathBuf::from)
                .context("config should have a filename")
        })
        .collect::<Result<Vec<_>>>()?;
    paths.sort();
    Ok(paths)
}

fn is_dbmd_config(path: &Path) -> bool {
    path.extension() == Some(OsStr::new("toml"))
        && path
            .file_stem()
            .and_then(OsStr::to_str)
            .is_some_and(|stem| stem == "dbmd" || stem.starts_with("dbmd."))
}

fn configured_output(config_path: &Path) -> Result<PathBuf> {
    let config = parse_toml(config_path)?;
    let output = config
        .get("output")
        .and_then(toml::Value::as_table)
        .and_then(|output| output.get("path"))
        .and_then(toml::Value::as_str)
        .context("config has no output path")?;
    let output = PathBuf::from(output);
    ensure!(
        !output.is_absolute()
            && !output
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir)),
        "example output must stay inside its project: {}",
        output.display()
    );
    Ok(output)
}

fn parse_toml(path: &Path) -> Result<toml::Value> {
    toml::from_str(&fs::read_to_string(path)?)
        .with_context(|| format!("failed to parse `{}`", path.display()))
}

fn read_sql_directory(path: &Path) -> Result<String> {
    let mut sql = String::new();
    for file in sql_files(path)? {
        sql.push_str(&fs::read_to_string(&file)?);
        sql.push('\n');
    }
    Ok(sql)
}

fn sql_files(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = fs::read_dir(path)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file() && path.extension() == Some(OsStr::new("sql")))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn just_recipes(path: &Path) -> Result<BTreeSet<String>> {
    let contents = fs::read_to_string(path)?;
    Ok(contents
        .lines()
        .filter(|line| !line.starts_with(char::is_whitespace))
        .filter_map(|line| line.split_once(':').map(|(recipe, _)| recipe))
        .filter(|line| !line.starts_with('[') && !line.contains(" := "))
        .filter_map(|recipe| recipe.split_whitespace().next())
        .map(str::to_owned)
        .collect())
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            fs::create_dir_all(&destination_path)?;
            copy_directory(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

fn read_artifact(path: &Path) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    fn visit(root: &Path, current: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) -> Result<()> {
        if current.is_file() {
            let relative =
                current.strip_prefix(root.parent().context("artifact should have a parent")?)?;
            files.insert(relative.to_path_buf(), fs::read(current)?);
            return Ok(());
        }
        ensure!(
            current.is_dir(),
            "artifact `{}` does not exist",
            current.display()
        );
        for entry in fs::read_dir(current)? {
            visit(root, &entry?.path(), files)?;
        }
        Ok(())
    }

    let mut files = BTreeMap::new();
    visit(path, path, &mut files)?;
    Ok(files)
}

fn replace_artifact(source: &Path, destination: &Path) -> Result<()> {
    if destination.is_dir() {
        fs::remove_dir_all(destination)?;
    } else if destination.exists() {
        fs::remove_file(destination)?;
    }
    if source.is_dir() {
        fs::create_dir_all(destination)?;
        copy_directory(source, destination)
    } else {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, destination)?;
        Ok(())
    }
}

fn assert_no_runtime_values(
    artifact: &BTreeMap<PathBuf, Vec<u8>>,
    project_root: &Path,
    environment: &BTreeMap<String, String>,
) -> Result<()> {
    let mut forbidden = environment.values().cloned().collect::<Vec<_>>();
    forbidden.push(project_root.to_string_lossy().into_owned());
    forbidden.extend([
        "dbmd-password-sentinel".to_string(),
        "dbmd-engine-secret-sentinel".to_string(),
        "dbmd-dictionary-secret-sentinel".to_string(),
        "dbmd-collection-secret-sentinel".to_string(),
        "dbmd-mariadb-server-secret-sentinel".to_string(),
    ]);

    for (path, contents) in artifact {
        let contents = String::from_utf8_lossy(contents);
        for value in forbidden.iter().filter(|value| !value.is_empty()) {
            ensure!(
                !contents.contains(value),
                "artifact `{}` contains runtime or credential value `{value}`",
                path.display()
            );
        }
    }
    Ok(())
}

fn should_update_examples() -> bool {
    std::env::var("DBMD_EXAMPLES_UPDATE").as_deref() == Ok("always")
}

fn examples_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}
