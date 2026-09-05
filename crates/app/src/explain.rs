use std::{collections::BTreeMap, fs, path::PathBuf};

use dbmd_backends::Backend;
use dbmd_core::SourceId;
use dbmd_render::{OutputLayout, SourceLayout};
use thiserror::Error;

use crate::config;

/// Inputs for resolving and explaining one configured render operation.
#[derive(Clone)]
pub struct ExplainRequest {
    config_path: PathBuf,
    environment: BTreeMap<String, String>,
    source_selection: Option<Vec<String>>,
    output_path: Option<PathBuf>,
    template_root: Option<PathBuf>,
    stdout: bool,
}

impl std::fmt::Debug for ExplainRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ExplainRequest")
            .field("config_path", &self.config_path)
            .field(
                "environment_keys",
                &self.environment.keys().collect::<Vec<_>>(),
            )
            .field("source_selection", &self.source_selection)
            .field("output_path", &self.output_path)
            .field("template_root", &self.template_root)
            .field("stdout", &self.stdout)
            .finish()
    }
}

impl ExplainRequest {
    /// Creates a request using the current process environment.
    #[must_use]
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
            environment: std::env::vars().collect(),
            source_selection: None,
            output_path: None,
            template_root: None,
            stdout: false,
        }
    }

    /// Creates a request with an explicit environment, primarily for embedding and tests.
    #[must_use]
    pub fn with_environment(
        config_path: impl Into<PathBuf>,
        environment: BTreeMap<String, String>,
    ) -> Self {
        Self {
            config_path: config_path.into(),
            environment,
            source_selection: None,
            output_path: None,
            template_root: None,
            stdout: false,
        }
    }

    /// Replaces the configured source selection while preserving supplied order.
    #[must_use]
    pub fn with_sources<I, S>(mut self, sources: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.source_selection = Some(sources.into_iter().map(Into::into).collect());
        self
    }

    /// Replaces the configured output path.
    #[must_use]
    pub fn with_output_path(mut self, output_path: impl Into<PathBuf>) -> Self {
        self.output_path = Some(output_path.into());
        self
    }

    /// Replaces the configured custom template root.
    #[must_use]
    pub fn with_template_root(mut self, template_root: impl Into<PathBuf>) -> Self {
        self.template_root = Some(template_root.into());
        self
    }

    /// Explains a stdout destination instead of the configured filesystem path.
    #[must_use]
    pub fn to_stdout(mut self) -> Self {
        self.stdout = true;
        self
    }
}

/// One credential-free selected source in a resolved operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainedSource {
    /// Stable source identity.
    pub id: SourceId,
    /// Concrete database family.
    pub backend: Backend,
}

impl ExplainedSource {
    /// Returns the stable configuration spelling for this backend.
    #[must_use]
    pub fn backend_name(&self) -> &'static str {
        self.backend.as_str()
    }
}

/// Resolved destination for the requested operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplainDestination {
    /// Credential-safe display form of the filesystem destination.
    Filesystem { display_path: PathBuf },
    /// A single-file artifact will be emitted to standard output.
    Stdout,
}

/// Template set selected by configuration and overrides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateSource {
    /// Built-in templates compiled into dbmd.
    Embedded,
    /// Credential-safe display form of a complete project-owned template root.
    Custom { display_root: PathBuf },
}

/// File organization used inside the directory layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryVariant {
    /// One stable Markdown file per supported schema object plus indexes.
    Objects,
}

/// Credential-free local resolution of one render operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplainReport {
    /// Configuration file that supplied the project contract.
    pub config_path: PathBuf,
    /// Selected sources in deterministic operation order.
    pub sources: Vec<ExplainedSource>,
    /// Credential-safe canonical path display with `${NAME}` references preserved.
    pub canonical_output_display_path: PathBuf,
    /// Effective destination after applying CLI precedence.
    pub destination: ExplainDestination,
    /// Whether the effective destination differs from the canonical contract.
    pub output_overridden: bool,
    /// File organization selected for rendering.
    pub layout: OutputLayout,
    /// Source nesting policy selected for rendering.
    pub source_layout: SourceLayout,
    /// Current directory organization, or none for single-file output.
    pub directory_variant: Option<DirectoryVariant>,
    /// Selected template profile.
    pub profile: String,
    /// Embedded or project-owned template source.
    pub template_source: TemplateSource,
    /// Complete required entrypoints relative to a profile directory.
    pub required_template_entrypoints: Vec<PathBuf>,
    /// Credential-safe output-file displays when knowable without introspection.
    pub planned_display_files: Option<Vec<PathBuf>>,
    /// Required environment variable names, never their values.
    pub required_environment: Vec<String>,
}

