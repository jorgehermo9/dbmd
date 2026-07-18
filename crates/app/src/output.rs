use std::{
    fs,
    io::{self, Write},
    path::{Component, Path, PathBuf},
};

use dbmd_render::RenderedArtifact;
use similar::TextDiff;
use tempfile::{NamedTempFile, TempDir};
use thiserror::Error;

pub(super) fn replace(path: &Path, artifact: &RenderedArtifact) -> Result<usize, OutputError> {
    match artifact {
        RenderedArtifact::SingleFile(contents) => {
            replace_file(path, contents)?;
            Ok(contents.len())
        }
        RenderedArtifact::Directory(files) => {
            replace_directory(path, files)?;
            Ok(files.values().map(Vec::len).sum())
        }
    }
}

pub(super) struct Comparison {
    pub changes: Vec<crate::ArtifactChange>,
    pub diff: Option<String>,
}

pub(super) fn compare(
    path: &Path,
    artifact: &RenderedArtifact,
    include_diff: bool,
) -> Result<Comparison, OutputError> {
    match artifact {
        RenderedArtifact::SingleFile(fresh) => compare_file(path, fresh, include_diff),
        RenderedArtifact::Directory(fresh) => compare_directory(path, fresh, include_diff),
    }
}

fn compare_file(path: &Path, fresh: &[u8], include_diff: bool) -> Result<Comparison, OutputError> {
    validate_file(path)?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("artifact")
        .to_string();
    let canonical = match fs::read(path) {
        Ok(contents) => Some(contents),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(source) => {
            return Err(OutputError::Read {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    let kind = match canonical.as_deref() {
        None => Some(crate::ArtifactChangeKind::Added),
        Some(contents) if contents != fresh => Some(crate::ArtifactChangeKind::Modified),
        Some(_) => None,
    };
    let changes = kind
        .map(|kind| {
            vec![crate::ArtifactChange {
                path: name.clone(),
                kind,
            }]
        })
        .unwrap_or_default();
    let diff = if include_diff && !changes.is_empty() {
        Some(unified_diff(
            canonical.as_deref().unwrap_or_default(),
            fresh,
            &name,
        ))
    } else {
        None
    };
    Ok(Comparison { changes, diff })
}

fn compare_directory(
    path: &Path,
    fresh: &std::collections::BTreeMap<dbmd_render::ArtifactPath, Vec<u8>>,
    include_diff: bool,
) -> Result<Comparison, OutputError> {
    validate_directory(path)?;
    let mut canonical = std::collections::BTreeMap::new();
    match fs::symlink_metadata(path) {
        Ok(_) => read_directory(path, path, &mut canonical)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(OutputError::Inspect {
                path: path.to_path_buf(),
                source,
            });
        }
    }
    let fresh = fresh
        .iter()
        .map(|(path, contents)| (path.as_str().to_string(), contents.as_slice()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let paths = canonical
        .keys()
        .chain(fresh.keys())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let mut changes = Vec::new();
    let mut diff = String::new();
    for relative in paths {
        let canonical_bytes = canonical.get(&relative).map(Vec::as_slice);
        let fresh_bytes = fresh.get(&relative).copied();
        let kind = match (canonical_bytes, fresh_bytes) {
            (None, Some(_)) => Some(crate::ArtifactChangeKind::Added),
            (Some(_), None) => Some(crate::ArtifactChangeKind::Deleted),
            (Some(left), Some(right)) if left != right => Some(crate::ArtifactChangeKind::Modified),
            _ => None,
        };
        if let Some(kind) = kind {
            changes.push(crate::ArtifactChange {
                path: relative.clone(),
                kind,
            });
            if include_diff {
                if !diff.is_empty() {
                    diff.push('\n');
                }
                diff.push_str(&unified_diff(
                    canonical_bytes.unwrap_or_default(),
                    fresh_bytes.unwrap_or_default(),
                    &relative,
                ));
            }
        }
    }
    Ok(Comparison {
        changes,
        diff: (include_diff && !diff.is_empty()).then_some(diff),
    })
}

fn read_directory(
    root: &Path,
    directory: &Path,
    files: &mut std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<(), OutputError> {
    let mut entries = fs::read_dir(directory)
        .map_err(|source| OutputError::Read {
            path: directory.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| OutputError::Read {
            path: directory.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let file_type = entry.file_type().map_err(|source| OutputError::Read {
            path: entry.path(),
            source,
        })?;
        if file_type.is_symlink() {
            return Err(OutputError::UnsafePath(entry.path()));
        }
        if file_type.is_dir() {
            read_directory(root, &entry.path(), files)?;
        } else if file_type.is_file() {
            let relative = entry
                .path()
                .strip_prefix(root)
                .map_err(|_| OutputError::UnsafePath(entry.path()))?
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/");
            let contents = fs::read(entry.path()).map_err(|source| OutputError::Read {
                path: entry.path(),
                source,
            })?;
            files.insert(relative, contents);
        } else {
            return Err(OutputError::UnsafePath(entry.path()));
        }
    }
    Ok(())
}

fn unified_diff(canonical: &[u8], fresh: &[u8], path: &str) -> String {
    let canonical = String::from_utf8_lossy(canonical);
    let fresh = String::from_utf8_lossy(fresh);
    TextDiff::from_lines(&canonical, &fresh)
        .unified_diff()
        .header(&format!("a/{path}"), &format!("b/{path}"))
        .to_string()
}

fn replace_file(path: &Path, contents: &[u8]) -> Result<(), OutputError> {
    validate_file(path)?;
    let parent = artifact_parent(path);
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

fn replace_directory(
    path: &Path,
    files: &std::collections::BTreeMap<dbmd_render::ArtifactPath, Vec<u8>>,
) -> Result<(), OutputError> {
    validate_directory(path)?;
    let parent = artifact_parent(path);
    fs::create_dir_all(parent).map_err(|source| OutputError::CreateParent {
        path: parent.to_path_buf(),
        source,
    })?;
    let staging = TempDir::new_in(parent).map_err(|source| OutputError::Temporary {
        path: parent.to_path_buf(),
        source,
    })?;
    let payload = staging.path().join("artifact");
    fs::create_dir(&payload).map_err(|source| OutputError::Temporary {
        path: parent.to_path_buf(),
        source,
    })?;
    for (relative, contents) in files {
        let destination = payload.join(relative.as_str());
        let destination_parent = destination.parent().unwrap_or(&payload);
        fs::create_dir_all(destination_parent).map_err(|source| OutputError::Write {
            path: destination.clone(),
            source,
        })?;
        let mut file = fs::File::create(&destination).map_err(|source| OutputError::Write {
            path: destination.clone(),
            source,
        })?;
        file.write_all(contents)
            .and_then(|()| file.sync_all())
            .map_err(|source| OutputError::Write {
                path: destination,
                source,
            })?;
    }

    let previous = staging.path().join("previous");
    let existed = match fs::symlink_metadata(path) {
        Ok(_) => {
            fs::rename(path, &previous).map_err(|source| OutputError::Replace {
                path: path.to_path_buf(),
                source,
            })?;
            true
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(source) => {
            return Err(OutputError::Inspect {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    if let Err(source) = fs::rename(&payload, path) {
        if existed {
            if let Err(rollback) = fs::rename(&previous, path) {
                return Err(OutputError::Rollback {
                    path: path.to_path_buf(),
                    replace: source,
                    rollback,
                });
            }
        }
        return Err(OutputError::Replace {
            path: path.to_path_buf(),
            source,
        });
    }
    Ok(())
}

fn validate_file(path: &Path) -> Result<(), OutputError> {
    validate_common(path)?;
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

fn validate_directory(path: &Path) -> Result<(), OutputError> {
    validate_common(path)?;
    if path
        .components()
        .any(|component| component.as_os_str() == ".git")
    {
        return Err(OutputError::UnsafePath(path.to_path_buf()));
    }
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
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

fn validate_common(path: &Path) -> Result<(), OutputError> {
    if path.as_os_str().is_empty()
        || path.file_name().is_none()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(OutputError::UnsafePath(path.to_path_buf()));
    }
    Ok(())
}

fn artifact_parent(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

#[derive(Debug, Error)]
pub enum OutputError {
    #[error("unsafe artifact output path `{0}`")]
    UnsafePath(PathBuf),
    #[error("failed to inspect output path `{path}`")]
    Inspect {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to create output parent `{path}`")]
    CreateParent {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to create temporary output beside `{path}`")]
    Temporary {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write temporary output for `{path}`")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read canonical artifact `{path}`")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to atomically replace output `{path}`")]
    Replace {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to replace output `{path}` and restore the previous artifact")]
    Rollback {
        path: PathBuf,
        replace: io::Error,
        rollback: io::Error,
    },
}
