//! ClickHouse source configuration.

use dbmd_core::SourceId;
use serde::Deserialize;

use super::ClickHouseSource;

/// Committed ClickHouse-specific fields inside one named source.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    url: String,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    display_name: Option<String>,
}

impl Config {
    /// Returns raw values that may contain environment references.
    #[must_use]
    pub fn environment_values(&self) -> Vec<&str> {
        std::iter::once(self.url.as_str())
            .chain(self.database.as_deref())
            .chain(self.username.as_deref())
            .chain(self.password.as_deref())
            .collect()
    }

    /// Resolves backend fields into a concrete ClickHouse source.
    ///
    /// # Errors
    ///
    /// Returns the caller's value-resolution error.
    pub fn resolve<E>(
        &self,
        id: SourceId,
        mut resolve_value: impl FnMut(&str) -> Result<String, E>,
    ) -> Result<ClickHouseSource, E> {
        let mut source = ClickHouseSource::new(id, resolve_value(&self.url)?);
        if let Some(database) = &self.database {
            source = source.with_database(resolve_value(database)?);
        }
        if self.username.is_some() || self.password.is_some() {
            source = source.with_credentials(
                self.username
                    .as_deref()
                    .map(&mut resolve_value)
                    .transpose()?,
                self.password
                    .as_deref()
                    .map(&mut resolve_value)
                    .transpose()?,
            );
        }
        if let Some(display_name) = &self.display_name {
            source = source.with_display_name(display_name);
        }
        Ok(source)
    }
}
