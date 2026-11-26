use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use super::port::{FileType, Metadata, System};

/// The process umask, as chezmoi reads it at start-up (`internal/chezmoi/chezmoi_unix.go`).
///
/// Reading the umask means setting it: it is set to 0 and straight back. A file another thread
/// creates in between would get the wrong mode, so call this once, before starting threads,
/// and pass the value on. It is 0 where there is no umask.
#[must_use]
pub fn process_umask() -> u32 {
    #[cfg(unix)]
    {
        use rustix::fs::Mode;
        use rustix::process::umask;
        let current = umask(Mode::empty());
        umask(current);
        u32::from(current.bits())
    }
    #[cfg(not(unix))]
    {
        0
    }
}

/// The real machine. The only place allowed to use `std::fs` directly.
#[derive(Debug, Default, Clone, Copy)]
pub struct RealSystem;

impl System for RealSystem {
    fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<OsString>> {
        let mut names = std::fs::read_dir(path)?
            .map(|entry| entry.map(|e| e.file_name()))
            .collect::<io::Result<Vec<_>>>()?;
        names.sort();
        Ok(names)
    }

    fn lstat(&self, path: &Path) -> io::Result<Metadata> {
        std::fs::symlink_metadata(path).map(|m| metadata(&m))
    }

    fn stat(&self, path: &Path) -> io::Result<Metadata> {
        std::fs::metadata(path).map(|m| metadata(&m))
    }

    fn read_link(&self, path: &Path) -> io::Result<PathBuf> {
        std::fs::read_link(path)
    }
}

fn metadata(meta: &std::fs::Metadata) -> Metadata {
    let file_type = meta.file_type();
    let file_type = if file_type.is_symlink() {
        FileType::Symlink
    } else if file_type.is_dir() {
        FileType::Dir
    } else if file_type.is_file() {
        FileType::File
    } else {
        FileType::Other
    };
    Metadata {
        file_type,
        mode: mode(meta),
    }
}

#[cfg(unix)]
fn mode(meta: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o7777
}

#[cfg(not(unix))]
fn mode(_meta: &std::fs::Metadata) -> u32 {
    0
}
