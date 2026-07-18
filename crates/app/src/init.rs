use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use serde::Serialize;
use thiserror::Error;

/// Inputs for project initialization.
#[derive(Debug, Clone)]
pub struct InitRequest {
    config_path: PathBuf,
}

impl InitRequest {
    /// Creates an initialization request for one config path.
    #[must_use]
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
        }
    }
}

/// Observable result of successful project initialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitReport {
    /// Newly created configuration path.
    pub config_path: PathBuf,
    /// Relative SQLite path selected by conservative discovery.
    pub detected_database: Option<PathBuf>,
}

/// Creates a safe-to-commit project configuration without replacing an existing file.
///
/// # Errors
///
/// Returns [`InitError`] when the destination exists, its parent cannot be
/// created, discovery cannot inspect the project, serialization fails, or the
/// new file cannot be durably written.
pub fn init(request: InitRequest) -> Result<InitReport, InitError> {
    if request.config_path.exists() {
        return Err(InitError::ExistingConfig(request.config_path));
    }
    let parent = request
        .config_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| InitError::CreateParent {
        path: parent.to_path_buf(),
        source,
    })?;
    let detected_database = discover_sqlite(parent)?;
    let database_path = detected_database
        .as_deref()
        .unwrap_or_else(|| Path::new("dev.db"));
    let config = generated_config(&database_path.to_string_lossy())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&request.config_path)
        .map_err(|source| {
            if source.kind() == io::ErrorKind::AlreadyExists {
                InitError::ExistingConfig(request.config_path.clone())
            } else {
                InitError::Write {
                    path: request.config_path.clone(),
                    source,
                }
            }
        })?;
    file.write_all(config.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|source| InitError::Write {
            path: request.config_path.clone(),
            source,
        })?;
    Ok(InitReport {
        config_path: request.config_path,
        detected_database,
    })
}

fn discover_sqlite(directory: &Path) -> Result<Option<PathBuf>, InitError> {
    let mut candidates = fs::read_dir(directory)
        .map_err(|source| InitError::ReadDirectory {
            path: directory.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| InitError::ReadDirectory {
            path: directory.to_path_buf(),
            source,
        })?;
    candidates.sort_by_key(std::fs::DirEntry::file_name);
    let mut sqlite = Vec::new();
    for candidate in candidates {
        let path = candidate.path();
        if !candidate
            .file_type()
            .map_err(|source| InitError::InspectCandidate {
                path: path.clone(),
                source,
            })?
            .is_file()
            || !has_sqlite_extension(&path)
        {
            continue;
        }
        if has_sqlite_header(&path)? {
            sqlite.push(PathBuf::from(candidate.file_name()));
        }
    }
    Ok((sqlite.len() == 1).then(|| sqlite.remove(0)))
}

fn has_sqlite_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "db" | "sqlite" | "sqlite3"
            )
        })
}

fn has_sqlite_header(path: &Path) -> Result<bool, InitError> {
    let mut file = File::open(path).map_err(|source| InitError::InspectCandidate {
        path: path.to_path_buf(),
        source,
    })?;
    let mut header = [0_u8; 16];
    match file.read_exact(&mut header) {
        Ok(()) => Ok(&header == b"SQLite format 3\0"),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Ok(false),
        Err(source) => Err(InitError::InspectCandidate {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn generated_config(database_path: &str) -> Result<String, InitError> {
    #[derive(Serialize)]
    struct Config<'a> {
        sources: Sources<'a>,
        output: Output,
    }
    #[derive(Serialize)]
    struct Sources<'a> {
        local: Source<'a>,
    }
    #[derive(Serialize)]
    struct Source<'a> {
        backend: &'static str,
        path: &'a str,
    }
    #[derive(Serialize)]
    struct Output {
        path: &'static str,
        profile: &'static str,
        sources: [&'static str; 1],
        layout: Layout,
    }
    #[derive(Serialize)]
    struct Layout {
        kind: &'static str,
        source_layout: &'static str,
    }

    let config = Config {
        sources: Sources {
            local: Source {
                backend: "sqlite",
                path: database_path,
            },
        },
        output: Output {
            path: "DATABASE.md",
            profile: "agent",
            sources: ["local"],
            layout: Layout {
                kind: "single_file",
                source_layout: "auto",
            },
        },
    };
    let mut rendered = toml::to_string_pretty(&config).map_err(InitError::Serialize)?;
    rendered.push('\n');
    Ok(rendered)
}

/// Why project initialization failed.
#[derive(Debug, Error)]
pub enum InitError {
    #[error("configuration `{0}` already exists")]
    ExistingConfig(PathBuf),
    #[error("failed to create configuration parent `{path}`")]
    CreateParent {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to inspect project directory `{path}`")]
    ReadDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to inspect SQLite candidate `{path}`")]
    InspectCandidate {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to serialize generated configuration")]
    Serialize(#[source] toml::ser::Error),
    #[error("failed to write new configuration `{path}`")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
