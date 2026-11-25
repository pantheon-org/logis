//! Walking the source directory into candidate entries: one per source entry, before
//! duplicates are checked.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::entry::SourceEntry;
use super::error::ReadError;
use crate::source_name::{DirAttr, FileAttr};
use crate::system::{FileType, Metadata, System};

/// Names starting with this are not source entries (chezmoi's `ignorePrefix`). The tool's
/// special files also start with it; they are not supported yet and are skipped too.
const IGNORE_PREFIX: char = '.';

/// The suffix removed from encrypted files' names. Encryption is not supported yet.
const ENCRYPTED_SUFFIX: &str = "";

/// Every source entry under `root`, with its target path, in walk order.
pub(super) fn walk(
    system: &dyn System,
    root: &Path,
) -> Result<Vec<(String, SourceEntry)>, ReadError> {
    let mut walker = Walker {
        system,
        found: Vec::new(),
    };
    walker.walk_dir(root, "", "")?;
    Ok(walker.found)
}

struct Walker<'a> {
    system: &'a dyn System,
    found: Vec<(String, SourceEntry)>,
}

impl Walker<'_> {
    /// Walk `dir`, whose path relative to the source directory is `source_rel` and whose
    /// target path is `target` (both empty for the source directory itself).
    fn walk_dir(&mut self, dir: &Path, source_rel: &str, target: &str) -> Result<(), ReadError> {
        for name in self.system.read_dir(dir).map_err(io_error(dir))? {
            let path = dir.join(&name);
            let name = utf8(&name, &path)?;
            if name.starts_with(IGNORE_PREFIX) {
                continue;
            }
            let entry = Entry {
                source_rel: join(source_rel, name),
                path,
                name,
            };
            self.visit(&entry, target)?;
        }
        Ok(())
    }

    fn visit(&mut self, entry: &Entry<'_>, parent_target: &str) -> Result<(), ReadError> {
        match self.followed_metadata(&entry.path)?.file_type {
            FileType::Dir => self.visit_dir(entry, parent_target),
            FileType::File => self.visit_file(entry, parent_target),
            FileType::Symlink | FileType::Other => {
                Err(ReadError::UnsupportedFileType(entry.path.clone()))
            }
        }
    }

    fn visit_dir(&mut self, entry: &Entry<'_>, parent_target: &str) -> Result<(), ReadError> {
        let attr = DirAttr::parse(entry.name).map_err(ReadError::InvalidName)?;
        let target = join(parent_target, &attr.target_name);
        self.found.push((
            target.clone(),
            SourceEntry::Dir {
                source_path: entry.path.clone(),
                source_rel: entry.source_rel.clone(),
                attr,
            },
        ));
        self.walk_dir(&entry.path, &entry.source_rel, &target)
    }

    fn visit_file(&mut self, entry: &Entry<'_>, parent_target: &str) -> Result<(), ReadError> {
        let attr = FileAttr::parse(entry.name, ENCRYPTED_SUFFIX).map_err(ReadError::InvalidName)?;
        self.found.push((
            join(parent_target, &attr.target_name),
            SourceEntry::File {
                source_path: entry.path.clone(),
                source_rel: entry.source_rel.clone(),
                attr,
            },
        ));
        Ok(())
    }

    /// Metadata of `path`, following a symlink: symlinks in the source directory stand for
    /// what they point to.
    fn followed_metadata(&self, path: &Path) -> Result<Metadata, ReadError> {
        let meta = self.system.lstat(path).map_err(io_error(path))?;
        if meta.file_type == FileType::Symlink {
            return self.system.stat(path).map_err(io_error(path));
        }
        Ok(meta)
    }
}

/// A source entry being visited.
struct Entry<'a> {
    path: PathBuf,
    name: &'a str,
    source_rel: String,
}

fn join(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_owned()
    } else {
        format!("{parent}/{name}")
    }
}

fn utf8<'a>(name: &'a OsStr, path: &Path) -> Result<&'a str, ReadError> {
    name.to_str()
        .ok_or_else(|| ReadError::NonUtf8Name(path.to_owned()))
}

fn io_error(path: &Path) -> impl FnOnce(std::io::Error) -> ReadError + '_ {
    move |source| ReadError::Io {
        path: path.to_owned(),
        source,
    }
}
