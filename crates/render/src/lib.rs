//! Presentation context and deterministic artifact rendering.

mod artifact;
mod context;

pub use artifact::{ArtifactPath, ArtifactPathError, RenderedArtifact};
pub use context::RenderContext;

use std::{fs, path::Path, path::PathBuf};

use dbmd_core::{DatabaseContext, SourceSnapshot, Table};
use minijinja::{context, Environment, UndefinedBehavior};
use thiserror::Error;

use crate::context::RenderTable;

/// File organization selected for a render operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputLayout {
    /// Emit one Markdown file.
    #[default]
    SingleFile,
    /// Emit an index and one Markdown file per schema object.
    Directory,
}

/// Whether source sections/directories are omitted when only one source exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceLayout {
    /// Add source nesting only when multiple sources are selected.
    #[default]
    Auto,
    /// Always add an explicit source section or directory.
    Nested,
}

/// Presentation choices for one render operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RenderOptions {
    /// File organization of the artifact.
    pub layout: OutputLayout,
    /// Source nesting policy within that layout.
    pub source_layout: SourceLayout,
}

/// One file in the complete embedded template set.
#[derive(Debug, Clone, Copy)]
pub struct EmbeddedTemplateFile {
    /// Path relative to one profile directory.
    pub relative_path: &'static str,
    /// Internal renderer entrypoint name.
    pub template_name: &'static str,
    /// Editable template source.
    pub contents: &'static str,
}

/// Returns the complete built-in profile template set.
#[must_use]
pub fn embedded_template_files() -> &'static [EmbeddedTemplateFile] {
    const FILES: &[EmbeddedTemplateFile] = &[
        EmbeddedTemplateFile {
            relative_path: "single_file/database.md.j2",
            template_name: "database.md.j2",
            contents: include_str!("../templates/database.md.j2"),
        },
        EmbeddedTemplateFile {
            relative_path: "directory/enum.md.j2",
            template_name: "enum.md.j2",
            contents: include_str!("../templates/enum.md.j2"),
        },
        EmbeddedTemplateFile {
            relative_path: "directory/table.md.j2",
            template_name: "table.md.j2",
            contents: include_str!("../templates/table.md.j2"),
        },
        EmbeddedTemplateFile {
            relative_path: "directory/view.md.j2",
            template_name: "view.md.j2",
            contents: include_str!("../templates/view.md.j2"),
        },
        EmbeddedTemplateFile {
            relative_path: "directory/trigger.md.j2",
            template_name: "trigger.md.j2",
            contents: include_str!("../templates/trigger.md.j2"),
        },
        EmbeddedTemplateFile {
            relative_path: "directory/function.md.j2",
            template_name: "function.md.j2",
            contents: include_str!("../templates/function.md.j2"),
        },
        EmbeddedTemplateFile {
            relative_path: "directory/root.md.j2",
            template_name: "directory_root.md.j2",
            contents: include_str!("../templates/directory_root.md.j2"),
        },
        EmbeddedTemplateFile {
            relative_path: "directory/index.md.j2",
            template_name: "directory_source.md.j2",
            contents: include_str!("../templates/directory_source.md.j2"),
        },
    ];
    FILES
}

/// Strict renderer backed by either the embedded or one complete custom template set.
pub struct Renderer {
    env: Environment<'static>,
}

impl Renderer {
    /// Compiles the built-in template set with strict undefined behavior.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError`] when an embedded template cannot compile.
    pub fn embedded() -> Result<Self, RenderError> {
        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        for file in embedded_template_files() {
            env.add_template(file.template_name, file.contents)?;
        }
        Ok(Self { env })
    }

