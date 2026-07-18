use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use tempfile::TempDir;
use thiserror::Error;

/// Inputs for copying the complete embedded template profile into a project.
#[derive(Debug, Clone)]
pub struct InitTemplatesRequest {
    template_root: PathBuf,
}

impl InitTemplatesRequest {
    /// Creates a request for a project-owned template root.
    #[must_use]
    pub fn new(template_root: impl Into<PathBuf>) -> Self {
        Self {
            template_root: template_root.into(),
        }
    }
}

/// Observable result of successful template initialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitTemplatesReport {
    /// Newly created root containing profile directories.
    pub template_root: PathBuf,
    /// Created files, relative to [`Self::template_root`].
    pub files: Vec<PathBuf>,
}

/// Atomically creates a complete editable copy of the embedded `agent` profile.
///
/// # Errors
///
/// Returns [`InitTemplatesError`] when the destination already exists or the
/// complete template tree cannot be staged and installed.
pub fn init_templates(
    request: InitTemplatesRequest,
) -> Result<InitTemplatesReport, InitTemplatesError> {
    if request.template_root.exists() {
        return Err(InitTemplatesError::ExistingRoot(request.template_root));
    }
    let parent = request
        .template_root
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| InitTemplatesError::CreateParent {
        path: parent.to_path_buf(),
        source,
    })?;
    let staging = TempDir::new_in(parent).map_err(|source| InitTemplatesError::Stage {
        path: parent.to_path_buf(),
        source,
    })?;
    let payload = staging.path().join("templates");
    let mut files = Vec::new();
    for template in dbmd_render::embedded_template_files() {
        let relative = Path::new("agent").join(template.relative_path);
        let destination = payload.join(&relative);
        let directory = destination
            .parent()
            .expect("embedded template paths always have a parent");
        fs::create_dir_all(directory).map_err(|source| InitTemplatesError::Stage {
            path: directory.to_path_buf(),
            source,
        })?;
        let mut file =
            fs::File::create(&destination).map_err(|source| InitTemplatesError::Write {
                path: destination.clone(),
                source,
            })?;
        file.write_all(template.contents.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|source| InitTemplatesError::Write {
                path: destination,
                source,
            })?;
        files.push(relative);
    }
    for template in dbmd_backends::all_template_files() {
        let relative = Path::new("agent").join(template.relative_path);
        let destination = payload.join(&relative);
        let directory = destination
            .parent()
            .expect("backend template paths always have a parent");
        fs::create_dir_all(directory).map_err(|source| InitTemplatesError::Stage {
            path: directory.to_path_buf(),
            source,
        })?;
        let mut file =
            fs::File::create(&destination).map_err(|source| InitTemplatesError::Write {
                path: destination.clone(),
                source,
            })?;
        file.write_all(template.contents.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|source| InitTemplatesError::Write {
                path: destination,
                source,
            })?;
        files.push(relative);
    }
    fs::rename(&payload, &request.template_root).map_err(|source| {
        if source.kind() == io::ErrorKind::AlreadyExists {
            InitTemplatesError::ExistingRoot(request.template_root.clone())
        } else {
            InitTemplatesError::Install {
                path: request.template_root.clone(),
                source,
            }
        }
    })?;
    Ok(InitTemplatesReport {
        template_root: request.template_root,
        files,
    })
}

/// Why a project-owned template tree could not be initialized.
#[derive(Debug, Error)]
pub enum InitTemplatesError {
    #[error("template root `{0}` already exists")]
    ExistingRoot(PathBuf),
    #[error("failed to create template parent `{path}`")]
    CreateParent {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to stage templates beneath `{path}`")]
    Stage {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write template `{path}`")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to install template root `{path}`")]
    Install {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
