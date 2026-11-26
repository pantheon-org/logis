use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::target_state::TargetError;

/// Why a target could not be applied.
#[derive(Debug)]
pub enum ApplyError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    /// The destination holds something that is not a file, directory or symlink.
    UnsupportedFileType(PathBuf),
    /// Applying this kind of target is not supported yet.
    Unsupported(&'static str),
    Target(TargetError),
}

/// An error while applying `target`, the target path relative to the destination directory.
#[derive(Debug)]
pub struct TargetApplyError {
    pub target: String,
    pub error: ApplyError,
}

impl fmt::Display for ApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::UnsupportedFileType(path) => {
                write!(f, "{}: unsupported file type", path.display())
            }
            Self::Unsupported(what) => write!(f, "{what} are not supported yet"),
            Self::Target(err) => err.fmt(f),
        }
    }
}

impl fmt::Display for TargetApplyError {
    /// chezmoi's `applyArgs` prefixes the target path.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.target, self.error)
    }
}

impl std::error::Error for ApplyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Target(err) => Some(err),
            _ => None,
        }
    }
}

impl std::error::Error for TargetApplyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

impl From<TargetError> for ApplyError {
    fn from(err: TargetError) -> Self {
        Self::Target(err)
    }
}

/// Wrap an I/O error with the path it concerns.
pub(super) fn io_error(path: &std::path::Path) -> impl FnOnce(io::Error) -> ApplyError + '_ {
    move |source| ApplyError::Io {
        path: path.to_owned(),
        source,
    }
}