impl ExplainReport {
    /// Returns the stable configuration spelling for the selected layout.
    #[must_use]
    pub fn layout_name(&self) -> &'static str {
        match self.layout {
            OutputLayout::SingleFile => "single_file",
            OutputLayout::Directory => "directory",
        }
    }

    /// Returns the stable configuration spelling for source nesting.
    #[must_use]
    pub fn source_layout_name(&self) -> &'static str {
        match self.source_layout {
            SourceLayout::Auto => "auto",
            SourceLayout::Nested => "nested",
        }
    }
}

pub(super) struct ResolvedOperation {
    pub config_path: PathBuf,
    pub plan: config::RenderPlan,
    pub stdout: bool,
    pub output_overridden: bool,
}

/// Resolves a configured render plan without connecting to a database.
///
/// # Errors
///
/// Returns [`ExplainError`] when the config cannot be read or resolved, or
/// when destination overrides conflict with the configured layout.
pub fn explain(request: ExplainRequest) -> Result<ExplainReport, ExplainError> {
    let resolved = resolve_operation(request)?;
    Ok(report(&resolved))
}

pub(super) fn resolve_operation(
    request: ExplainRequest,
) -> Result<ResolvedOperation, ExplainError> {
    if request.stdout && request.output_path.is_some() {
        return Err(ExplainError::ConflictingDestination);
    }
    let contents =
        fs::read_to_string(&request.config_path).map_err(|source| ExplainError::ReadConfig {
            path: request.config_path.clone(),
            source,
        })?;
    let output_override_requested = request.output_path.is_some();
    let plan = config::resolve(
        &contents,
        &request.config_path,
        &request.environment,
        config::Overrides {
            source_selection: request.source_selection,
            all_sources: false,
            output_path: request.output_path,
            template_root: request.template_root,
            resolve_canonical_output: true,
        },
    )?;
    if request.stdout && plan.render_options.layout != OutputLayout::SingleFile {
        return Err(ExplainError::StdoutRequiresSingleFile);
    }
    let output_overridden = request.stdout
        || (output_override_requested && plan.output_path != plan.canonical_output_path);
    Ok(ResolvedOperation {
        config_path: request.config_path,
        plan,
        stdout: request.stdout,
        output_overridden,
    })
}

fn report(operation: &ResolvedOperation) -> ExplainReport {
    let canonical_output_display_path = operation
        .plan
        .canonical_output_display_path
        .clone()
        .expect("configured plans always contain a canonical output display path");
    let destination = if operation.stdout {
        ExplainDestination::Stdout
    } else {
        let path = if operation.output_overridden {
            operation
                .plan
                .output_path
                .clone()
                .expect("configured plans always contain an output path")
        } else {
            canonical_output_display_path.clone()
        };
        ExplainDestination::Filesystem { display_path: path }
    };
    let planned_display_files = match (&destination, operation.plan.render_options.layout) {
        (ExplainDestination::Filesystem { display_path }, OutputLayout::SingleFile) => {
            Some(vec![display_path.clone()])
        }
        (ExplainDestination::Stdout, OutputLayout::SingleFile) => Some(Vec::new()),
        (_, OutputLayout::Directory) => None,
    };
    ExplainReport {
        config_path: operation.config_path.clone(),
        sources: operation
            .plan
            .sources
            .iter()
            .map(|source| ExplainedSource {
                id: source.id().clone(),
                backend: source.backend(),
            })
            .collect(),
        canonical_output_display_path,
        destination,
        output_overridden: operation.output_overridden,
        layout: operation.plan.render_options.layout,
        source_layout: operation.plan.render_options.source_layout,
        directory_variant: (operation.plan.render_options.layout == OutputLayout::Directory)
            .then_some(DirectoryVariant::Objects),
        profile: operation.plan.profile.clone(),
        template_source: operation
            .plan
            .template_display_root
            .clone()
            .map_or(TemplateSource::Embedded, |display_root| {
                TemplateSource::Custom { display_root }
            }),
        required_template_entrypoints: {
            let backend_templates = dbmd_backends::all_template_files();
            dbmd_render::embedded_template_files()
                .iter()
                .chain(&backend_templates)
                .map(|file| PathBuf::from(file.relative_path))
                .collect()
        },
        planned_display_files,
        required_environment: operation.plan.required_environment.clone(),
    }
}

/// Why a local operation plan could not be explained.
#[derive(Debug, Error)]
pub enum ExplainError {
    #[error("failed to read configuration `{path}`")]
    ReadConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error("stdout cannot be combined with an output path")]
    ConflictingDestination,
    #[error("stdout is available only for single-file layout")]
    StdoutRequiresSingleFile,
}
