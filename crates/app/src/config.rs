use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    str::FromStr,
};

use dbmd_backends::{Source, SourceConfig, SourceConfigResolveError, SourceValidationError};
use dbmd_core::SourceId;
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
    pub canonical_output_path: Option<PathBuf>,
    pub canonical_output_display_path: Option<PathBuf>,
    pub template_root: Option<PathBuf>,
    pub template_display_root: Option<PathBuf>,
    pub profile: String,
    pub render_options: RenderOptions,
    pub required_environment: Vec<String>,
}

#[derive(Debug)]
struct ResolvedProject {
    sources: Vec<Source>,
    project_root: Option<PathBuf>,
    repository_root: Option<PathBuf>,
    output_path: Option<PathBuf>,
    canonical_output_path: Option<PathBuf>,
    canonical_output_display_path: Option<PathBuf>,
    template_root: Option<PathBuf>,
    template_display_root: Option<PathBuf>,
    profile: String,
    render_options: RenderOptions,
    required_environment: Vec<String>,
}

#[derive(Debug, Default)]
pub(super) struct Overrides {
    pub source_selection: Option<Vec<String>>,
    pub all_sources: bool,
    pub output_path: Option<PathBuf>,
    pub template_root: Option<PathBuf>,
    pub resolve_canonical_output: bool,
}

