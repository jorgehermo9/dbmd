#![doc = include_str!("../README.md")]

mod config;
mod doctor;
mod explain;
mod init;
mod init_agents;
mod init_ci;
mod init_templates;
mod output;

pub use config::{ConfigError, ConfigParseKind};
pub use doctor::{
    doctor, Diagnostic, DiagnosticStage, DiagnosticStatus, DoctorReport, DoctorRequest,
};
pub use explain::{
    explain, DirectoryVariant, ExplainDestination, ExplainError, ExplainReport, ExplainRequest,
    ExplainedSource, TemplateSource,
};
pub use init::{init, InitError, InitReport, InitRequest};
pub use init_agents::{init_agents, InitAgentsError, InitAgentsReport, InitAgentsRequest};
pub use init_ci::{init_ci, InitCiError, InitCiReport, InitCiRequest};
pub use init_templates::{
    init_templates, InitTemplatesError, InitTemplatesReport, InitTemplatesRequest,
};
pub use output::OutputError;

use std::{collections::BTreeMap, fs, path::Path, path::PathBuf, str::FromStr};

use dbmd_core::{DatabaseContext, SourceId};
use dbmd_introspect::{self as introspect, IntrospectionError};
use dbmd_render::{RenderedArtifact, Renderer};
use thiserror::Error;

/// Inputs for one configured render operation.
#[derive(Clone)]
pub struct RenderRequest {
    input: RenderInput,
    environment: BTreeMap<String, String>,
    source_selection: Option<Vec<String>>,
    output_path: Option<PathBuf>,
    template_root: Option<PathBuf>,
    stdout: bool,
}

impl std::fmt::Debug for RenderRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RenderRequest")
            .field("input", &self.input)
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

#[derive(Debug, Clone)]
enum RenderInput {
    Config(PathBuf),
    Sqlite(PathBuf),
}

impl RenderRequest {
    /// Creates a request using the current process environment.
    #[must_use]
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Self {
            input: RenderInput::Config(config_path.into()),
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
            input: RenderInput::Config(config_path.into()),
            environment,
            source_selection: None,
            output_path: None,
            template_root: None,
            stdout: false,
        }
    }

    /// Creates a configless one-off request for one SQLite database.
    #[must_use]
    pub fn sqlite(path: impl Into<PathBuf>) -> Self {
        Self {
            input: RenderInput::Sqlite(path.into()),
            environment: BTreeMap::new(),
            source_selection: None,
            output_path: None,
            template_root: None,
            stdout: false,
        }
    }

    /// Replaces the configured source selection while preserving the supplied order.
    #[must_use]
    pub fn with_sources<I, S>(mut self, sources: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.source_selection = Some(sources.into_iter().map(Into::into).collect());
        self
    }

    /// Replaces the configured output path, or supplies one for a configless request.
    #[must_use]
    pub fn with_output_path(mut self, output_path: impl Into<PathBuf>) -> Self {
        self.output_path = Some(output_path.into());
        self
    }

    /// Replaces the configured custom template root for this render.
    #[must_use]
    pub fn with_template_root(mut self, template_root: impl Into<PathBuf>) -> Self {
        self.template_root = Some(template_root.into());
        self
    }

    /// Returns a single-file artifact in memory instead of writing an output path.
    #[must_use]
    pub fn to_stdout(mut self) -> Self {
        self.stdout = true;
        self
    }
}

/// Observable result of a completed render operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReport {
    pub sources: Vec<SourceId>,
    pub output: RenderOutput,
}

/// Destination-specific result of a completed render operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderOutput {
    /// An artifact was atomically written to a filesystem path.
    Written { path: PathBuf, bytes_written: usize },
    /// A single-file artifact was returned to the caller without filesystem output.
    Stdout(Vec<u8>),
}

impl RenderOutput {
    /// Returns the written path, when the render targeted the filesystem.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Written { path, .. } => Some(path),
            Self::Stdout(_) => None,
        }
    }

    /// Returns the number of rendered bytes.
    #[must_use]
    pub fn bytes_written(&self) -> usize {
        match self {
            Self::Written { bytes_written, .. } => *bytes_written,
            Self::Stdout(contents) => contents.len(),
        }
    }
}

