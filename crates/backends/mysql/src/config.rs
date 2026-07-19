use dbmd_core::SourceId;
use serde::Deserialize;

use super::MysqlSource;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    url: String,
    schema: Option<String>,
    display_name: Option<String>,
}

impl Config {
    #[must_use]
    pub fn environment_values(&self) -> Vec<&str> {
        std::iter::once(self.url.as_str())
            .chain(self.schema.as_deref())
            .collect()
    }

    /// Resolves connection and schema fields into a concrete MySQL source.
    ///
    /// # Errors
    ///
    /// Returns the caller's value-resolution error.
    pub fn resolve<E>(
        &self,
        id: SourceId,
        mut resolve_value: impl FnMut(&str) -> Result<String, E>,
    ) -> Result<MysqlSource, E> {
        let mut source = MysqlSource::new(id, resolve_value(&self.url)?);
        if let Some(schema) = &self.schema {
            source = source.with_schema(resolve_value(schema)?);
        }
        if let Some(display_name) = &self.display_name {
            source = source.with_display_name(display_name);
        }
        Ok(source)
    }
}
