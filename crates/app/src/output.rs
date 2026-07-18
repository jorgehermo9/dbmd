use std::{
    fs,
    io::{self, Write},
    path::{Component, Path},
};

use tempfile::NamedTempFile;
use thiserror::Error;

pub(super) fn replace_file(path: &Path, contents: &[u8]) -> Result<(), OutputError> {
    validate(path)?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| OutputError::CreateParent {
        path: parent.to_path_buf(),
        source,
    })?;

    let mut temporary = NamedTempFile::new_in(parent).map_err(|source| OutputError::Temporary {
        path: parent.to_path_buf(),
        source,
    })?;
    temporary
        .write_all(contents)
        .and_then(|()| temporary.flush())
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|source| OutputError::Write {
            path: path.to_path_buf(),
            source,
        })?;
    temporary
        .persist(path)
        .map_err(|error| OutputError::Replace {
            path: path.to_path_buf(),
            source: error.error,
        })?;
    Ok(())
}

fn validate(path: &Path) -> Result<(), OutputError> {
    if path.as_os_str().is_empty()
        || path.file_name().is_none()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(OutputError::UnsafePath(path.to_path_buf()));
    }
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || metadata.is_dir() => {
            Err(OutputError::UnsafePath(path.to_path_buf()))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(OutputError::Inspect {
            path: path.to_path_buf(),
            source,
        }),
    }
}

#[derive(Debug, Error)]
pub enum OutputError {
    #[error("unsafe single-file output path `{0}`")]
    UnsafePath(std::path::PathBuf),
    #[error("failed to inspect output path `{path}`")]
    Inspect {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to create output parent `{path}`")]
    CreateParent {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to create temporary output beside `{path}`")]
    Temporary {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write temporary output for `{path}`")]
    Write {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to atomically replace output `{path}`")]
    Replace {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
}
