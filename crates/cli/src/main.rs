use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use dbmd_app::RenderRequest;

#[derive(Debug, Parser)]
#[command(version, about = "Generate agent-readable database schema markdown")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Render the configured database artifact.
    Render {
        /// Project configuration path.
        #[arg(long, default_value = "dbmd.toml")]
        config: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Render { config } => {
            let report = dbmd_app::render(RenderRequest::new(config))?;
            println!(
                "Rendered {} source(s) to {} ({} bytes)",
                report.sources.len(),
                report.output_path.display(),
                report.bytes_written
            );
        }
    }

    Ok(())
}
