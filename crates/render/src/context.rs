use std::fmt::Write as _;

use minijinja::value::Value;
use serde::Serialize;

/// Presentation-only data supplied to templates.
///
/// This type contains no database catalog types, connection settings,
/// environment values, driver handles, or internal error values.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub struct RenderContext {
    version: u32,
    sources: Vec<RenderSource>,
}

impl RenderContext {
    /// Creates a versioned context from backend-prepared sources.
    #[must_use]
    pub fn new(sources: Vec<RenderSource>) -> Self {
        Self {
            version: 2,
            sources,
        }
    }

    /// Returns backend-prepared sources in deterministic operation order.
    #[must_use]
    pub fn sources(&self) -> &[RenderSource] {
        &self.sources
    }
}

/// One backend-prepared source supplied to the shared artifact renderer.
///
/// `data` is deliberately opaque to `dbmd-render`. Its shape and semantics are
/// owned by the backend template named by this source.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub struct RenderSource {
    id: String,
    name: String,
    has_display_name: bool,
    backend: &'static str,
    single_file_template: &'static str,
    directory_template: &'static str,
    nested: bool,
    data: Value,
    #[serde(skip)]
    objects: Vec<RenderObject>,
}

impl RenderSource {
    /// Starts named construction of a backend-prepared source.
    #[must_use]
    pub fn builder(
        id: impl Into<String>,
        backend: &'static str,
        templates: (&'static str, &'static str),
        data: impl Serialize,
    ) -> RenderSourceBuilder {
        RenderSourceBuilder {
            id: id.into(),
            display_name: None,
            backend,
            single_file_template: templates.0,
            directory_template: templates.1,
            nested: false,
            data: Value::from_serialize(data),
            objects: Vec::new(),
        }
    }

    #[must_use]
    /// Returns the stable source ID used for selection and nested paths.
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    /// Returns the Markdown-ready explicit or fallback source name.
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    /// Returns the stable backend tag.
    pub const fn backend(&self) -> &'static str {
        self.backend
    }

    #[must_use]
    /// Returns the backend entrypoint for single-file rendering.
    pub const fn single_file_template(&self) -> &'static str {
        self.single_file_template
    }

    #[must_use]
    /// Returns the backend entrypoint for a directory source index.
    pub const fn directory_template(&self) -> &'static str {
        self.directory_template
    }

    #[must_use]
    /// Returns whether this source is rendered with explicit source nesting.
    pub const fn nested(&self) -> bool {
        self.nested
    }

    #[must_use]
    /// Returns backend-declared directory object artifacts in render order.
    pub fn objects(&self) -> &[RenderObject] {
        &self.objects
    }
}

/// Named construction for a backend-prepared render source.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RenderSourceBuilder {
    id: String,
    display_name: Option<String>,
    backend: &'static str,
    single_file_template: &'static str,
    directory_template: &'static str,
    nested: bool,
    data: Value,
    objects: Vec<RenderObject>,
}

impl RenderSourceBuilder {
    /// Sets an optional Markdown-ready display name.
    #[must_use]
    pub fn display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    /// Sets whether explicit source nesting is active.
    #[must_use]
    pub const fn nested(mut self, nested: bool) -> Self {
        self.nested = nested;
        self
    }

    /// Sets backend-declared directory object artifacts in render order.
    #[must_use]
    pub fn objects(mut self, objects: Vec<RenderObject>) -> Self {
        self.objects = objects;
        self
    }

    /// Finishes the render source, deriving the fallback name from its ID.
    #[must_use]
    pub fn build(self) -> RenderSource {
        let has_display_name = self.display_name.is_some();
        let name = self.display_name.unwrap_or_else(|| inline_code(&self.id));
        RenderSource {
            id: self.id,
            name,
            has_display_name,
            backend: self.backend,
            single_file_template: self.single_file_template,
            directory_template: self.directory_template,
            nested: self.nested,
            data: self.data,
            objects: self.objects,
        }
    }
}

/// One backend-declared object file in a directory-layout artifact.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub struct RenderObject {
    relative_path: String,
    template: &'static str,
    data: Value,
}

impl RenderObject {
    /// Creates one object file from a validated-at-render-time relative path,
    /// an embedded/custom template name, and backend-owned presentation data.
    #[must_use]
    pub fn new(
        relative_path: impl Into<String>,
        template: &'static str,
        data: impl Serialize,
    ) -> Self {
        Self {
            relative_path: relative_path.into(),
            template,
            data: Value::from_serialize(data),
        }
    }

    #[must_use]
    /// Returns the source-relative artifact path declared by the backend.
    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }

    #[must_use]
    /// Returns the template selected for this object artifact.
    pub const fn template(&self) -> &'static str {
        self.template
    }

    #[must_use]
    /// Returns the opaque backend-owned presentation payload.
    pub fn data(&self) -> &Value {
        &self.data
    }
}

/// Escapes a value as Markdown inline code safe for a table cell.
#[must_use]
pub fn inline_code(value: &str) -> String {
    let longest_run = longest_backtick_run(value);
    let fence = "`".repeat(longest_run.saturating_add(1).max(1));
    let padding = value.starts_with('`') || value.ends_with('`');
    let rendered = if padding {
        format!("{fence} {value} {fence}")
    } else {
        format!("{fence}{value}{fence}")
    };
    table_cell(&rendered)
}

/// Wraps stored source text in a safe Markdown code fence.
#[must_use]
pub fn code_block(language: &str, value: &str) -> String {
    let fence = "`".repeat(longest_backtick_run(value).saturating_add(1).max(3));
    format!("{fence}{language}\n{value}\n{fence}")
}

/// Escapes arbitrary text for a Markdown table cell.
#[must_use]
pub fn text(value: &str) -> String {
    table_cell(value)
}

/// Returns the deterministic artifact filename for a qualified object.
#[must_use]
pub fn object_file_name(namespace: &str, name: &str) -> String {
    format!("{}.{}.md", path_component(namespace), path_component(name))
}

fn longest_backtick_run(value: &str) -> usize {
    value
        .split(|character| character != '`')
        .map(str::len)
        .max()
        .unwrap_or(0)
}

fn table_cell(value: &str) -> String {
    value
        .replace('|', "\\|")
        .replace("\r\n", "<br>")
        .replace(['\r', '\n'], "<br>")
}

fn path_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-') {
            encoded.push(char::from(byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}
