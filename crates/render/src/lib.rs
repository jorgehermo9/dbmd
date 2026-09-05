//! Presentation context and deterministic artifact rendering.

mod artifact;
mod context;

pub use artifact::{ArtifactPath, ArtifactPathError, RenderedArtifact};
pub use context::{
    code_block, inline_code, object_file_name, text, RenderContext, RenderObject, RenderSource,
    RenderSourceBuilder,
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
#[non_exhaustive]
pub struct TemplateFile {
    /// Path relative to one profile directory.
    pub relative_path: &'static str,
    /// Internal renderer entrypoint name.
    pub template_name: &'static str,
    /// Editable template source.
    pub contents: &'static str,
}

impl TemplateFile {
    /// Declares one embedded/custom-profile template entry.
    #[must_use]
    pub const fn new(
        relative_path: &'static str,
        template_name: &'static str,
        contents: &'static str,
    ) -> Self {
        Self {
            relative_path,
            template_name,
            contents,
        }
    }
}

/// Returns the complete built-in profile template set.
#[must_use]
pub fn embedded_template_files() -> &'static [TemplateFile] {
    const FILES: &[TemplateFile] = &[
        TemplateFile::new(
            "single_file/database.md.j2",
            "database.md.j2",
            include_str!("../templates/database.md.j2"),
        ),
        TemplateFile::new(
            "directory/enum.md.j2",
            "enum.md.j2",
            include_str!("../templates/enum.md.j2"),
        ),
        TemplateFile::new(
            "directory/table.md.j2",
            "table.md.j2",
            include_str!("../templates/table.md.j2"),
        ),
        TemplateFile::new(
            "directory/view.md.j2",
            "view.md.j2",
            include_str!("../templates/view.md.j2"),
        ),
        TemplateFile::new(
            "directory/trigger.md.j2",
            "trigger.md.j2",
            include_str!("../templates/trigger.md.j2"),
        ),
        TemplateFile::new(
            "directory/function.md.j2",
            "function.md.j2",
            include_str!("../templates/function.md.j2"),
        ),
        TemplateFile::new(
            "directory/root.md.j2",
            "directory_root.md.j2",
            include_str!("../templates/directory_root.md.j2"),
        ),
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
            || (options.source_layout == SourceLayout::Auto && render_context.sources().len() > 1);
        if render_context
            .sources()
            .iter()
            .any(|source| source.nested() != nested)
        {
            return Err(RenderError::InconsistentSourceLayout);
        }
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
                .sources()
                .iter()
                .map(|source| SourceLink {
                    id: source.id(),
                    name: source.name(),
                    backend: source.backend(),
                    path: format!("{}/index.md", source.id()),
                })
                .collect::<Vec<_>>();
            let root = self
                .env
                .get_template("directory_root.md.j2")?
                .render(context! { sources => links })?;
            insert_artifact(
                &mut files,
                ArtifactPath::from_str("index.md")?,
                root.into_bytes(),
            )?;
            for source in context.sources() {
                self.render_source_directory(&mut files, source, Some(source.id()))?;
            }
        } else if let Some(source) = context.sources().first() {
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
            .get_template(source.directory_template())?
            .render(context! { source => source })?;
        let index_path = prefix.map_or_else(
            || "index.md".to_string(),
            |prefix| format!("{prefix}/index.md"),
        );
        insert_artifact(files, index_path.parse()?, source_index.into_bytes())?;

        for object in source.objects() {
            let path = artifact_object_path(prefix, object.relative_path())?;
            let markdown = self.env.get_template(object.template())?.render(context! {
                source => source,
                object => object.data(),
                heading => "#",
                detail_heading => "##",
            })?;
            insert_artifact(files, path, markdown.into_bytes())?;
        }
        Ok(())
    }
}

fn insert_artifact(
    files: &mut std::collections::BTreeMap<ArtifactPath, Vec<u8>>,
    path: ArtifactPath,
    contents: Vec<u8>,
) -> Result<(), RenderError> {
    use std::collections::btree_map::Entry;

    match files.entry(path) {
        Entry::Vacant(entry) => {
            entry.insert(contents);
            Ok(())
        }
        Entry::Occupied(entry) => Err(RenderError::DuplicateArtifactPath(
            entry.key().as_str().to_string(),
        )),
    }
}

fn artifact_object_path(
    prefix: Option<&str>,
    relative_path: &str,
) -> Result<ArtifactPath, ArtifactPathError> {
    prefix
        .map_or_else(
            || relative_path.to_string(),
            |prefix| format!("{prefix}/{relative_path}"),
        )
        .parse()
}

/// Why a database context could not be rendered.
#[derive(Debug, Error)]
#[non_exhaustive]
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
    /// Backend-prepared source headings do not match the selected source layout.
    #[error("render context source nesting does not match the selected source layout")]
    InconsistentSourceLayout,
    /// Two backend declarations resolve to the same artifact path.
    #[error("render manifest declares artifact path `{0}` more than once")]
    DuplicateArtifactPath(String),
    /// A generated directory entry was not a safe relative path.
    #[error(transparent)]
    ArtifactPath(#[from] ArtifactPathError),
}
