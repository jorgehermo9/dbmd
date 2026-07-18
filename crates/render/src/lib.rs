//! Presentation context and deterministic artifact rendering.

mod artifact;
mod context;

pub use artifact::{ArtifactPath, ArtifactPathError, RenderedArtifact};
pub use context::{
    code_block, inline_code, object_file_name, text, RenderColumn, RenderConstraint, RenderContext,
    RenderEnum, RenderFact, RenderFunction, RenderIndex, RenderNamespace, RenderSource,
    RenderTable, RenderTableDetails, RenderTrigger, RenderView,
};

use std::{fs, path::Path, path::PathBuf};

use minijinja::{context, Environment, UndefinedBehavior};
use thiserror::Error;

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
pub struct TemplateFile {
    /// Path relative to one profile directory.
    pub relative_path: &'static str,
    /// Internal renderer entrypoint name.
    pub template_name: &'static str,
    /// Editable template source.
    pub contents: &'static str,
}

/// Returns the complete built-in profile template set.
#[must_use]
pub fn embedded_template_files() -> &'static [TemplateFile] {
    const FILES: &[TemplateFile] = &[
        TemplateFile {
            relative_path: "single_file/database.md.j2",
            template_name: "database.md.j2",
            contents: include_str!("../templates/database.md.j2"),
        },
        TemplateFile {
            relative_path: "directory/enum.md.j2",
            template_name: "enum.md.j2",
            contents: include_str!("../templates/enum.md.j2"),
        },
        TemplateFile {
            relative_path: "directory/table.md.j2",
            template_name: "table.md.j2",
            contents: include_str!("../templates/table.md.j2"),
        },
        TemplateFile {
            relative_path: "directory/view.md.j2",
            template_name: "view.md.j2",
            contents: include_str!("../templates/view.md.j2"),
        },
        TemplateFile {
            relative_path: "directory/trigger.md.j2",
            template_name: "trigger.md.j2",
            contents: include_str!("../templates/trigger.md.j2"),
        },
        TemplateFile {
            relative_path: "directory/function.md.j2",
            template_name: "function.md.j2",
            contents: include_str!("../templates/function.md.j2"),
        },
        TemplateFile {
            relative_path: "directory/root.md.j2",
            template_name: "directory_root.md.j2",
            contents: include_str!("../templates/directory_root.md.j2"),
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
    pub fn embedded(backend_templates: &[TemplateFile]) -> Result<Self, RenderError> {
        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        for file in embedded_template_files() {
            env.add_template(file.template_name, file.contents)?;
        }
        for file in backend_templates {
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
    pub fn from_template_root(
        root: &Path,
        profile: &str,
        backend_templates: &[TemplateFile],
    ) -> Result<Self, RenderError> {
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
        for file in backend_templates {
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
    pub fn render(&self, context: &RenderContext) -> Result<RenderedArtifact, RenderError> {
        self.render_with_options(context, RenderOptions::default())
    }

    /// Renders an ordered database context with explicit presentation choices.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError`] when a template fails or a generated artifact
    /// path violates the relative-path invariant.
    pub fn render_with_options(
        &self,
        render_context: &RenderContext,
        options: RenderOptions,
    ) -> Result<RenderedArtifact, RenderError> {
        let nested = options.source_layout == SourceLayout::Nested
            || (options.source_layout == SourceLayout::Auto && render_context.sources.len() > 1);
        match options.layout {
            OutputLayout::SingleFile => {
                let template = self.env.get_template("database.md.j2")?;
                let markdown = template.render(context! { context => render_context })?;
                Ok(RenderedArtifact::SingleFile(markdown.into_bytes()))
            }
            OutputLayout::Directory => self.render_directory(render_context, nested),
        }
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
            .get_template(source.directory_template)?
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
