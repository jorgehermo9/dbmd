use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    str::FromStr,
};

use dbmd_core::SourceId;
use dbmd_introspect::sqlite::{SqliteSource, SqliteSourceError};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectConfig {
    sources: BTreeMap<String, SourceConfig>,
    output: OutputConfig,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceConfig {
    backend: String,
    path: String,
    display_name: Option<String>,
    #[serde(default)]
    attachments: BTreeMap<String, AttachmentConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AttachmentConfig {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OutputConfig {
    path: String,
    sources: Option<Vec<String>>,
}

#[derive(Debug)]
pub(super) struct RenderPlan {
    pub sources: Vec<SqliteSource>,
    pub output_path: PathBuf,
}

pub(super) fn resolve(
    contents: &str,
    config_path: &Path,
    environment: &BTreeMap<String, String>,
) -> Result<RenderPlan, ConfigError> {
    let config: ProjectConfig = toml::from_str(contents)?;
    if config.sources.is_empty() {
        return Err(ConfigError::NoSources);
    }

    let base = config_path.parent().unwrap_or_else(|| Path::new("."));
    let selection = config
        .output
        .sources
        .unwrap_or_else(|| config.sources.keys().cloned().collect());
    if selection.is_empty() {
        return Err(ConfigError::EmptySelection);
    }

    let mut seen = BTreeSet::new();
    let mut sources = Vec::with_capacity(selection.len());
    for selected_id in selection {
        if !seen.insert(selected_id.clone()) {
            return Err(ConfigError::DuplicateSelection(selected_id));
        }
        let source_config = config
            .sources
            .get(&selected_id)
            .ok_or_else(|| ConfigError::UnknownSource(selected_id.clone()))?;
        if source_config.backend != "sqlite" {
            return Err(ConfigError::UnsupportedBackend {
                source_id: selected_id,
                backend: source_config.backend.clone(),
            });
        }

        let source_id = SourceId::from_str(&selected_id)?;
        let path = resolve_path(base, &expand_environment(&source_config.path, environment)?);
        let mut source = SqliteSource::new(source_id, path);
        if let Some(display_name) = &source_config.display_name {
            source = source.with_display_name(display_name);
        }
        for (namespace, attachment) in &source_config.attachments {
            let path = resolve_path(base, &expand_environment(&attachment.path, environment)?);
            source = source.with_attached_database(namespace, path)?;
        }
        sources.push(source);
    }

    let output_path = resolve_path(base, &expand_environment(&config.output.path, environment)?);
    Ok(RenderPlan {
        sources,
        output_path,
    })
}

fn resolve_path(base: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn expand_environment(
    value: &str,
    environment: &BTreeMap<String, String>,
) -> Result<String, ConfigError> {
    let mut expanded = String::with_capacity(value.len());
    let mut remaining = value;
    while let Some(start) = remaining.find("${") {
        expanded.push_str(&remaining[..start]);
        let after_start = &remaining[start + 2..];
        let end = after_start
            .find('}')
            .ok_or(ConfigError::UnclosedEnvironment)?;
        let name = &after_start[..end];
        if !is_environment_name(name) {
            return Err(ConfigError::InvalidEnvironmentName(name.to_string()));
        }
        let replacement = environment
            .get(name)
            .ok_or_else(|| ConfigError::MissingEnvironment(name.to_string()))?;
        expanded.push_str(replacement);
        remaining = &after_start[end + 1..];
    }
    expanded.push_str(remaining);
    Ok(expanded)
}

fn is_environment_name(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to parse project configuration")]
    Parse(#[from] toml::de::Error),
    #[error("project configuration contains no sources")]
    NoSources,
    #[error("output source selection cannot be empty")]
    EmptySelection,
    #[error("output selects source `{0}` more than once")]
    DuplicateSelection(String),
    #[error("output selects unknown source `{0}`")]
    UnknownSource(String),
    #[error("source `{source_id}` uses unsupported backend `{backend}`")]
    UnsupportedBackend { source_id: String, backend: String },
    #[error(transparent)]
    SourceId(#[from] dbmd_core::SourceIdError),
    #[error(transparent)]
    SqliteSource(#[from] SqliteSourceError),
    #[error("environment reference is missing a closing brace")]
    UnclosedEnvironment,
    #[error("invalid environment variable name `{0}`")]
    InvalidEnvironmentName(String),
    #[error("required environment variable `{0}` is not set")]
    MissingEnvironment(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_multiple_environment_references_without_shell_semantics() {
        let environment = BTreeMap::from([
            ("ROOT".to_string(), "/data".to_string()),
            ("NAME".to_string(), "app".to_string()),
        ]);

        let result = expand_environment("${ROOT}/${NAME}.db", &environment);

        assert_eq!(
            result.expect("valid environment references should expand"),
            "/data/app.db"
        );
    }

    #[test]
    fn preserves_explicit_named_source_selection_order() {
        let config = r#"
[sources.zeta]
backend = "sqlite"
path = "zeta.db"

[sources.alpha]
backend = "sqlite"
path = "alpha.db"

[output]
path = "DATABASE.md"
sources = ["zeta", "alpha"]
"#;

        let plan = resolve(config, Path::new("/project/dbmd.toml"), &BTreeMap::new())
            .expect("selection should resolve");

        assert_eq!(
            plan.sources
                .iter()
                .map(|source| source.id().as_str())
                .collect::<Vec<_>>(),
            ["zeta", "alpha"]
        );
    }

    #[test]
    fn sorts_all_sources_by_stable_id_when_selection_is_omitted() {
        let config = r#"
[sources.zeta]
backend = "sqlite"
path = "zeta.db"

[sources.alpha]
backend = "sqlite"
path = "alpha.db"

[output]
path = "DATABASE.md"
"#;

        let plan = resolve(config, Path::new("/project/dbmd.toml"), &BTreeMap::new())
            .expect("default selection should resolve");

        assert_eq!(
            plan.sources
                .iter()
                .map(|source| source.id().as_str())
                .collect::<Vec<_>>(),
            ["alpha", "zeta"]
        );
    }

    #[test]
    fn rejects_a_missing_required_environment_variable() {
        let config = r#"
[sources.app]
backend = "sqlite"
path = "${DATABASE_PATH}"

[output]
path = "DATABASE.md"
"#;

        let error = resolve(config, Path::new("/project/dbmd.toml"), &BTreeMap::new())
            .expect_err("unresolved environment reference should fail");

        assert!(matches!(
            error,
            ConfigError::MissingEnvironment(name) if name == "DATABASE_PATH"
        ));
    }
}
