use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;
use thiserror::Error;

/// Inputs for creating the GitHub Actions verification workflow.
#[derive(Debug, Clone)]
pub struct InitCiRequest {
    workflow_path: PathBuf,
    overwrite: bool,
}

impl InitCiRequest {
    /// Creates a protected workflow initialization request.
    #[must_use]
    pub fn new(workflow_path: impl Into<PathBuf>) -> Self {
        Self {
            workflow_path: workflow_path.into(),
            overwrite: false,
        }
    }

    /// Explicitly permits replacement of an existing workflow.
    #[must_use]
    pub fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }
}

/// Observable result of successful CI initialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitCiReport {
    /// Created or explicitly replaced workflow path.
    pub workflow_path: PathBuf,
}

/// Creates a GitHub Actions workflow that installs a pinned dbmd and runs verify.
///
/// # Errors
///
/// Returns [`InitCiError`] when an existing workflow is protected or the new
/// workflow cannot be durably staged and installed.
pub fn init_ci(request: InitCiRequest) -> Result<InitCiReport, InitCiError> {
    if request.workflow_path.exists() && !request.overwrite {
        return Err(InitCiError::ExistingWorkflow(request.workflow_path));
    }
    let parent = request
        .workflow_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| InitCiError::CreateParent {
        path: parent.to_path_buf(),
        source,
    })?;
    let mut staged = NamedTempFile::new_in(parent).map_err(|source| InitCiError::Stage {
        path: parent.to_path_buf(),
        source,
    })?;
    let workflow = workflow_source(env!("CARGO_PKG_VERSION"));
    staged
        .write_all(workflow.as_bytes())
        .and_then(|()| staged.flush())
        .and_then(|()| staged.as_file().sync_all())
        .map_err(|source| InitCiError::Write {
            path: request.workflow_path.clone(),
            source,
        })?;
    let installed = if request.overwrite {
        staged.persist(&request.workflow_path)
    } else {
        staged.persist_noclobber(&request.workflow_path)
    };
    installed.map_err(|error| {
        if error.error.kind() == io::ErrorKind::AlreadyExists && !request.overwrite {
            InitCiError::ExistingWorkflow(request.workflow_path.clone())
        } else {
            InitCiError::Install {
                path: request.workflow_path.clone(),
                source: error.error,
            }
        }
    })?;
    Ok(InitCiReport {
        workflow_path: request.workflow_path,
    })
}

fn workflow_source(version: &str) -> String {
    format!(
        r#"name: dbmd

on:
  pull_request:
  push:
    branches: [main]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install dbmd
        run: cargo install dbmd --locked --version {version}
      - name: Verify database documentation
        run: dbmd verify
        env:
          DATABASE_URL: ${{{{ secrets.DATABASE_URL }}}}
          # Add any additional source variables using GitHub Actions secrets.
"#,
    )
}

/// Why GitHub Actions initialization failed.
#[derive(Debug, Error)]
pub enum InitCiError {
    #[error("CI workflow `{0}` already exists; explicit overwrite is required")]
    ExistingWorkflow(PathBuf),
    #[error("failed to create CI workflow parent `{path}`")]
    CreateParent {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to stage CI workflow beneath `{path}`")]
    Stage {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write staged CI workflow for `{path}`")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to install CI workflow `{path}`")]
    Install {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
