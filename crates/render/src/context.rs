use std::fmt::Write as _;

use serde::Serialize;

/// Presentation-only data supplied to templates.
///
/// This type contains no database catalog types, connection settings,
/// environment values, driver handles, or internal error values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderContext {
    pub version: u32,
    pub sources: Vec<RenderSource>,
}

impl RenderContext {
    /// Creates a versioned context from backend-prepared sources.
    #[must_use]
    pub fn new(sources: Vec<RenderSource>) -> Self {
        Self {
            version: 1,
            sources,
        }
    }
}

/// One backend-prepared source supplied to the shared artifact renderer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderSource {
    pub id: String,
    pub name: String,
    pub has_display_name: bool,
    pub backend: &'static str,
    pub single_file_template: &'static str,
    pub directory_template: &'static str,
    pub nested: bool,
    pub section_heading: &'static str,
    pub object_heading: &'static str,
    pub detail_heading: &'static str,
    pub namespaces: Vec<RenderNamespace>,
    pub enums: Vec<RenderEnum>,
    pub tables: Vec<RenderTable>,
    pub views: Vec<RenderView>,
    pub triggers: Vec<RenderTrigger>,
    pub functions: Vec<RenderFunction>,
}

/// One namespace row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderNamespace {
    pub name: String,
    pub comment: Option<String>,
}

/// One enum type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderEnum {
    pub heading: &'static str,
    pub qualified_name: String,
    pub file_name: String,
    pub comment: Option<String>,
    pub values: String,
}

/// One table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderTable {
    pub heading: &'static str,
    pub detail_heading: &'static str,
    pub qualified_name: String,
    pub file_name: String,
    pub comment: Option<String>,
    pub columns: Vec<RenderColumn>,
    pub constraints: Vec<RenderConstraint>,
    pub indexes: Vec<RenderIndex>,
    pub backend: RenderTableDetails,
}

/// One column row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderColumn {
    pub name: String,
    pub data_type: String,
    pub nullable: &'static str,
    pub default: String,
    pub notes: String,
}

/// One constraint row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderConstraint {
    pub name: String,
    pub kind: String,
    pub columns: String,
    pub details: String,
}

/// One index row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderIndex {
    pub name: String,
    pub terms: String,
    pub unique: &'static str,
    pub origin: String,
    pub predicate: String,
}

/// Backend-owned table facts represented without exposing its catalog type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderTableDetails {
    pub title: &'static str,
    pub facts: Vec<RenderFact>,
    pub notices: Vec<&'static str>,
    pub definition: Option<String>,
}

/// One labeled presentation fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderFact {
    pub label: &'static str,
    pub value: String,
}

impl RenderFact {
    #[must_use]
    pub fn new(label: &'static str, value: impl Into<String>) -> Self {
        Self {
            label,
            value: value.into(),
        }
    }
}

/// One view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderView {
    pub heading: &'static str,
    pub qualified_name: String,
    pub file_name: String,
    pub columns: Vec<RenderColumn>,
    pub definition: String,
}

/// One trigger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderTrigger {
    pub heading: &'static str,
    pub qualified_name: String,
    pub file_name: String,
    pub event: String,
    pub target: String,
    pub facts: Vec<RenderFact>,
    pub when_expression: Option<String>,
    pub definition: String,
}

/// One function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RenderFunction {
    pub heading: &'static str,
    pub qualified_name: String,
    pub file_name: String,
    pub comment: Option<String>,
    pub facts: Vec<RenderFact>,
    pub definition: Option<String>,
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
