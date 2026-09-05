use std::{collections::BTreeMap, fmt, str::FromStr};

use thiserror::Error;

/// A validated, platform-independent relative path inside a directory artifact.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArtifactPath(String);

impl ArtifactPath {
    /// Returns this path with `/` separators.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ArtifactPath {
    type Err = ArtifactPathError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() || value.starts_with('/') || value.contains(['\\', '\0']) {
            return Err(ArtifactPathError(value.to_string()));
        }
        if value
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
        {
            return Err(ArtifactPathError(value.to_string()));
        }
        Ok(Self(value.to_string()))
    }
}

impl fmt::Display for ArtifactPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Why a relative artifact path was rejected.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid relative artifact path `{0}`")]
pub struct ArtifactPathError(String);

/// A complete generated artifact held in memory before writing or comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderedArtifact {
    /// One file containing the complete database context.
    SingleFile(Vec<u8>),
    /// A deterministic map from validated relative paths to file bytes.
    Directory(BTreeMap<ArtifactPath, Vec<u8>>),
}

impl RenderedArtifact {
    /// Returns the single-file bytes when this artifact uses that layout.
    #[must_use]
    pub fn as_single_file(&self) -> Option<&[u8]> {
        match self {
            Self::SingleFile(bytes) => Some(bytes),
            Self::Directory(_) => None,
        }
    }
}