pub(super) struct DoctorPlan {
    pub render: RenderPlan,
    pub missing_environment: Vec<String>,
    pub missing_output_environment: bool,
    pub missing_template_environment: bool,
    pub missing_source_environment: BTreeMap<String, Vec<String>>,
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

pub(super) fn resolve_canonical_output_display(
    contents: &str,
    config_path: &Path,
) -> Result<PathBuf, ConfigError> {
    let parsed = parse(contents)?;
    let base = config_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    environment_names(&parsed.output.path)?;
    Ok(resolve_path(base, &parsed.output.path))
}

pub(super) fn resolve_doctor(
    contents: &str,
    config_path: &Path,
    environment: &BTreeMap<String, String>,
    all_sources: bool,
) -> Result<DoctorPlan, ConfigError> {
    let parsed = parse(contents)?;
    let requirements = environment_requirements(&parsed, all_sources)?;
    let missing_environment = requirements
        .all
        .iter()
        .filter(|name| !environment.contains_key(*name))
        .cloned()
        .collect::<Vec<_>>();
    let mut augmented_environment = environment.clone();
    for name in &missing_environment {
        augmented_environment.insert(name.clone(), "__DBMD_MISSING_ENVIRONMENT__".to_string());
    }
    let render = RenderPlan::try_from(resolve_project(
        parsed,
        config_path,
        &augmented_environment,
        Overrides {
            all_sources,
            resolve_canonical_output: true,
            ..Overrides::default()
        },
    )?)?;
    let missing =
        |names: &BTreeSet<String>| names.iter().any(|name| !environment.contains_key(name));
    let missing_source_environment = requirements
        .by_source
        .into_iter()
        .filter_map(|(source, names)| {
            let missing = names
                .into_iter()
                .filter(|name| !environment.contains_key(name))
                .collect::<Vec<_>>();
            (!missing.is_empty()).then_some((source, missing))
        })
        .collect();
    Ok(DoctorPlan {
        render,
        missing_environment,
        missing_output_environment: missing(&requirements.output),
        missing_template_environment: missing(&requirements.templates),
        missing_source_environment,
    })
}

struct EnvironmentRequirements {
    all: BTreeSet<String>,
    output: BTreeSet<String>,
    templates: BTreeSet<String>,
    by_source: BTreeMap<String, BTreeSet<String>>,
}

fn environment_requirements(
    config: &ParsedProject,
    all_sources: bool,
) -> Result<EnvironmentRequirements, ConfigError> {
    let selection = if all_sources {
        config.sources.keys().cloned().collect::<Vec<_>>()
    } else {
        config
            .output
            .sources
            .clone()
            .unwrap_or_else(|| config.sources.keys().cloned().collect())
    };
    let output = environment_names(&config.output.path)?;
    let templates = config
        .templates
        .as_ref()
        .map(|templates| environment_names(&templates.dir))
        .transpose()?
        .unwrap_or_default();
    let mut all = output.union(&templates).cloned().collect::<BTreeSet<_>>();
    let mut by_source = BTreeMap::new();
    for id in selection {
        let Some(source) = config.sources.get(&id) else {
            continue;
        };
        let mut names = BTreeSet::new();
        for value in source.environment_values() {
            names.extend(environment_names(value)?);
        }
        all.extend(names.iter().cloned());
        by_source.insert(id, names);
    }
    Ok(EnvironmentRequirements {
        all,
        output,
        templates,
        by_source,
    })
}

fn environment_names(value: &str) -> Result<BTreeSet<String>, ConfigError> {
    let mut names = BTreeSet::new();
    let mut remaining = value;
    while let Some(start) = remaining.find("${") {
        let after_start = &remaining[start + 2..];
        let end = after_start
            .find('}')
            .ok_or(ConfigError::UnclosedEnvironment)?;
        let name = &after_start[..end];
        if !is_environment_name(name) {
            return Err(ConfigError::InvalidEnvironmentName(name.to_string()));
        }
        names.insert(name.to_string());
        remaining = &after_start[end + 1..];
    }
    Ok(names)
}

fn parse(contents: &str) -> Result<ParsedProject, ConfigError> {
    toml::from_str(contents).map_err(|error| {
        let kind = ConfigParseKind::from_message(error.message());
        let (line, column) = error
            .span()
            .map(|span| line_and_column(contents, span.start))
            .unwrap_or((1, 1));
        ConfigError::Parse { kind, line, column }
    })
}

fn line_and_column(contents: &str, offset: usize) -> (usize, usize) {
    let mut offset = offset.min(contents.len());
    while !contents.is_char_boundary(offset) {
        offset -= 1;
    }
    let before = &contents[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = before
        .rsplit_once('\n')
        .map_or(before.len(), |(_, current)| current.len())
        + 1;
    (line, column)
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
    let selection = if overrides.all_sources {
        config.sources.keys().cloned().collect()
    } else {
        overrides
            .source_selection
            .or(config.output.sources.clone())
            .unwrap_or_else(|| config.sources.keys().cloned().collect())
    };
    if selection.is_empty() {
        return Err(ConfigError::EmptySelection);
    }

    let mut required_environment = BTreeSet::new();
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
        let source = source_config
            .resolve(source_id, base, |value| {
                expand_environment(value, environment, &mut required_environment)
            })
            .map_err(|error| match error {
                SourceConfigResolveError::Value(error) => error,
                SourceConfigResolveError::Backend(error) => ConfigError::BackendSource(error),
            })?;
        sources.push(source);
    }

    let profile = config.output.profile.unwrap_or_else(|| "agent".to_string());
    let (template_root, template_display_root) = match overrides.template_root {
        Some(path) => {
            let path = resolve_path(base, &path.to_string_lossy());
            (Some(path.clone()), Some(path))
        }
        None => match config.templates {
            Some(templates) => (
                Some(resolve_path(
                    base,
                    &expand_environment(&templates.dir, environment, &mut required_environment)?,
                )),
                Some(resolve_path(base, &templates.dir)),
            ),
            None => (None, None),
        },
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
    let canonical_output_path =
        if overrides.output_path.is_none() || overrides.resolve_canonical_output {
            Some(resolve_path(
                base,
                &expand_environment(&config.output.path, environment, &mut required_environment)?,
            ))
        } else {
            None
        };
    let canonical_output_display_path = resolve_path(base, &config.output.path);
    let output_path = match overrides.output_path {
        Some(path) => resolve_path(base, &path.to_string_lossy()),
        None => canonical_output_path
            .clone()
            .expect("canonical output is resolved when no override is supplied"),
    };
    Ok(ResolvedProject {
        sources,
        project_root: Some(base.to_path_buf()),
        repository_root: find_repository_root(base),
        output_path: Some(output_path),
        canonical_output_path,
        canonical_output_display_path: Some(canonical_output_display_path),
        template_root,
        template_display_root,
        profile,
        render_options,
        required_environment: required_environment.into_iter().collect(),
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
            canonical_output_path: project.canonical_output_path,
            canonical_output_display_path: project.canonical_output_display_path,
            template_root: project.template_root,
            template_display_root: project.template_display_root,
            profile: project.profile,
            render_options: project.render_options,
            required_environment: project.required_environment,
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
    required: &mut BTreeSet<String>,
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
        required.insert(name.to_string());
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
    #[error("project configuration has {kind} at line {line}, column {column}")]
    Parse {
        kind: ConfigParseKind,
        line: usize,
        column: usize,
    },
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
    BackendSource(#[from] SourceValidationError),
    #[error("environment reference is missing a closing brace")]
    UnclosedEnvironment,
    #[error("invalid environment variable name `{0}`")]
    InvalidEnvironmentName(String),
    #[error("required environment variable `{0}` is not set")]
    MissingEnvironment(String),
    #[error("unsupported output profile `{0}`")]
    UnsupportedProfile(String),
}

/// Credential-safe category for a TOML configuration parse failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigParseKind {
    /// TOML syntax is malformed.
    Syntax,
    /// The configuration contains a field outside the supported schema.
    UnknownField,
    /// A required field is absent.
    MissingField,
    /// A field has the wrong TOML value type.
    InvalidType,
    /// A field is declared more than once.
    DuplicateField,
    /// A value cannot be decoded into its configured enum or constrained type.
    InvalidValue,
}

impl ConfigParseKind {
    fn from_message(message: &str) -> Self {
        if message.starts_with("unknown field") {
            Self::UnknownField
        } else if message.starts_with("missing field") {
            Self::MissingField
        } else if message.starts_with("invalid type") {
            Self::InvalidType
        } else if message.starts_with("duplicate field") || message.starts_with("duplicate key") {
            Self::DuplicateField
        } else if message.starts_with("invalid value") || message.starts_with("unknown variant") {
            Self::InvalidValue
        } else {
            Self::Syntax
        }
    }
}

impl std::fmt::Display for ConfigParseKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Syntax => "invalid TOML syntax",
            Self::UnknownField => "an unknown field",
            Self::MissingField => "a missing required field",
            Self::InvalidType => "an invalid field type",
            Self::DuplicateField => "a duplicate field",
            Self::InvalidValue => "an invalid field value",
        })
    }
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

