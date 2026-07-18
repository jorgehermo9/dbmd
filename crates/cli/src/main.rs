use std::{io::Write, path::PathBuf, process::ExitCode};

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use dbmd_app::{
    ArtifactChangeKind, InitCiRequest, InitRequest, InitTemplatesRequest, RenderOutput,
    RenderRequest, VerifyRequest,
};

#[derive(Debug, Parser)]
#[command(version, about = "Generate agent-readable database schema markdown")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a safe-to-commit project configuration.
    Init {
        #[command(subcommand)]
        command: Option<InitCommand>,
        /// Configuration path to create without replacement.
        #[arg(long, default_value = "dbmd.toml")]
        config: PathBuf,
    },
    /// Copy the complete built-in template profile for project customization.
    InitTemplates {
        /// New template root to create without overlaying existing files.
        #[arg(long, default_value = "templates/dbmd")]
        dir: PathBuf,
    },
    /// Render the configured database artifact.
    Render {
        /// Project configuration path.
        #[arg(long, conflicts_with_all = ["backend", "path"])]
        config: Option<PathBuf>,
        /// Source ID to render; repeat to replace configured selection in flag order.
        #[arg(long = "source", conflicts_with = "backend")]
        sources: Vec<String>,
        /// Backend for a configless one-off render.
        #[arg(long, value_enum, requires = "path")]
        backend: Option<OneOffBackend>,
        /// Database path for a configless one-off render.
        #[arg(long, requires = "backend")]
        path: Option<PathBuf>,
        /// Replace the configured output path, or set it for a one-off render.
        #[arg(long, conflicts_with = "stdout")]
        output: Option<PathBuf>,
        /// Print a single-file artifact without writing the configured output.
        #[arg(long, conflicts_with = "output")]
        stdout: bool,
        /// Complete custom template root containing the selected profile.
        #[arg(long)]
        template_root: Option<PathBuf>,
    },
    /// Check whether the canonical artifact exactly matches a fresh render.
    Verify {
        /// Project configuration path.
        #[arg(long, default_value = "dbmd.toml")]
        config: PathBuf,
        /// Print a complete unified diff when drift exists.
        #[arg(long)]
        diff: bool,
    },
}

#[derive(Debug, Subcommand)]
enum InitCommand {
    /// Create a GitHub Actions workflow that runs canonical verification.
    Ci {
        /// Workflow path to create.
        #[arg(long, default_value = ".github/workflows/dbmd.yml")]
        path: PathBuf,
        /// Explicitly replace an existing workflow.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OneOffBackend {
    Sqlite,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(exit_code) => exit_code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode> {
    match cli.command {
        Command::Init {
            command: Some(InitCommand::Ci { path, force }),
            ..
        } => {
            let report = dbmd_app::init_ci(InitCiRequest::new(path).with_overwrite(force))?;
            println!("Created {}", report.workflow_path.display());
            Ok(ExitCode::SUCCESS)
        }
        Command::Init {
            command: None,
            config,
        } => {
            let report = dbmd_app::init(InitRequest::new(config))?;
            println!("Created {}", report.config_path.display());
            if let Some(database) = report.detected_database {
                println!("Detected SQLite database: {}", database.display());
            } else {
                println!("Edit the example SQLite source before rendering.");
            }
            println!("Next:\n  dbmd render");
            Ok(ExitCode::SUCCESS)
        }
        Command::InitTemplates { dir } => {
            let report = dbmd_app::init_templates(InitTemplatesRequest::new(dir))?;
            println!(
                "Created {} with {} template files",
                report.template_root.display(),
                report.files.len()
            );
            Ok(ExitCode::SUCCESS)
        }
        Command::Render {
            config,
            sources,
            backend,
            path,
            output,
            stdout,
            template_root,
        } => {
            let request = match (backend, path) {
                (Some(OneOffBackend::Sqlite), Some(path)) => RenderRequest::sqlite(path),
                (None, None) => RenderRequest::new(config.unwrap_or_else(|| "dbmd.toml".into())),
                _ => unreachable!("clap validates one-off backend and path together"),
            };
            let request = if sources.is_empty() {
                request
            } else {
                request.with_sources(sources)
            };
            let request = match output {
                Some(path) => request.with_output_path(path),
                None => request,
            };
            let request = if stdout { request.to_stdout() } else { request };
            let request = match template_root {
                Some(root) => request.with_template_root(root),
                None => request,
            };
            let report = dbmd_app::render(request)?;
            match report.output {
                RenderOutput::Written {
                    path,
                    bytes_written,
                } => println!(
                    "Rendered {} source(s) to {} ({} bytes)",
                    report.sources.len(),
                    path.display(),
                    bytes_written
                ),
                RenderOutput::Stdout(contents) => {
                    std::io::stdout().write_all(&contents)?;
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Verify { config, diff } => {
            let report = dbmd_app::verify(VerifyRequest::new(config).with_diff(diff))?;
            if report.is_fresh() {
                println!(
                    "Canonical artifact is fresh: {}",
                    report.output_path.display()
                );
                return Ok(ExitCode::SUCCESS);
            }

            eprintln!("error: canonical artifact has drifted\n");
            eprintln!("Changed:");
            for change in report.changes {
                let status = match change.kind {
                    ArtifactChangeKind::Added => "added",
                    ArtifactChangeKind::Modified => "modified",
                    ArtifactChangeKind::Deleted => "deleted",
                };
                eprintln!("  {status:<9} {}", change.path);
            }
            eprintln!("\nRun:\n  dbmd render");
            if let Some(diff) = report.diff {
                eprintln!("\n{diff}");
            }
            Ok(ExitCode::FAILURE)
        }
    }
}
