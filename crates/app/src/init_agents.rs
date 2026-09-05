use std::{fs, io::Write, path::Path, path::PathBuf};

use tempfile::NamedTempFile;
use thiserror::Error;

use crate::config;

const BEGIN_MARKER: &str = "<!-- dbmd:begin -->";
const END_MARKER: &str = "<!-- dbmd:end -->";

/// Inputs for generating or safely installing agent-facing database guidance.
#[derive(Clone)]
pub struct InitAgentsRequest {
    config_path: PathBuf,
    file: Option<PathBuf>,
}

impl std::fmt::Debug for InitAgentsRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InitAgentsRequest")
            .field("config_path", &self.config_path)
            .field("file", &self.file)
            .finish()
    }
}

impl InitAgentsRequest {
    /// Creates a stdout-only request.
    #[must_use]
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
            file: None,
        }
    }

    /// Explicitly selects an instruction file to create or update in place.
    #[must_use]
    pub fn with_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = Some(file.into());
        self
    }
}

/// Observable result of agent-instruction initialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitAgentsReport {
    /// Config-relative canonical artifact with environment references preserved.
    pub canonical_artifact: PathBuf,
    /// Complete marked snippet suitable for stdout or installation.
    pub instructions: String,
    /// Explicitly updated instruction file, when requested.
    pub written_path: Option<PathBuf>,
    /// Whether file contents changed.
    pub changed: bool,
}

/// Generates agent guidance and optionally installs one isolated marked block.
///
/// Source connection environment is not required because this operation only
/// parses the canonical output field. Existing unrelated instructions are
/// preserved byte-for-byte.
///
/// # Errors
///
/// Returns [`InitAgentsError`] when configuration cannot be read or resolved,
/// an existing marked block is malformed, or an explicit file update fails.
pub fn init_agents(request: InitAgentsRequest) -> Result<InitAgentsReport, InitAgentsError> {
    let contents =
        fs::read_to_string(&request.config_path).map_err(|source| InitAgentsError::ReadConfig {
            path: request.config_path.clone(),
            source,
        })?;
    let canonical_artifact =
        config::resolve_canonical_output_display(&contents, &request.config_path)?;
    let config_root = request
        .config_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let displayed_artifact = canonical_artifact
        .strip_prefix(config_root)
        .unwrap_or(&canonical_artifact);
    let instructions = instruction_block(displayed_artifact);

    let Some(path) = request.file else {
        return Ok(InitAgentsReport {
            canonical_artifact,
            instructions,
            written_path: None,
            changed: false,
        });
    };
    let (previous, permissions) = match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err(InitAgentsError::UnsafeFile(path));
        }
        Ok(metadata) => (
            fs::read_to_string(&path).map_err(|source| InitAgentsError::ReadFile {
                path: path.clone(),
                source,
            })?,
            Some(metadata.permissions()),
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (String::new(), None),
        Err(source) => {
            return Err(InitAgentsError::InspectFile {
                path: path.clone(),
                source,
            });
        }
    };
    let next = merge_instructions(&previous, &instructions)
        .map_err(|()| InitAgentsError::MalformedBlock(path.clone()))?;
    let changed = next != previous;
    if changed {
        atomic_write(&path, next.as_bytes(), permissions)?;
    }
    Ok(InitAgentsReport {
        canonical_artifact,
        instructions,
        written_path: Some(path),
        changed,
    })
}

fn instruction_block(artifact: &Path) -> String {
    let artifact = artifact
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/");
    let artifact = markdown_code_span(&artifact);
    format!(
        "{BEGIN_MARKER}\n## Database documentation\n\nRead the generated artifact at {artifact} before reconstructing database schema state from migrations. Prefer that artifact for structural questions when `dbmd verify` is expected to be current. When freshness is uncertain, run or request `dbmd verify`. If verification fails, run `dbmd render` and review the generated changes. Do not edit the generated artifact manually. Query a live database only when the artifact cannot answer an operational or data question.\n{END_MARKER}\n"
    )
}

fn markdown_code_span(value: &str) -> String {
    let longest = value
        .split(|character| character != '`')
        .map(str::len)
        .max()
        .unwrap_or_default();
    let fence = "`".repeat(longest + 1);
    format!("{fence} {value} {fence}")
}

fn merge_instructions(previous: &str, block: &str) -> Result<String, ()> {
    let begins = standalone_markers(previous, BEGIN_MARKER)?;
    let ends = standalone_markers(previous, END_MARKER)?;
    match (begins.as_slice(), ends.as_slice()) {
        ([], []) => {
            let mut merged = previous.to_string();
            if !merged.is_empty() && !merged.ends_with('\n') {
                merged.push('\n');
            }
            if !merged.is_empty() && !merged.ends_with("\n\n") {
                merged.push('\n');
            }
            merged.push_str(block);
            Ok(merged)
        }
        ([(begin, _)], [(end, after)]) if begin < end => {
            let mut merged = String::with_capacity(previous.len() + block.len());
            merged.push_str(&previous[..*begin]);
            merged.push_str(block);
            merged.push_str(&previous[*after..]);
            Ok(merged)
        }
        _ => Err(()),
    }
}

fn standalone_markers(contents: &str, marker: &str) -> Result<Vec<(usize, usize)>, ()> {
    contents
        .match_indices(marker)
        .map(|(offset, _)| {
            let line_start = contents[..offset].rfind('\n').map_or(0, |index| index + 1);
            let line_end = contents[offset..]
                .find('\n')
                .map_or(contents.len(), |index| offset + index);
            let line = contents[line_start..line_end]
                .strip_suffix('\r')
                .unwrap_or(&contents[line_start..line_end]);
            if line == marker {
                Ok((
                    line_start,
                    line_end + usize::from(line_end < contents.len()),
                ))
            } else {
                Err(())
            }
        })
        .collect()
}

fn atomic_write(
    path: &Path,
    contents: &[u8],
    permissions: Option<fs::Permissions>,
) -> Result<(), InitAgentsError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| InitAgentsError::CreateParent {
        path: parent.to_path_buf(),
        source,
    })?;
    let mut staged = NamedTempFile::new_in(parent).map_err(|source| InitAgentsError::Stage {
        path: parent.to_path_buf(),
        source,
    })?;
    staged
        .write_all(contents)
        .and_then(|()| staged.flush())
        .and_then(|()| staged.as_file().sync_all())
        .map_err(|source| InitAgentsError::Write {
            path: path.to_path_buf(),
            source,
        })?;
    if let Some(permissions) = permissions {
        staged
            .as_file()
            .set_permissions(permissions)
            .map_err(|source| InitAgentsError::Write {
                path: path.to_path_buf(),
                source,
            })?;
    }
    staged
        .persist(path)
        .map_err(|error| InitAgentsError::Install {
            path: path.to_path_buf(),
            source: error.error,
        })?;
    Ok(())
}

/// Why agent-instruction initialization failed.
#[derive(Debug, Error)]
pub enum InitAgentsError {
    #[error("failed to read configuration `{path}`")]
    ReadConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error("instruction file `{0}` must be a regular non-symlink file")]
    UnsafeFile(PathBuf),
    #[error("failed to inspect instruction file `{path}`")]
    InspectFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read instruction file `{path}` as UTF-8 text")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("instruction file `{0}` contains a malformed dbmd marker block")]
    MalformedBlock(PathBuf),
    #[error("failed to create instruction parent `{path}`")]
    CreateParent {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to stage instruction update beneath `{path}`")]
    Stage {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write instruction update for `{path}`")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to install instruction update `{path}`")]
    Install {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