    /// Loads and compiles a complete custom profile from `root/profile`.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError`] when the profile name is unsafe, a required file
    /// cannot be read, or a template cannot compile.
    pub fn from_template_root(root: &Path, profile: &str) -> Result<Self, RenderError> {
        if profile.is_empty()
            || !profile.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
            })
        {
            return Err(RenderError::InvalidProfile(profile.to_string()));
        }
        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        for file in embedded_template_files() {
            let path = root.join(profile).join(file.relative_path);
            let contents =
                fs::read_to_string(&path).map_err(|source| RenderError::ReadTemplate {
                    path: path.clone(),
                    source,
                })?;
            env.add_template_owned(file.template_name, contents)?;
        }
        Ok(Self { env })
    }

    /// Renders an ordered database context into one in-memory artifact.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError`] when template loading or execution fails.
    pub fn render(&self, database: &DatabaseContext) -> Result<RenderedArtifact, RenderError> {
        self.render_with_options(database, RenderOptions::default())
    }

    /// Renders an ordered database context with explicit presentation choices.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError`] when a template fails or a generated artifact
    /// path violates the relative-path invariant.
    pub fn render_with_options(
        &self,
        database: &DatabaseContext,
        options: RenderOptions,
    ) -> Result<RenderedArtifact, RenderError> {
        let nested = options.source_layout == SourceLayout::Nested
            || (options.source_layout == SourceLayout::Auto && database.sources().len() > 1);
        let render_context = RenderContext::new(database, nested);
        match options.layout {
            OutputLayout::SingleFile => {
                let template = self.env.get_template("database.md.j2")?;
                let markdown = template.render(context! { context => render_context })?;
                Ok(RenderedArtifact::SingleFile(markdown.into_bytes()))
            }
            OutputLayout::Directory => self.render_directory(&render_context, nested),
        }
    }

    /// Renders one source with the default single-source presentation.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError`] when template loading or execution fails.
    pub fn render_database(&self, source: &SourceSnapshot) -> Result<String, RenderError> {
        let database = DatabaseContext::new(vec![source.clone()])?;
        let RenderedArtifact::SingleFile(bytes) = self.render(&database)? else {
            return Err(RenderError::UnexpectedArtifact);
        };
        String::from_utf8(bytes).map_err(RenderError::Utf8)
    }

    /// Renders one table with the default object heading depth.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError`] when template loading or execution fails.
    pub fn render_table(&self, table: &Table) -> Result<String, RenderError> {
        let template = self.env.get_template("table.md.j2")?;
        let table = RenderTable::new(table, "###", "####");
        Ok(template.render(context! { table => table })?)
    }

    fn render_directory(
        &self,
        context: &RenderContext,
        nested: bool,
    ) -> Result<RenderedArtifact, RenderError> {
        use std::{collections::BTreeMap, str::FromStr};

        use serde::Serialize;

        #[derive(Serialize)]
        struct SourceLink<'a> {
            id: &'a str,
            name: &'a str,
            backend: &'a str,
            path: String,
        }

        let mut files = BTreeMap::new();
        if nested {
            let links = context
                .sources
                .iter()
                .map(|source| SourceLink {
                    id: &source.id,
                    name: &source.name,
                    backend: source.backend,
                    path: format!("{}/index.md", source.id),
                })
                .collect::<Vec<_>>();
            let root = self
                .env
                .get_template("directory_root.md.j2")?
                .render(context! { sources => links })?;
            files.insert(ArtifactPath::from_str("index.md")?, root.into_bytes());
            for source in &context.sources {
                self.render_source_directory(&mut files, source, Some(&source.id))?;
            }
        } else if let Some(source) = context.sources.first() {
            self.render_source_directory(&mut files, source, None)?;
        }
        Ok(RenderedArtifact::Directory(files))
    }

    fn render_source_directory(
        &self,
        files: &mut std::collections::BTreeMap<ArtifactPath, Vec<u8>>,
        source: &crate::context::RenderSource,
        prefix: Option<&str>,
    ) -> Result<(), RenderError> {
        let source_index = self
            .env
            .get_template("directory_source.md.j2")?
            .render(context! { source => source })?;
        let index_path = prefix.map_or_else(
            || "index.md".to_string(),
            |prefix| format!("{prefix}/index.md"),
        );
        files.insert(index_path.parse()?, source_index.into_bytes());

        let enum_template = self.env.get_template("enum.md.j2")?;
        for enum_type in &source.enums {
            let path = artifact_object_path(prefix, "enums", &enum_type.file_name)?;
            let mut enum_type = enum_type.clone();
            enum_type.heading = "#";
            let markdown = enum_template.render(context! { enum_type => enum_type })?;
            files.insert(path, markdown.into_bytes());
        }

        let table_template = self.env.get_template("table.md.j2")?;
        for table in &source.tables {
            let path = artifact_object_path(prefix, "tables", &table.file_name)?;
            let mut table = table.clone();
            table.heading = "#";
            table.detail_heading = "##";
            let markdown = table_template.render(context! { table => table })?;
            files.insert(path, markdown.into_bytes());
        }
        let view_template = self.env.get_template("view.md.j2")?;
        for view in &source.views {
            let path = artifact_object_path(prefix, "views", &view.file_name)?;
            let mut view = view.clone();
            view.heading = "#";
            let markdown = view_template.render(context! { view => view })?;
            files.insert(path, markdown.into_bytes());
        }
        let trigger_template = self.env.get_template("trigger.md.j2")?;
        for trigger in &source.triggers {
            let path = artifact_object_path(prefix, "triggers", &trigger.file_name)?;
            let mut trigger = trigger.clone();
            trigger.heading = "#";
            let markdown = trigger_template.render(context! { trigger => trigger })?;
            files.insert(path, markdown.into_bytes());
        }
        let function_template = self.env.get_template("function.md.j2")?;
        for function in &source.functions {
            let path = artifact_object_path(prefix, "functions", &function.file_name)?;
            let mut function = function.clone();
            function.heading = "#";
            let markdown = function_template.render(context! { function => function })?;
            files.insert(path, markdown.into_bytes());
        }
        Ok(())
    }
}

