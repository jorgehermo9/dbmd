use std::{collections::BTreeMap, fs, path::PathBuf};

use dbmd_core::SourceId;

use crate::{config, output};

/// Inputs for diagnosing whether dbmd can operate in a project.
#[derive(Clone)]
pub struct DoctorRequest {
    config_path: PathBuf,
    environment: BTreeMap<String, String>,
    all_sources: bool,
    connections: bool,
}

impl std::fmt::Debug for DoctorRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DoctorRequest")
            .field("config_path", &self.config_path)
            .field(
                "environment_keys",
                &self.environment.keys().collect::<Vec<_>>(),
            )
            .field("all_sources", &self.all_sources)
            .field("connections", &self.connections)
            .finish()
    }
}

impl DoctorRequest {
    /// Creates a local-only diagnostic request using the process environment.
    #[must_use]
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
            environment: std::env::vars().collect(),
            all_sources: false,
            connections: false,
        }
    }

    /// Creates a local-only request with an explicit environment.
    #[must_use]
    pub fn with_environment(
        config_path: impl Into<PathBuf>,
        environment: BTreeMap<String, String>,
    ) -> Self {
        Self {
            config_path: config_path.into(),
            environment,
            all_sources: false,
            connections: false,
        }
    }

    /// Includes configured sources outside the canonical output selection.
    #[must_use]
    pub fn with_all_sources(mut self) -> Self {
        self.all_sources = true;
        self
    }

    /// Enables explicit database connection and introspection checks.
    #[must_use]
    pub fn with_connections(mut self) -> Self {
        self.connections = true;
        self
    }
}

/// Stable diagnostic stages used for grouped presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticStage {
    /// Configuration existence, syntax, schema, and selection.
    Configuration,
    /// Required environment references.
    Environment,
    /// Artifact path safety and writability.
    Output,
    /// Template availability and strict compilation.
    Templates,
    /// Explicit connection, permissions, compatibility, and introspection.
    Connection,
}

/// Outcome of one independent operational check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticStatus {
    /// The check completed successfully.
    Passed,
    /// The check found an actionable problem.
    Failed,
    /// The check was intentionally not requested or depended on failed input.
    Skipped,
}

/// One actionable doctor result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Operational stage that produced the result.
    pub stage: DiagnosticStage,
    /// Source identity for source-specific checks.
    pub source: Option<SourceId>,
    /// Check outcome.
    pub status: DiagnosticStatus,
    /// Credential-free human guidance.
    pub message: String,
}

/// Complete deterministic diagnostic report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorReport {
    /// Checks ordered by execution stage and configured source order.
    pub diagnostics: Vec<Diagnostic>,
}

impl DoctorReport {
    /// Returns true when no enabled or local prerequisite check failed.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|diagnostic| diagnostic.status != DiagnosticStatus::Failed)
    }
}

/// Diagnoses local prerequisites and, when explicitly enabled, source connectivity.
#[must_use]
pub fn doctor(request: DoctorRequest) -> DoctorReport {
    let contents = match fs::read_to_string(&request.config_path) {
        Ok(contents) => contents,
        Err(error) => {
            return report_error(
                DiagnosticStage::Configuration,
                format!(
                    "Could not read {} ({:?}). Fix the path and rerun doctor.",
                    request.config_path.display(),
                    error.kind()
                ),
            );
        }
    };
    let plan = match config::resolve_doctor(
        &contents,
        &request.config_path,
        &request.environment,
        request.all_sources,
    ) {
        Ok(plan) => plan,
        Err(error) => {
            return report_error(
                DiagnosticStage::Configuration,
                format!("{error}. Fix the project configuration and rerun doctor."),
            );
        }
    };

    let mut diagnostics = vec![Diagnostic {
        stage: DiagnosticStage::Configuration,
        source: None,
        status: DiagnosticStatus::Passed,
        message: format!(
            "Resolved {} selected source(s) from {}.",
            plan.render.sources.len(),
            request.config_path.display()
        ),
    }];
    diagnostics.push(environment_diagnostic(&plan));
    diagnostics.push(output_diagnostic(&plan));
    diagnostics.push(template_diagnostic(&plan));

    if request.connections {
        diagnostics.extend(plan.render.sources.iter().map(|source| {
            if let Some(names) = plan.missing_source_environment.get(source.id().as_str()) {
                return Diagnostic {
                    stage: DiagnosticStage::Connection,
                    source: Some(source.id().clone()),
                    status: DiagnosticStatus::Skipped,
                    message: format!(
                        "Connection check requires missing environment variables: {}.",
                        names.join(", ")
                    ),
                };
            }
            match dbmd_backends::introspect(source) {
                Ok(_) => Diagnostic {
                    stage: DiagnosticStage::Connection,
                    source: Some(source.id().clone()),
                    status: DiagnosticStatus::Passed,
                    message: "Connection and full schema introspection succeeded.".to_string(),
                },
                Err(error) => Diagnostic {
                    stage: DiagnosticStage::Connection,
                    source: Some(source.id().clone()),
                    status: DiagnosticStatus::Failed,
                    message: format!(
                        "{}. Check connectivity, metadata permissions, and backend compatibility.",
                        error.diagnostic()
                    ),
                },
            }
        }));
    } else {
        diagnostics.push(Diagnostic {
            stage: DiagnosticStage::Connection,
            source: None,
            status: DiagnosticStatus::Skipped,
            message: "Connection checks are disabled; rerun with --connect to enable them."
                .to_string(),
        });
    }

    DoctorReport { diagnostics }
}

