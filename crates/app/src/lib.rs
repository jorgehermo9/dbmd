#![doc = include_str!("../README.md")]

mod config;
mod init;
mod output;

pub use config::ConfigError;
pub use init::{init, InitError, InitReport, InitRequest};
pub use output::OutputError;

use std::{collections::BTreeMap, fs, path::PathBuf};

use dbmd_core::{DatabaseContext, SourceId};
use dbmd_introspect::sqlite;
use dbmd_render::{RenderedArtifact, Renderer};
use thiserror::Error;

/// Inputs for one configured render operation.
#[derive(Debug, Clone)]
pub struct RenderRequest {
    config_path: PathBuf,
    environment: BTreeMap<String, String>,
}

impl RenderRequest {
    /// Creates a request using the current process environment.
    #[must_use]
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
            environment: std::env::vars().collect(),
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
        }
    }
}

/// Observable result of a completed render operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReport {
    pub output_path: PathBuf,
    pub sources: Vec<SourceId>,
    pub bytes_written: usize,
}

/// Inputs for one canonical verification operation.
#[derive(Debug, Clone)]
pub struct VerifyRequest {
    config_path: PathBuf,
    environment: BTreeMap<String, String>,
    include_diff: bool,
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
    output_path: PathBuf,
    source_ids: Vec<SourceId>,
    artifact: RenderedArtifact,
}

/// Resolves config, introspects selected sources, renders Markdown, and atomically replaces output.
///
/// # Errors
///
/// Returns [`RenderError`] when configuration, introspection, rendering, or output replacement fails.
pub fn render(request: RenderRequest) -> Result<RenderReport, RenderError> {
    let generated = generate(&request.config_path, &request.environment)?;
    let bytes_written = output::replace(&generated.output_path, &generated.artifact)?;

    Ok(RenderReport {
        output_path: generated.output_path,
        sources: generated.source_ids,
        bytes_written,
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
    let generated = generate(&request.config_path, &request.environment)?;
    let comparison = output::compare(
        &generated.output_path,
        &generated.artifact,
        request.include_diff,
    )?;
    Ok(VerifyReport {
        output_path: generated.output_path,
        changes: comparison.changes,
        diff: comparison.diff,
    })
}

fn generate(
    config_path: &std::path::Path,
    environment: &BTreeMap<String, String>,
) -> Result<GeneratedArtifact, ArtifactBuildError> {
    let contents =
        fs::read_to_string(config_path).map_err(|source| ArtifactBuildError::ReadConfig {
            path: config_path.to_path_buf(),
            source,
        })?;
    let plan = config::resolve(&contents, config_path, environment)?;
    let renderer = Renderer::embedded()?;
    let snapshots = plan
        .sources
        .iter()
        .map(sqlite::introspect)
        .collect::<Result<Vec<_>, _>>()?;
    let context = DatabaseContext::new(snapshots)?;
    let source_ids = context
        .sources()
        .iter()
        .map(|source| source.id.clone())
        .collect::<Vec<_>>();
    let artifact = renderer.render_with_options(&context, plan.render_options)?;
    Ok(GeneratedArtifact {
        output_path: plan.output_path,
        source_ids,
        artifact,
    })
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
    Introspection(#[from] sqlite::IntrospectionError),
    #[error(transparent)]
    Context(#[from] dbmd_core::DatabaseContextError),
    #[error(transparent)]
    Rendering(#[from] dbmd_render::RenderError),
}

/// Why a configured render operation failed.
#[derive(Debug, Error)]
pub enum RenderError {
    #[error(transparent)]
    Build(#[from] ArtifactBuildError),
    #[error(transparent)]
    Output(#[from] OutputError),
}

/// Why canonical verification could not produce a trustworthy comparison.
#[derive(Debug, Error)]
pub enum VerifyError {
    #[error(transparent)]
    Build(#[from] ArtifactBuildError),
    #[error(transparent)]
    Output(#[from] OutputError),
}
