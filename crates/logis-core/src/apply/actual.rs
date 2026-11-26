//! What is in the destination now, ported from chezmoi's `NewActualStateEntry`
//! (`internal/chezmoi/actualstateentry.go`). Contents and link targets are read only when a
//! comparison needs them.

use std::io;
use std::path::Path;

use super::error::{ApplyError, io_error};
use crate::system::{FileType, System};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Actual {
    Absent,
    Dir { perm: u32 },
    File { perm: u32 },
    Symlink,
}

impl Actual {
    /// The entry at `path`, not following a symlink.
    pub fn read(system: &dyn System, path: &Path) -> Result<Self, ApplyError> {
        let meta = match system.lstat(path) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Self::Absent),
            other => other.map_err(io_error(path))?,
        };
        // chezmoi compares `fs.FileMode.Perm()`: setuid, setgid and sticky bits do not count.
        let perm = meta.mode & 0o777;
        match meta.file_type {
            FileType::Dir => Ok(Self::Dir { perm }),
            FileType::File => Ok(Self::File { perm }),
            FileType::Symlink => Ok(Self::Symlink),
            FileType::Other => Err(ApplyError::UnsupportedFileType(path.to_owned())),
        }
    }

    /// Remove the entry, unless there is none (chezmoi's `ActualStateEntry.Remove`).
    pub fn remove(self, system: &dyn System, path: &Path) -> Result<(), ApplyError> {
        match self {
            Self::Absent => Ok(()),
            _ => system.remove_all(path).map_err(io_error(path)),
        }
    }
}