/// Inputs for one canonical verification operation.
#[derive(Clone)]
pub struct VerifyRequest {
    config_path: PathBuf,
    environment: BTreeMap<String, String>,
    include_diff: bool,
}

impl std::fmt::Debug for VerifyRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VerifyRequest")
            .field("config_path", &self.config_path)
            .field(
                "environment_keys",
                &self.environment.keys().collect::<Vec<_>>(),
            )
            .field("include_diff", &self.include_diff)
            .finish()
    }
}

impl VerifyRequest {
    /// Creates a request using the current process environment.
    #[must_use]
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
            environment: std::env::vars().collect(),
            include_diff: false,
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
            include_diff: false,
        }
    }

    /// Selects whether the report includes a complete unified diff.
    #[must_use]
    pub fn with_diff(mut self, include_diff: bool) -> Self {
        self.include_diff = include_diff;
        self
    }
}

/// How one canonical artifact entry differs from a fresh render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactChangeKind {
    /// Fresh rendering requires a path missing from the canonical artifact.
    Added,
    /// Both artifacts contain the path with different bytes.
    Modified,
    /// The canonical directory contains a stale path.
    Deleted,
}

/// One deterministic canonical-artifact difference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactChange {
    /// Relative path within the canonical artifact, or the configured filename.
    pub path: String,
    /// Kind of exact file-set or byte difference.
    pub kind: ArtifactChangeKind,
}

/// Observable result of a completed verification operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyReport {
    /// Canonical output path that was compared without modification.
    pub output_path: PathBuf,
    /// Deterministically ordered differences.
    pub changes: Vec<ArtifactChange>,
    /// Complete unified diff when requested and drift exists.
    pub diff: Option<String>,
}

impl VerifyReport {
    /// Returns true when the canonical artifact exactly matches a fresh render.
    #[must_use]
    pub fn is_fresh(&self) -> bool {
        self.changes.is_empty()
    }
}

struct GeneratedArtifact {
    output_path: Option<output::ValidatedOutputPath>,
    source_ids: Vec<SourceId>,
    artifact: RenderedArtifact,
}

/// Resolves config, introspects selected sources, renders Markdown, and atomically replaces output.
///
/// # Errors
///
/// Returns [`RenderError`] when configuration, introspection, rendering, or output replacement fails.
pub fn render(request: RenderRequest) -> Result<RenderReport, RenderError> {
    if request.stdout && request.output_path.is_some() {
        return Err(RenderError::ConflictingDestination);
    }
    let generated = generate(
        &request.input,
        &request.environment,
        config::Overrides {
            source_selection: request.source_selection,
            output_path: request.output_path,
            template_root: request.template_root,
            ..config::Overrides::default()
        },
        if request.stdout {
            BuildDestination::Stdout
        } else {
            BuildDestination::Filesystem
        },
    )?;
    let output = if request.stdout {
        match generated.artifact {
            RenderedArtifact::SingleFile(contents) => RenderOutput::Stdout(contents),
            RenderedArtifact::Directory(_) => {
                unreachable!("stdout layout is validated before introspection")
            }
        }
    } else {
        let destination = generated
            .output_path
            .expect("filesystem destination is validated before introspection");
        let bytes_written = output::replace(&destination, &generated.artifact)?;
        RenderOutput::Written {
            path: destination.into_path(),
            bytes_written,
        }
    };

    Ok(RenderReport {
        sources: generated.source_ids,
        output,
    })
}

/// Renders the canonical project contract in memory and compares exact output.
///
/// This operation never writes to the configured output path.
///
/// # Errors
///
/// Returns [`VerifyError`] when config resolution, introspection, rendering, or
/// reading the canonical artifact fails. Drift is returned as a successful
/// [`VerifyReport`].
pub fn verify(request: VerifyRequest) -> Result<VerifyReport, VerifyError> {
    let generated = generate(
        &RenderInput::Config(request.config_path),
        &request.environment,
        config::Overrides::default(),
        BuildDestination::Filesystem,
    )?;
    let destination = generated
        .output_path
        .expect("configured verification plans always have an output path");
    let comparison = output::compare(&destination, &generated.artifact, request.include_diff)?;
    Ok(VerifyReport {
        output_path: destination.into_path(),
        changes: comparison.changes,
        diff: comparison.diff,
    })
}

