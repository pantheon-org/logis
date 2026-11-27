//! A `System` that writes a git diff of every change, ported from chezmoi's `GitDiffSystem`
//! (`internal/chezmoi/gitdiffsystem.go`). Wrap a `DryRunSystem` and apply through it: each
//! write becomes a file patch from what is in the destination now.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use super::patch::{Side, file_patch};
use crate::entry_type::{EntryType, EntryTypeFilter};
use crate::system::{FileType, Metadata, System};

const DIR: u32 = 0o040_000;
const FILE: u32 = 0o100_000;
const SYMLINK: u32 = 0o120_000;

/// What a patch side holds before it is borrowed: contents and git mode.
type Owned = Option<(Vec<u8>, u32)>;

#[derive(Debug)]
pub struct GitDiffSystem<'a> {
    inner: &'a dyn System,
    dest_dir: &'a Path,
    filter: EntryTypeFilter,
    reverse: bool,
    /// Entries removed so far: later reads no longer see them or anything under them.
    removed: RefCell<BTreeSet<PathBuf>>,
    out: RefCell<Vec<u8>>,
}

impl<'a> GitDiffSystem<'a> {
    /// Patches name paths relative to `dest_dir`. Writes of types `filter` leaves out make
    /// no patch. `reverse` swaps the sides.
    #[must_use]
    pub fn new(
        inner: &'a dyn System,
        dest_dir: &'a Path,
        filter: EntryTypeFilter,
        reverse: bool,
    ) -> Self {
        Self {
            inner,
            dest_dir,
            filter,
            reverse,
            removed: RefCell::default(),
            out: RefCell::default(),
        }
    }

    /// The diff written so far.
    #[must_use]
    pub fn into_output(self) -> Vec<u8> {
        self.out.into_inner()
    }

    fn is_removed(&self, path: &Path) -> bool {
        let removed = self.removed.borrow();
        path.ancestors().any(|p| removed.contains(p))
    }

    fn hidden(&self, path: &Path) -> io::Result<()> {
        if self.is_removed(path) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{}: removed", path.display()),
            ));
        }
        Ok(())
    }

    /// Write the patch from what is at `path` now to `to`.
    fn encode(&self, path: &Path, to: Owned) -> io::Result<()> {
        let mut from = self.current(path)?;
        let mut to = to;
        if self.reverse {
            std::mem::swap(&mut from, &mut to);
        }
        let rel = path.strip_prefix(self.dest_dir).unwrap_or(path);
        let rel: Vec<_> = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect();
        let encoded = file_patch(&rel.join("/"), side(&from), side(&to));
        self.out.borrow_mut().extend(encoded);
        Ok(())
    }

    /// The contents and git mode at `path`, from the wrapped system: a file's contents, a
    /// symlink's target and a newline, nothing for a directory.
    fn current(&self, path: &Path) -> io::Result<Owned> {
        let meta = match self.inner.lstat(path) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
            other => other?,
        };
        let mode = git_mode(meta)?;
        let data = match meta.file_type {
            FileType::File => self.inner.read_file(path)?,
            FileType::Symlink => link_data(&self.inner.read_link(path)?),
            FileType::Dir | FileType::Other => Vec::new(),
        };
        Ok(Some((data, mode)))
    }
}

fn side(owned: &Owned) -> Option<Side<'_>> {
    owned
        .as_ref()
        .map(|(data, mode)| Side { data, mode: *mode })
}

/// A symlink's patch contents: its target and a newline.
fn link_data(linkname: &Path) -> Vec<u8> {
    let mut data = linkname.as_os_str().as_encoded_bytes().to_vec();
    data.push(b'\n');
    data
}

/// go-git's file mode for `meta`: the type bits and the permission bits.
fn git_mode(meta: Metadata) -> io::Result<u32> {
    let kind = match meta.file_type {
        FileType::File => FILE,
        FileType::Dir => DIR,
        FileType::Symlink => SYMLINK,
        FileType::Other => {
            return Err(io::Error::other("unsupported file type for a diff"));
        }
    };
    Ok(kind | (meta.mode & 0o777))
}

fn entry_type(file_type: FileType) -> Option<EntryType> {
    match file_type {
        FileType::File => Some(EntryType::Files),
        FileType::Dir => Some(EntryType::Dirs),
        FileType::Symlink => Some(EntryType::Symlinks),
        FileType::Other => None,
    }
}

impl System for GitDiffSystem<'_> {
    fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.inner.read_file(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<OsString>> {
        self.hidden(path)?;
        let names = self.inner.read_dir(path)?;
        Ok(names
            .into_iter()
            .filter(|name| !self.is_removed(&path.join(name)))
            .collect())
    }

    fn lstat(&self, path: &Path) -> io::Result<Metadata> {
        self.hidden(path)?;
        self.inner.lstat(path)
    }

    fn stat(&self, path: &Path) -> io::Result<Metadata> {
        self.hidden(path)?;
        self.inner.stat(path)
    }

    fn read_link(&self, path: &Path) -> io::Result<PathBuf> {
        self.inner.read_link(path)
    }

    fn mkdir(&self, path: &Path, perm: u32) -> io::Result<()> {
        if self.filter.includes_type(EntryType::Dirs) {
            self.encode(path, Some((Vec::new(), DIR | perm)))?;
        }
        self.inner.mkdir(path, perm)
    }

    /// The patch keeps the contents and changes the mode.
    fn chmod(&self, path: &Path, perm: u32) -> io::Result<()> {
        let meta = self.inner.stat(path)?;
        if entry_type(meta.file_type).is_some_and(|t| self.filter.includes_type(t)) {
            let data = match meta.file_type {
                FileType::File => self.inner.read_file(path)?,
                _ => Vec::new(),
            };
            let kind = git_mode(meta)? & !0o777;
            self.encode(path, Some((data, kind | perm)))?;
        }
        self.inner.chmod(path, perm)
    }

    fn write_file(&self, path: &Path, contents: &[u8], perm: u32) -> io::Result<()> {
        if self.filter.includes_type(EntryType::Files) {
            self.encode(path, Some((contents.to_vec(), FILE | perm)))?;
        }
        self.inner.write_file(path, contents, perm)
    }

    fn write_symlink(&self, linkname: &Path, path: &Path) -> io::Result<()> {
        if self.filter.includes_type(EntryType::Symlinks) {
            self.encode(path, Some((link_data(linkname), SYMLINK)))?;
        }
        self.inner.write_symlink(linkname, path)
    }

    fn remove_all(&self, path: &Path) -> io::Result<()> {
        if self.filter.includes_type(EntryType::Remove) {
            self.encode(path, None)?;
        }
        self.inner.remove_all(path)?;
        self.removed.borrow_mut().insert(path.to_owned());
        Ok(())
    }
}
