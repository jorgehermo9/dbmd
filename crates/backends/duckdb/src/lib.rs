#![doc = include_str!("../README.md")]

mod catalog;
mod config;
mod introspect;
mod render;

pub use catalog::*;
pub use config::{Config, DuckDbConfigError};
pub use introspect::{introspect, DuckDbSource, DuckDbSourceError, IntrospectionError};

use dbmd_core::SourceId;
use dbmd_render::{RenderSource, TemplateFile};

#[must_use]
pub fn render_source(
    id: &SourceId,
    display_name: Option<&str>,
    catalog: &Catalog,
    nested: bool,
) -> RenderSource {
    render::source(id, display_name, catalog, nested)
}

#[must_use]
pub const fn template_files() -> &'static [TemplateFile] {
    render::TEMPLATES
}
