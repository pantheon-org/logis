//! The `unmanaged` query: entries in the destination that the source state neither manages
//! nor ignores. Ported from chezmoi's `runUnmanagedCmd` (`internal/cmd/unmanagedcmd.go`).
//!
//! Each root is walked without following symlinks. An entry that is not managed is listed (if
//! the filter includes its type) and, if it is a directory, not entered; so is an ignored one,
//! which is never listed. Not yet: externals, whose git-repo directories chezmoi does not
//! enter.

use std::collections::BTreeSet;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::entry_type::{EntryType, EntryTypeFilter};
use crate::source_state::SourceState;
use crate::system::{FileType, Metadata, System};

/// Why the walk stopped.
#[derive(Debug)]
pub enum UnmanagedError {
    /// A root outside the destination directory.
    NotInDestination { path: PathBuf, dest_dir: PathBuf },
    /// A root that could not be read.
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for UnmanagedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInDestination { path, dest_dir } => {
                write!(f, "{}: not in {}", path.display(), dest_dir.display())
            }
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl std::error::Error for UnmanagedError {}

/// The unmanaged entries under `roots` (absolute paths), as sorted target paths, once each.
/// A root that is missing or outside `dest_dir` stops the walk. Entries below a root that
/// cannot be read are reported to `warn` and skipped, as chezmoi does.
pub fn unmanaged(
    system: &dyn System,
    state: &SourceState,
    dest_dir: &Path,
    roots: &[PathBuf],
    filter: EntryTypeFilter,
    warn: &mut dyn FnMut(String),
) -> Result<Vec<String>, UnmanagedError> {
    let roots: BTreeSet<&PathBuf> = roots.iter().collect();
    let mut walker = Walker {
        system,
        state,
        filter,
        warn,
        found: BTreeSet::new(),
    };
    for root in roots {
        let rel = relative(dest_dir, root)?;
        let meta = system.lstat(root).map_err(|source| UnmanagedError::Io {
            path: root.clone(),
            source,
        })?;
        walker.visit(root, &rel, meta);
    }
    Ok(walker.found.into_iter().collect())
}

/// `path` relative to `dest_dir`, `/`-separated, empty for `dest_dir` itself.
fn relative(dest_dir: &Path, path: &Path) -> Result<String, UnmanagedError> {
    let rel = path
        .strip_prefix(dest_dir)
        .map_err(|_| UnmanagedError::NotInDestination {
            path: path.to_owned(),
            dest_dir: dest_dir.to_owned(),
        })?;
    Ok(rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

struct Walker<'a> {
    system: &'a dyn System,
    state: &'a SourceState,
    filter: EntryTypeFilter,
    warn: &'a mut dyn FnMut(String),
    found: BTreeSet<String>,
}

impl Walker<'_> {
    /// Visit the entry at `path`, whose target path is `rel` (empty for the destination
    /// directory, which is entered but never listed).
    fn visit(&mut self, path: &Path, rel: &str, meta: Metadata) {
        let enter = if rel.is_empty() {
            true
        } else {
            let managed = self.state.get(rel).is_some();
            let ignored = self.state.is_ignored(rel);
            if !managed && !ignored && self.includes(meta.file_type) {
                self.found.insert(rel.to_owned());
            }
            managed && !ignored
        };
        if enter && meta.file_type == FileType::Dir {
            self.enter(path, rel);
        }
    }

    fn enter(&mut self, dir: &Path, rel: &str) {
        let names = match self.system.read_dir(dir) {
            Ok(names) => names,
            Err(err) => return (self.warn)(format!("{}: {err}", dir.display())),
        };
        for name in names {
            let path = dir.join(&name);
            let name = name.to_string_lossy();
            let child = if rel.is_empty() {
                name.into_owned()
            } else {
                format!("{rel}/{name}")
            };
            match self.system.lstat(&path) {
                Ok(meta) => self.visit(&path, &child, meta),
                Err(err) => (self.warn)(format!("{}: {err}", path.display())),
            }
        }
    }

    /// chezmoi's `EntryTypeFilter.IncludeFileInfo`.
    fn includes(&self, file_type: FileType) -> bool {
        let entry_type = match file_type {
            FileType::Dir => EntryType::Dirs,
            FileType::File => EntryType::Files,
            FileType::Symlink => EntryType::Symlinks,
            FileType::Other => return false,
        };
        self.filter.includes_type(entry_type)
    }
}
