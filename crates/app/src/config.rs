use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    str::FromStr,
};

use dbmd_core::SourceId;
use dbmd_introspect::sqlite::{SqliteSource, SqliteSourceError};
use dbmd_introspect::{postgres::PostgresSource, Source};
use dbmd_render::{OutputLayout, RenderOptions, SourceLayout};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParsedProject {
    sources: BTreeMap<String, SourceConfig>,
    output: OutputConfig,
    templates: Option<TemplatesConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "backend", rename_all = "lowercase", deny_unknown_fields)]
enum SourceConfig {
    Sqlite {
        path: String,
        display_name: Option<String>,
        #[serde(default)]
        attachments: BTreeMap<String, AttachmentConfig>,
    },
    Postgres {
        url: String,
        display_name: Option<String>,
    },
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
    profile: Option<String>,
    layout: Option<LayoutConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplatesConfig {
    dir: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct LayoutConfig {
    #[serde(default)]
    kind: LayoutKind,
    #[serde(default)]
    source_layout: SourceLayoutConfig,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum LayoutKind {
    #[default]
    SingleFile,
    Directory,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum SourceLayoutConfig {
    #[default]
    Auto,
    Nested,
}

#[derive(Debug)]
pub(super) struct RenderPlan {
    pub sources: Vec<Source>,
    pub project_root: Option<PathBuf>,
    pub repository_root: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub template_root: Option<PathBuf>,
    pub profile: String,
    pub render_options: RenderOptions,
}

#[derive(Debug)]
struct ResolvedProject {
    sources: Vec<Source>,
    project_root: Option<PathBuf>,
    repository_root: Option<PathBuf>,
    output_path: Option<PathBuf>,
    template_root: Option<PathBuf>,
    profile: String,
    render_options: RenderOptions,
}

#[derive(Debug, Default)]
pub(super) struct Overrides {
    pub source_selection: Option<Vec<String>>,
    pub output_path: Option<PathBuf>,
    pub template_root: Option<PathBuf>,
}

pub(super) fn resolve(
    contents: &str,
    config_path: &Path,
    environment: &BTreeMap<String, String>,
    overrides: Overrides,
) -> Result<RenderPlan, ConfigError> {
    let parsed = parse(contents)?;
    let resolved = resolve_project(parsed, config_path, environment, overrides)?;
    RenderPlan::try_from(resolved)
}

fn parse(contents: &str) -> Result<ParsedProject, ConfigError> {
    Ok(toml::from_str(contents)?)
}

fn resolve_project(
    config: ParsedProject,
    config_path: &Path,
    environment: &BTreeMap<String, String>,
    overrides: Overrides,
) -> Result<ResolvedProject, ConfigError> {
    if config.sources.is_empty() {
        return Err(ConfigError::NoSources);
    }

    let base = config_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let selection = overrides
        .source_selection
        .or(config.output.sources)
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
        let source_id = SourceId::from_str(&selected_id)?;
        let source = match source_config {
            SourceConfig::Sqlite {
                path,
                display_name,
                attachments,
            } => {
                let path = resolve_path(base, &expand_environment(path, environment)?);
                let mut source = SqliteSource::new(source_id, path);
                if let Some(display_name) = display_name {
                    source = source.with_display_name(display_name);
                }
                for (namespace, attachment) in attachments {
                    let path =
                        resolve_path(base, &expand_environment(&attachment.path, environment)?);
                    source = source.with_attached_database(namespace, path)?;
                }
                Source::Sqlite(source)
            }
            SourceConfig::Postgres { url, display_name } => {
                let mut source =
                    PostgresSource::new(source_id, expand_environment(url, environment)?);
                if let Some(display_name) = display_name {
                    source = source.with_display_name(display_name);
                }
                Source::Postgres(source)
            }
        };
        sources.push(source);
    }

    let profile = config.output.profile.unwrap_or_else(|| "agent".to_string());
    let template_root = match overrides.template_root {
        Some(path) => Some(resolve_path(base, &path.to_string_lossy())),
        None => config
            .templates
            .map(|templates| expand_environment(&templates.dir, environment))
            .transpose()?
            .map(|path| resolve_path(base, &path)),
    };
    let layout = config.output.layout.unwrap_or_default();
    let render_options = RenderOptions {
        layout: match layout.kind {
            LayoutKind::SingleFile => OutputLayout::SingleFile,
            LayoutKind::Directory => OutputLayout::Directory,
        },
        source_layout: match layout.source_layout {
            SourceLayoutConfig::Auto => SourceLayout::Auto,
            SourceLayoutConfig::Nested => SourceLayout::Nested,
        },
    };
    let output_path = match overrides.output_path {
        Some(path) => resolve_path(base, &path.to_string_lossy()),
        None => resolve_path(base, &expand_environment(&config.output.path, environment)?),
    };
    Ok(ResolvedProject {
        sources,
        project_root: Some(base.to_path_buf()),
        repository_root: find_repository_root(base),
        output_path: Some(output_path),
        template_root,
        profile,
        render_options,
    })
}

impl TryFrom<ResolvedProject> for RenderPlan {
    type Error = ConfigError;

    fn try_from(project: ResolvedProject) -> Result<Self, Self::Error> {
        if project.template_root.is_none() && project.profile != "agent" {
            return Err(ConfigError::UnsupportedProfile(project.profile));
        }
        Ok(Self {
            sources: project.sources,
            project_root: project.project_root,
            repository_root: project.repository_root,
            output_path: project.output_path,
            template_root: project.template_root,
            profile: project.profile,
            render_options: project.render_options,
        })
    }
}

fn find_repository_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|ancestor| std::fs::symlink_metadata(ancestor.join(".git")).is_ok())
        .map(Path::to_path_buf)
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
    #[error("source selection overrides require project configuration")]
    SelectionWithoutConfig,
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
    #[error("unsupported output profile `{0}`")]
    UnsupportedProfile(String),
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

        let plan = resolve(
            config,
            Path::new("/project/dbmd.toml"),
            &BTreeMap::new(),
            Overrides::default(),
        )
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

        let plan = resolve(
            config,
            Path::new("/project/dbmd.toml"),
            &BTreeMap::new(),
            Overrides::default(),
        )
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

        let error = resolve(
            config,
            Path::new("/project/dbmd.toml"),
            &BTreeMap::new(),
            Overrides::default(),
        )
        .expect_err("unresolved environment reference should fail");

        assert!(matches!(
            error,
            ConfigError::MissingEnvironment(name) if name == "DATABASE_PATH"
        ));
    }

