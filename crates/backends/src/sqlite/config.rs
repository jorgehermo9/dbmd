use std::{collections::BTreeMap, path::Path};

use dbmd_core::SourceId;
use serde::Deserialize;

use super::{SqliteSource, SqliteSourceError};

/// Committed SQLite-specific fields inside one named source.
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
}

impl Config {
    /// Returns raw values that may contain environment references.
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

    /// Resolves backend-specific values into a concrete SQLite source.
    ///
    /// # Errors
    ///
    /// Returns the caller's value-resolution error or an invalid attachment
    /// namespace error.
    pub fn resolve<E>(
        &self,
        id: SourceId,
        base: &Path,
        mut resolve_value: impl FnMut(&str) -> Result<String, E>,
    ) -> Result<SqliteSource, ConfigResolveError<E>> {
        let mut source = SqliteSource::new(id, resolve_path(base, &resolve_value(&self.path)?));
        if let Some(display_name) = &self.display_name {
            source = source.with_display_name(display_name);
        }
        for (namespace, attachment) in &self.attachments {
            let path = resolve_path(base, &resolve_value(&attachment.path)?);
            source = source
                .with_attached_database(namespace, path)
                .map_err(ConfigResolveError::Source)?;
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

/// Why SQLite-specific source configuration could not be resolved.
#[derive(Debug, thiserror::Error)]
pub enum ConfigResolveError<E> {
    #[error(transparent)]
    Value(#[from] E),
    #[error(transparent)]
    Source(SqliteSourceError),
}
