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

    fn mkdir(&self, path: &Path, perm: u32) -> io::Result<()> {
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, perm);
        #[cfg(not(unix))]
        let _ = perm;
        builder.create(path)
    }

    fn chmod(&self, path: &Path, perm: u32) -> io::Result<()> {
        set_perm(path, perm)
    }

    /// Like chezmoi's safe mode (`renameio`): a temporary file in the same directory, given
    /// its mode before its contents, then renamed over `path`.
    fn write_file(&self, path: &Path, contents: &[u8], perm: u32) -> io::Result<()> {
        use std::io::Write;
        let mut file = temp_builder().tempfile_in(parent(path))?;
        set_perm(file.path(), perm)?;
        file.write_all(contents)?;
        file.persist(path).map(drop).map_err(|e| e.error)
    }

    fn write_symlink(&self, linkname: &Path, path: &Path) -> io::Result<()> {
        let link = temp_builder().make_in(parent(path), |temp| symlink(linkname, temp))?;
        link.persist(path).map(drop).map_err(|e| e.error)
    }

    fn remove_all(&self, path: &Path) -> io::Result<()> {
        match std::fs::symlink_metadata(path) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
            Ok(meta) if meta.is_dir() => std::fs::remove_dir_all(path),
            Ok(_) => std::fs::remove_file(path),
        }
    }
}

fn temp_builder() -> tempfile::Builder<'static, 'static> {
    let mut builder = tempfile::Builder::new();
    builder.prefix(".logis-tmp-");
    builder
}

/// The directory `path` is in; the current directory for a bare name.
fn parent(path: &Path) -> &Path {
    match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir,
        _ => Path::new("."),
    }
}

#[cfg(unix)]
fn set_perm(path: &Path, perm: u32) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(perm))
}

#[cfg(not(unix))]
fn set_perm(_path: &Path, _perm: u32) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn symlink(linkname: &Path, path: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(linkname, path)
}

#[cfg(not(unix))]
fn symlink(_linkname: &Path, _path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "symlinks are not supported on this platform yet",
    ))
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
