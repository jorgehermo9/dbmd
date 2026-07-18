use anyhow::Result;
use clap::{Parser, Subcommand};
use dbmd_core::{DatabaseSchema, PostgresTable, PostgresTableKind, Table, TableEngine};
use dbmd_render::Renderer;

#[derive(Debug, Parser)]
#[command(version, about = "Generate agent-readable database schema markdown")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Render a placeholder DATABASE.md from the embedded templates.
    Render,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Render => {
            let schema = placeholder_schema();
            let output = Renderer::embedded()?.render_database(&schema)?;
            print!("{output}");
        }
    }

    Ok(())
}

fn placeholder_schema() -> DatabaseSchema {
    let mut schema = DatabaseSchema::new("example");
    schema.tables.push(Table {
        schema: "public".to_string(),
        name: "users".to_string(),
        comment: Some("Placeholder table used until database drivers land.".to_string()),
        columns: Vec::new(),
        constraints: Vec::new(),
        indexes: Vec::new(),
        engine: TableEngine::Postgres(PostgresTable {
            table_kind: PostgresTableKind::Table,
            tablespace: None,
            inherits: Vec::new(),
            partition: None,
            row_level_security: false,
        }),
    });
    schema
}
