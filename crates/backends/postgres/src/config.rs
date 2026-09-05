use dbmd_core::SourceId;
use serde::Deserialize;

use super::PostgresSource;

/// Committed PostgreSQL-specific fields inside one named source.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    url: String,
    display_name: Option<String>,
    #[serde(default)]
    include_cluster_objects: bool,
}

impl Config {
    /// Returns raw values that may contain environment references.
    #[must_use]
    pub fn environment_values(&self) -> Vec<&str> {
        vec![self.url.as_str()]
    }

    /// Resolves backend-specific values into a concrete PostgreSQL source.
    ///
    /// # Errors
    ///
    /// Returns the caller's value-resolution error.
    pub fn resolve<E>(
        &self,
        id: SourceId,
        mut resolve_value: impl FnMut(&str) -> Result<String, E>,
    ) -> Result<PostgresSource, E> {
        let mut source = PostgresSource::new(id, resolve_value(&self.url)?);
        if let Some(display_name) = &self.display_name {
            source = source.with_display_name(display_name);
        }
        source = source.with_cluster_objects(self.include_cluster_objects);
        Ok(source)
    }
}