fn environment_diagnostic(plan: &config::DoctorPlan) -> Diagnostic {
    if plan.missing_environment.is_empty() {
        Diagnostic {
            stage: DiagnosticStage::Environment,
            source: None,
            status: DiagnosticStatus::Passed,
            message: match plan.render.required_environment.as_slice() {
                [] => "No environment variables are required.".to_string(),
                names => format!(
                    "Required environment variables are set: {}.",
                    names.join(", ")
                ),
            },
        }
    } else {
        Diagnostic {
            stage: DiagnosticStage::Environment,
            source: None,
            status: DiagnosticStatus::Failed,
            message: format!(
                "Required environment variables are missing: {}. Set them and rerun doctor.",
                plan.missing_environment.join(", ")
            ),
        }
    }
}

fn output_diagnostic(plan: &config::DoctorPlan) -> Diagnostic {
    let display = plan
        .render
        .canonical_output_display_path
        .as_deref()
        .expect("configured doctor plans contain a safe output display path");
    if plan.missing_output_environment {
        return Diagnostic {
            stage: DiagnosticStage::Output,
            source: None,
            status: DiagnosticStatus::Skipped,
            message: format!(
                "Output preflight requires missing environment values referenced by {}.",
                display.display()
            ),
        };
    }
    let path = plan
        .render
        .output_path
        .as_deref()
        .expect("configured doctor plans always contain an output path");
    let result = output::validate(
        path,
        plan.render.render_options.layout,
        plan.render.project_root.as_deref(),
        plan.render.repository_root.as_deref(),
    )
    .and_then(|destination| output::probe_writable(&destination));
    match result {
        Ok(()) => Diagnostic {
            stage: DiagnosticStage::Output,
            source: None,
            status: DiagnosticStatus::Passed,
            message: format!("Output path is safe and writable: {}.", display.display()),
        },
        Err(_) => Diagnostic {
            stage: DiagnosticStage::Output,
            source: None,
            status: DiagnosticStatus::Failed,
            message: format!(
                "Output path failed safety or writability preflight: {}. Choose a safe regular-file or owned-directory destination.",
                display.display()
            ),
        },
    }
}

fn template_diagnostic(plan: &config::DoctorPlan) -> Diagnostic {
    let display = plan
        .render
        .template_display_root
        .as_deref()
        .map_or_else(|| "embedded".to_string(), |path| path.display().to_string());
    if plan.missing_template_environment {
        return Diagnostic {
            stage: DiagnosticStage::Templates,
            source: None,
            status: DiagnosticStatus::Skipped,
            message: format!(
                "Template preflight requires missing environment values referenced by {display}."
            ),
        };
    }
    let result = match &plan.render.template_root {
        Some(root) => dbmd_render::Renderer::from_template_root(
            root,
            &plan.render.profile,
            &dbmd_backends::all_template_files(),
        ),
        None => dbmd_render::Renderer::embedded(&dbmd_backends::all_template_files()),
    };
    match result {
        Ok(_) => Diagnostic {
            stage: DiagnosticStage::Templates,
            source: None,
            status: DiagnosticStatus::Passed,
            message: format!(
                "Template profile `{}` compiles from {display}.",
                plan.render.profile
            ),
        },
        Err(_) => Diagnostic {
            stage: DiagnosticStage::Templates,
            source: None,
            status: DiagnosticStatus::Failed,
            message: format!(
                "Template profile `{}` failed strict preflight from {display}. Restore every required entrypoint and fix template syntax.",
                plan.render.profile
            ),
        },
    }
}

fn report_error(stage: DiagnosticStage, message: String) -> DoctorReport {
    DoctorReport {
        diagnostics: vec![Diagnostic {
            stage,
            source: None,
            status: DiagnosticStatus::Failed,
            message,
        }],
    }
}
