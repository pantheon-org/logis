use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

/// The type of a filesystem entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Dir,
    Symlink,
    /// Anything else: a device, socket or FIFO.
    Other,
}

/// What logis needs to know about an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Metadata {
    pub file_type: FileType,
    /// Permission bits (`0o7777` at most). Zero where the system has none.
    pub mode: u32,
}

/// Filesystem operations logis needs. Grows test-first, one method per failing scenario.
///
/// `tests/system_contract.rs` runs the same checks against every implementation.
pub trait System: std::fmt::Debug {
    /// Read a file's full contents.
    fn read_file(&self, path: &Path) -> io::Result<Vec<u8>>;

    /// The names of a directory's entries, sorted, without `.` and `..`.
    fn read_dir(&self, path: &Path) -> io::Result<Vec<OsString>>;

    /// Metadata of `path` itself; a symlink is reported as a symlink.
    fn lstat(&self, path: &Path) -> io::Result<Metadata>;

    /// Metadata of what `path` points to, following symlinks.
    fn stat(&self, path: &Path) -> io::Result<Metadata>;

    /// The target of the symlink at `path`.
    fn read_link(&self, path: &Path) -> io::Result<PathBuf>;
}
