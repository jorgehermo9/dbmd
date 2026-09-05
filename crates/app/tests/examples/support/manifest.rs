use std::{collections::BTreeSet, path::Path};

use anyhow::{bail, ensure, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Suite {
    pub version: u32,
    pub example: Vec<Example>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Example {
    pub path: String,
    #[serde(default)]
    pub cli: bool,
    #[serde(default)]
    pub drift_check: bool,
    pub source: Vec<ExampleSource>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExampleSource {
    pub id: String,
    pub backend: Backend,
    pub schema_dir: String,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    Clickhouse,
    Duckdb,
    Mariadb,
    Mysql,
    Postgres,
    Sqlite,
}

impl Backend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clickhouse => "clickhouse",
            Self::Duckdb => "duckdb",
            Self::Mariadb => "mariadb",
            Self::Mysql => "mysql",
            Self::Postgres => "postgres",
            Self::Sqlite => "sqlite",
        }
    }

    pub fn is_embedded(self) -> bool {
        matches!(self, Self::Duckdb | Self::Sqlite)
    }
}

impl Suite {
    pub fn parse(contents: &str) -> Result<Self> {
        let suite: Self = toml::from_str(contents)?;
        suite.validate()?;
        Ok(suite)
    }

    pub fn example(&self, path: &str) -> &Example {
        self.example
            .iter()
            .find(|example| example.path == path)
            .unwrap_or_else(|| panic!("example `{path}` should be registered"))
    }

    fn validate(&self) -> Result<()> {
        ensure!(
            self.version == 1,
            "unsupported example suite version {}",
            self.version
        );
        ensure!(
            !self.example.is_empty(),
            "example suite must register at least one project"
        );

        let mut paths = BTreeSet::new();
        for example in &self.example {
            ensure_safe_relative(&example.path, "example path")?;
            ensure!(
                paths.insert(&example.path),
                "duplicate example path `{}`",
                example.path
            );
            ensure!(
                !example.source.is_empty(),
                "example `{}` must declare at least one source",
                example.path
            );

            let mut source_ids = BTreeSet::new();
            for source in &example.source {
                ensure_safe_relative(&source.schema_dir, "schema directory")?;
                ensure!(
                    !source.id.is_empty(),
                    "example `{}` has an empty source ID",
                    example.path
                );
                ensure!(
                    source_ids.insert(&source.id),
                    "example `{}` repeats source `{}`",
                    example.path,
                    source.id
                );
            }
        }
        Ok(())
    }
}

fn ensure_safe_relative(value: &str, label: &str) -> Result<()> {
    let path = Path::new(value);
    if value.is_empty() || path.is_absolute() {
        bail!("{label} `{value}` must be a non-empty relative path");
    }
    if path
        .components()
        .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        bail!("{label} `{value}` must not contain traversal or special components");
    }
    Ok(())
}