        let result = expand_environment("${ROOT}/${NAME}.db", &environment, &mut BTreeSet::new());

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
            dbmd_backends::Source::Postgres(_)
        ));
        assert!(!format!("{:?}", plan.sources[0]).contains("password"));
    }

    fn selection_error(selection: Vec<String>) -> ConfigError {
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

        resolve(
            config,
            Path::new("/project/dbmd.toml"),
            &BTreeMap::new(),
            Overrides {
                source_selection: Some(selection),
                ..Overrides::default()
            },
        )
        .expect_err("invalid selection should fail")
    }

    #[test]
    fn rejects_an_explicit_empty_source_selection() {
        assert!(matches!(
            selection_error(Vec::new()),
            ConfigError::EmptySelection
        ));
    }

    #[test]
    fn rejects_a_duplicate_source_selection_with_the_duplicate_identity() {
        assert!(matches!(
            selection_error(vec!["app".into(), "app".into()]),
            ConfigError::DuplicateSelection(id) if id == "app"
        ));
    }

    #[test]
    fn rejects_an_unknown_selected_source_with_the_requested_identity() {
        assert!(matches!(
            selection_error(vec!["missing".into()]),
            ConfigError::UnknownSource(id) if id == "missing"
        ));
    }

    #[test]
    fn rejects_a_path_like_source_identity_before_backend_resolution() {
        assert!(matches!(
            selection_error(vec!["bad/id".into()]),
            ConfigError::SourceId(_)
        ));
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

    fn assert_parse_failure(contents: &str, expected_kind: ConfigParseKind) {
        let error = resolve(
            contents,
            Path::new("/project/dbmd.toml"),
            &BTreeMap::new(),
            Overrides::default(),
        )
        .expect_err("invalid configuration should fail");
        let ConfigError::Parse { kind, line, column } = error else {
            panic!("expected parse error, got {error}");
        };
        assert_eq!(kind, expected_kind);
        assert!(line > 0);
        assert!(column > 0);
        assert!(!format!("{error}").contains("sentinel-secret"));
    }

    macro_rules! config_parse_cases {
        ($($name:ident: $contents:expr => $kind:expr;)+) => {
            $(
                #[test]
                fn $name() {
                    assert_parse_failure($contents, $kind);
                }
            )+
        };
    }

    config_parse_cases! {
        classifies_invalid_toml_syntax: "this is not = valid toml" => ConfigParseKind::Syntax;
        classifies_unknown_configuration_field: r#"
[sources.app]
backend = "sqlite"
path = "app.db"
password = "sentinel-secret"
[output]
path = "DATABASE.md"
"# => ConfigParseKind::UnknownField;
        classifies_missing_required_configuration_field: r#"
[sources.app]
backend = "sqlite"
[output]
path = "DATABASE.md"
"# => ConfigParseKind::MissingField;
        classifies_invalid_configuration_field_type: r#"
[sources.app]
backend = "sqlite"
path = 7
[output]
path = "DATABASE.md"
"# => ConfigParseKind::InvalidType;
        classifies_duplicate_configuration_field: r#"
[sources.app]
backend = "sqlite"
path = "app.db"
path = "other.db"
[output]
path = "DATABASE.md"
"# => ConfigParseKind::DuplicateField;
        classifies_invalid_configuration_value: r#"
[sources.app]
backend = "sqlite"
path = "app.db"
[output]
path = "DATABASE.md"
[output.layout]
kind = "archive"
"# => ConfigParseKind::InvalidValue;
    }

    fn assert_invalid_environment_reference(value: &str) {
        let config = format!(
            r#"
[sources.app]
backend = "sqlite"
path = "{value}"

[output]
path = "DATABASE.md"
"#
        );
        let error = resolve(
            &config,
            Path::new("/project/dbmd.toml"),
            &BTreeMap::new(),
            Overrides::default(),
        )
        .expect_err("malformed environment reference should fail");

        assert!(
            matches!(
                error,
                ConfigError::UnclosedEnvironment | ConfigError::InvalidEnvironmentName(_)
            ),
            "{error}"
        );
    }

    macro_rules! invalid_environment_cases {
        ($($name:ident: $value:literal;)+) => {
            $(
                #[test]
                fn $name() {
                    assert_invalid_environment_reference($value);
                }
            )+
        };
    }

    invalid_environment_cases! {
        rejects_unclosed_environment_reference: "${DATABASE_PATH";
        rejects_empty_environment_name: "${}";
        rejects_environment_name_with_numeric_prefix: "${1DATABASE_PATH}";
        rejects_environment_name_with_punctuation: "${DATABASE-PATH}";
    }

    #[test]
    fn embedded_templates_reject_profiles_the_binary_does_not_provide() {
        let config = r#"
[sources.app]
backend = "sqlite"
path = "app.db"

[output]
path = "DATABASE.md"
profile = "human"
"#;

        let error = resolve(
            config,
            Path::new("/project/dbmd.toml"),
            &BTreeMap::new(),
            Overrides::default(),
        )
        .expect_err("unavailable embedded profile should fail locally");

        assert!(matches!(
            error,
            ConfigError::UnsupportedProfile(profile) if profile == "human"
        ));
    }
}