fn generate(
    input: &RenderInput,
    environment: &BTreeMap<String, String>,
    overrides: config::Overrides,
    destination: BuildDestination,
) -> Result<GeneratedArtifact, ArtifactBuildError> {
    let plan = match input {
        RenderInput::Config(config_path) => {
            let contents = fs::read_to_string(config_path).map_err(|source| {
                ArtifactBuildError::ReadConfig {
                    path: config_path.to_path_buf(),
                    source,
                }
            })?;
            config::resolve(&contents, config_path, environment, overrides)?
        }
        RenderInput::Sqlite(path) => {
            if overrides.source_selection.is_some() {
                return Err(ArtifactBuildError::Config(
                    ConfigError::SelectionWithoutConfig,
                ));
            }
            config::RenderPlan {
                sources: vec![dbmd_introspect::sqlite::SqliteSource::new(
                    SourceId::from_str("local")
                        .expect("the built-in one-off source ID is always valid"),
                    path,
                )
                .into()],
                project_root: None,
                repository_root: None,
                output_path: overrides.output_path,
                canonical_output_path: None,
                canonical_output_display_path: None,
                template_root: overrides.template_root,
                template_display_root: None,
                profile: "agent".to_string(),
                render_options: dbmd_render::RenderOptions::default(),
                required_environment: Vec::new(),
            }
        }
    };
    let output_path = match destination {
        BuildDestination::Stdout => {
            if plan.render_options.layout != dbmd_render::OutputLayout::SingleFile {
                return Err(ArtifactBuildError::StdoutRequiresSingleFile);
            }
            None
        }
        BuildDestination::Filesystem => {
            let output_path = plan
                .output_path
                .as_deref()
                .ok_or(ArtifactBuildError::MissingOutputPath)?;
            Some(output::validate(
                output_path,
                plan.render_options.layout,
                plan.project_root.as_deref(),
                plan.repository_root.as_deref(),
            )?)
        }
    };
    let renderer = match &plan.template_root {
        Some(root) => Renderer::from_template_root(root, &plan.profile)?,
        None => Renderer::embedded()?,
    };
    let snapshots = plan
        .sources
        .iter()
        .map(introspect::introspect)
        .collect::<Result<Vec<_>, _>>()?;
    let context = DatabaseContext::new(snapshots)?;
    let source_ids = context
        .sources()
        .iter()
        .map(|source| source.id.clone())
        .collect::<Vec<_>>();
    let artifact = renderer.render_with_options(&context, plan.render_options)?;
    Ok(GeneratedArtifact {
        output_path,
        source_ids,
        artifact,
    })
}

#[derive(Debug, Clone, Copy)]
enum BuildDestination {
    Filesystem,
    Stdout,
}

/// Why a canonical artifact could not be built in memory.
#[derive(Debug, Error)]
pub enum ArtifactBuildError {
    #[error("failed to read configuration `{path}`")]
    ReadConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Introspection(#[from] IntrospectionError),
    #[error(transparent)]
    Context(#[from] dbmd_core::DatabaseContextError),
    #[error(transparent)]
    Rendering(#[from] dbmd_render::RenderError),
    #[error(transparent)]
    OutputPreflight(#[from] OutputError),
    #[error("one-off rendering requires `--output` unless `--stdout` is selected")]
    MissingOutputPath,
    #[error("stdout is available only for single-file layout")]
    StdoutRequiresSingleFile,
}

/// Why a configured render operation failed.
#[derive(Debug, Error)]
pub enum RenderError {
    #[error(transparent)]
    Build(#[from] ArtifactBuildError),
    #[error(transparent)]
    Output(#[from] OutputError),
    #[error("stdout cannot be combined with an output path")]
    ConflictingDestination,
}

/// Why canonical verification could not produce a trustworthy comparison.
#[derive(Debug, Error)]
pub enum VerifyError {
    #[error(transparent)]
    Build(#[from] ArtifactBuildError),
    #[error(transparent)]
    Output(#[from] OutputError),
}
