#![doc = include_str!("../README.md")]

mod config;
mod output;

pub use config::ConfigError;
pub use output::OutputError;

use std::{collections::BTreeMap, fs, path::PathBuf};

use dbmd_core::{DatabaseContext, SourceId};
use dbmd_introspect::sqlite;
use dbmd_render::Renderer;
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

/// Resolves config, introspects selected sources, renders Markdown, and atomically replaces output.
///
/// # Errors
///
/// Returns [`RenderError`] when configuration, introspection, rendering, or output replacement fails.
pub fn render(request: RenderRequest) -> Result<RenderReport, RenderError> {
    let contents =
        fs::read_to_string(&request.config_path).map_err(|source| RenderError::ReadConfig {
            path: request.config_path.clone(),
            source,
        })?;
    let plan = config::resolve(&contents, &request.config_path, &request.environment)?;
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
    let rendered_sources = context
        .sources()
        .iter()
        .map(|source| renderer.render_database(source))
        .collect::<Result<Vec<_>, _>>()?;
    let markdown = rendered_sources.join("\n");
    output::replace_file(&plan.output_path, markdown.as_bytes())?;

    Ok(RenderReport {
        output_path: plan.output_path,
        sources: source_ids,
        bytes_written: markdown.len(),
    })
}

/// Why a configured render operation failed.
#[derive(Debug, Error)]
pub enum RenderError {
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
    #[error(transparent)]
    Output(#[from] OutputError),
}
