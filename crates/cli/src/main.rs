use std::{path::PathBuf, process::ExitCode};

use anyhow::Result;
use clap::{Parser, Subcommand};
use dbmd_app::{ArtifactChangeKind, InitRequest, RenderRequest, VerifyRequest};

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
        /// Configuration path to create without replacement.
        #[arg(long, default_value = "dbmd.toml")]
        config: PathBuf,
    },
    /// Render the configured database artifact.
    Render {
        /// Project configuration path.
        #[arg(long, default_value = "dbmd.toml")]
        config: PathBuf,
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
        Command::Init { config } => {
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
        Command::Render { config } => {
            let report = dbmd_app::render(RenderRequest::new(config))?;
            println!(
                "Rendered {} source(s) to {} ({} bytes)",
                report.sources.len(),
                report.output_path.display(),
                report.bytes_written
            );
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
