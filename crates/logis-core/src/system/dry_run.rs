//! A `System` that reads through another and changes nothing, ported from chezmoi's
//! `DryRunSystem` (`internal/chezmoi/dryrunsystem.go`).

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use super::port::{Metadata, System};

/// Reads go to the wrapped system; every write succeeds without doing anything. Later reads
/// therefore see the state before the run, as in chezmoi.
#[derive(Debug, Clone, Copy)]
pub struct DryRunSystem<'a> {
    inner: &'a dyn System,
}

impl<'a> DryRunSystem<'a> {
    #[must_use]
    pub fn new(inner: &'a dyn System) -> Self {
        Self { inner }
    }
}

impl System for DryRunSystem<'_> {
    fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.inner.read_file(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<OsString>> {
        self.inner.read_dir(path)
    }

    fn lstat(&self, path: &Path) -> io::Result<Metadata> {
        self.inner.lstat(path)
    }

    fn stat(&self, path: &Path) -> io::Result<Metadata> {
        self.inner.stat(path)
    }

    fn read_link(&self, path: &Path) -> io::Result<PathBuf> {
        self.inner.read_link(path)
    }

    fn mkdir(&self, _path: &Path, _perm: u32) -> io::Result<()> {
        Ok(())
    }

    fn chmod(&self, _path: &Path, _perm: u32) -> io::Result<()> {
        Ok(())
    }

    fn write_file(&self, _path: &Path, _contents: &[u8], _perm: u32) -> io::Result<()> {
        Ok(())
    }

    fn write_symlink(&self, _linkname: &Path, _path: &Path) -> io::Result<()> {
        Ok(())
    }

    fn remove_all(&self, _path: &Path) -> io::Result<()> {
        Ok(())
    }
}