fn artifact_object_path(
    prefix: Option<&str>,
    kind: &str,
    file_name: &str,
) -> Result<ArtifactPath, ArtifactPathError> {
    prefix
        .map_or_else(
            || format!("{kind}/{file_name}"),
            |prefix| format!("{prefix}/{kind}/{file_name}"),
        )
        .parse()
}

/// Why a database context could not be rendered.
#[derive(Debug, Error)]
pub enum RenderError {
    /// A required custom template could not be read.
    #[error("failed to read required template `{path}`")]
    ReadTemplate {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// A profile name could escape or ambiguously address a template root.
    #[error("invalid template profile `{0}`")]
    InvalidProfile(String),
    /// Template compilation or execution failed.
    #[error("template error: {0}")]
    Template(#[from] minijinja::Error),
    /// The supplied database context violated a core invariant.
    #[error(transparent)]
    DatabaseContext(#[from] dbmd_core::DatabaseContextError),
    /// A rendered artifact that must be UTF-8 was not.
    #[error("rendered artifact was not UTF-8")]
    Utf8(#[source] std::string::FromUtf8Error),
    /// An internal single-file operation unexpectedly produced a directory.
    #[error("renderer produced an unexpected artifact layout")]
    UnexpectedArtifact,
    /// A generated directory entry was not a safe relative path.
    #[error(transparent)]
    ArtifactPath(#[from] ArtifactPathError),
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use dbmd_core::{
        ClickHouseColumn, ClickHouseTable, Column, ColumnBackend, Constraint, Index, Table,
        TableBackend,
    };

    use super::Renderer;

    #[test]
    fn renders_clickhouse_table_engine_details() {
        let mut settings = BTreeMap::new();
        settings.insert("index_granularity".to_string(), "8192".to_string());

        let table = Table {
            namespace: "analytics".to_string(),
            name: "events".to_string(),
            comment: Some("Raw event stream with deduplication".to_string()),
            columns: vec![Column {
                name: "event_type".to_string(),
                data_type: "LowCardinality(String)".to_string(),
                nullable: Some(false),
                default: None,
                comment: Some("Values: page_view, click, purchase".to_string()),
                backend: ColumnBackend::ClickHouse(ClickHouseColumn {
                    codec: None,
                    ttl: None,
                }),
            }],
            constraints: Vec::<Constraint>::new(),
            indexes: Vec::<Index>::new(),
            backend: TableBackend::ClickHouse(ClickHouseTable {
                engine: "ReplacingMergeTree".to_string(),
                engine_params: vec!["version".to_string(), "is_deleted".to_string()],
                order_by: vec!["user_id".to_string(), "occurred_at".to_string()],
                partition_by: Some("toYYYYMM(occurred_at)".to_string()),
                primary_key: vec!["user_id".to_string()],
                sample_by: None,
                ttl: Some("occurred_at + INTERVAL 90 DAY".to_string()),
                settings,
            }),
        };

        let out = Renderer::embedded()
            .expect("embedded templates should compile")
            .render_table(&table)
            .expect("ClickHouse table should render");

        assert!(out.contains("### `analytics.events`"));
        assert!(out.contains("**Engine:** `ReplacingMergeTree(version, is_deleted)`"));
        assert!(out.contains("**Primary key:** `user_id`"));
        assert!(out.contains("**Partition by:** `toYYYYMM(occurred_at)`"));
        assert!(out.contains("`index_granularity = 8192`"));
    }

    #[test]
    fn uses_a_longer_fence_when_stored_sql_contains_markdown_fences() {
        let table = Table {
            namespace: "main".to_string(),
            name: "odd|`name".to_string(),
            comment: None,
            columns: Vec::new(),
            constraints: Vec::new(),
            indexes: Vec::new(),
            backend: TableBackend::Sqlite(dbmd_core::SqliteTable {
                without_rowid: false,
                strict: false,
                definition: Some("CREATE TABLE \"odd|`name\" (id INTEGER); -- ```".to_string()),
                kind: dbmd_core::SqliteTableKind::Ordinary,
            }),
        };

        let out = Renderer::embedded()
            .expect("embedded templates should compile")
            .render_table(&table)
            .expect("SQLite table should render");

        assert!(out.contains("``main.odd\\|`name``"));
        assert!(out.contains("````sql\nCREATE TABLE"));
        assert!(out.contains("-- ```\n````"));
    }
}