    #[test]
    fn resolves_mixed_backend_sources_without_exposing_postgres_credentials() {
        let config = r#"
[sources.local]
backend = "sqlite"
path = "local.db"

[sources.production]
backend = "postgres"
url = "${DATABASE_URL}"
display_name = "Production"

[output]
path = "DATABASE.md"
sources = ["production", "local"]
"#;
        let environment = BTreeMap::from([(
            "DATABASE_URL".to_string(),
            "postgres://secret:password@database/app".to_string(),
        )]);

        let plan = resolve(
            config,
            Path::new("/project/dbmd.toml"),
            &environment,
            Overrides::default(),
        )
        .expect("both concrete backends should resolve");

        assert_eq!(
            plan.sources
                .iter()
                .map(|source| source.id().as_str())
                .collect::<Vec<_>>(),
            ["production", "local"]
        );
        assert!(matches!(
            &plan.sources[0],
            dbmd_introspect::Source::Postgres(_)
        ));
        assert!(!format!("{:?}", plan.sources[0]).contains("password"));
    }

    #[test]
    fn rejects_empty_duplicate_unknown_and_invalid_source_selections() {
        let config = r#"
[sources.app]
backend = "sqlite"
path = "app.db"

[sources."bad/id"]
backend = "sqlite"
path = "bad.db"

[output]
path = "DATABASE.md"
"#;
        let cases = [
            (Vec::<String>::new(), "cannot be empty"),
            (vec!["app".into(), "app".into()], "more than once"),
            (vec!["missing".into()], "unknown source"),
            (vec!["bad/id".into()], "invalid character"),
        ];

        for (selection, expected) in cases {
            let error = resolve(
                config,
                Path::new("/project/dbmd.toml"),
                &BTreeMap::new(),
                Overrides {
                    source_selection: Some(selection),
                    ..Overrides::default()
                },
            )
            .expect_err("invalid selection should fail");
            assert!(error.to_string().contains(expected), "{error}");
        }
    }

    #[test]
    fn resolves_cli_output_and_template_overrides_against_the_config_directory() {
        let config = r#"
[sources.app]
backend = "sqlite"
path = "app.db"

[output]
path = "DATABASE.md"

[templates]
dir = "configured-templates"
"#;

        let plan = resolve(
            config,
            Path::new("/project/config/dbmd.toml"),
            &BTreeMap::new(),
            Overrides {
                output_path: Some("alternate/DB.md".into()),
                template_root: Some("one-off-templates".into()),
                ..Overrides::default()
            },
        )
        .expect("CLI paths should resolve");

        assert_eq!(
            plan.output_path.as_deref(),
            Some(Path::new("/project/config/alternate/DB.md"))
        );
        assert_eq!(
            plan.template_root.as_deref(),
            Some(Path::new("/project/config/one-off-templates"))
        );
    }
}
