use std::{collections::BTreeMap, path::Path};

use dbmd_core::SourceId;
use serde::Deserialize;

use super::{DuckDbSource, DuckDbSourceError};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    path: String,
    display_name: Option<String>,
    #[serde(default)]
    attachments: BTreeMap<String, AttachmentConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct AttachmentConfig {
    path: String,
    #[serde(default = "default_read_only")]
    read_only: bool,
}

impl Config {
    #[must_use]
    pub fn environment_values(&self) -> Vec<&str> {
        std::iter::once(self.path.as_str())
            .chain(
                self.attachments
                    .values()
                    .map(|attachment| attachment.path.as_str()),
            )
            .collect()
    }

    /// Resolves paths and environment references into a concrete DuckDB source.
    ///
    /// # Errors
    ///
    /// Returns the caller's value-resolution error or an invalid source or
    /// attachment error.
    pub fn resolve<E>(
        &self,
        id: SourceId,
        base: &Path,
        mut resolve_value: impl FnMut(&str) -> Result<String, E>,
    ) -> Result<DuckDbSource, DuckDbConfigError<E>> {
        let path = resolve_value(&self.path).map_err(DuckDbConfigError::Value)?;
        let path = Path::new(&path);
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            base.join(path)
        };
        let mut source = DuckDbSource::new(id, path)?;
        if let Some(display_name) = &self.display_name {
            source = source.with_display_name(display_name);
        }
        for (name, attachment) in &self.attachments {
            let path = resolve_value(&attachment.path).map_err(DuckDbConfigError::Value)?;
            let path = resolve_path(base, &path);
            source = source.with_attached_database(name, path, attachment.read_only)?;
        }
        Ok(source)
    }
}

fn resolve_path(base: &Path, value: &str) -> std::path::PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

const fn default_read_only() -> bool {
    true
}

#[derive(Debug, thiserror::Error)]
pub enum DuckDbConfigError<E> {
    #[error(transparent)]
    Value(E),
    #[error(transparent)]
    Source(#[from] DuckDbSourceError),
}
