//! Walking the source directory into candidate entries: one per source entry, before ignored
//! targets are dropped and duplicates are checked.
//!
//! Entries are visited in name order, as chezmoi's `WalkSourceDir` does. The ignore file is
//! read when the walk reaches it, so entries that sort before it are checked against fewer
//! patterns; `SourceState::read` checks every candidate again once the walk is done.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::entry::SourceEntry;
use super::error::ReadError;
use super::patterns::add_patterns;
use super::scripts;
use super::walk_util::{followed_metadata, io_error, join, utf8};
use crate::pattern_set::{PatternMatch, PatternSet};
use crate::source_name::{DirAttr, FileAttr};
use crate::special::{IGNORE, PREFIX, SCRIPTS_DIR};
use crate::system::{FileType, System};

/// Names starting with this are not source entries (chezmoi's `ignorePrefix`), unless they
/// are special files.
const IGNORE_PREFIX: char = '.';

/// The suffix removed from encrypted files' names. Encryption is not supported yet.
const ENCRYPTED_SUFFIX: &str = "";

/// What the walk found.
pub(super) struct Walk {
    /// Every source entry with its target path, in walk order.
    pub found: Vec<(String, SourceEntry)>,
    /// The patterns of every ignore file.
    pub ignore: PatternSet,
    /// Targets the walk skipped because they were ignored.
    pub ignored: BTreeSet<String>,
}

impl Walk {
    pub fn is_ignored(&self, target: &str) -> bool {
        self.ignore.matches(target) == PatternMatch::Include
    }
}

pub(super) fn walk(system: &dyn System, root: &Path) -> Result<Walk, ReadError> {
    let mut walker = Walker {
        system,
        walk: Walk {
            found: Vec::new(),
            ignore: PatternSet::default(),
            ignored: BTreeSet::new(),
        },
    };
    walker.walk_dir(root, "", "")?;
    Ok(walker.walk)
}

struct Walker<'a> {
    system: &'a dyn System,
    walk: Walk,
}

impl Walker<'_> {
    /// Walk `dir`, whose path relative to the source directory is `source_rel` and whose
    /// target path is `target` (both empty for the source directory itself).
    fn walk_dir(&mut self, dir: &Path, source_rel: &str, target: &str) -> Result<(), ReadError> {
        for name in self.system.read_dir(dir).map_err(io_error(dir))? {
            let path = dir.join(&name);
            let name = utf8(&name, &path)?;
            let entry = Entry {
                source_rel: join(source_rel, name),
                path,
                name,
            };
            if name.starts_with(PREFIX) {
                self.visit_special(&entry, target)?;
            } else if !name.starts_with(IGNORE_PREFIX) {
                self.visit(&entry, target)?;
            }
        }
        Ok(())
    }

    /// A name starting with the special prefix. Special files that are not supported yet are
    /// skipped, like any other name starting with `.`.
    fn visit_special(&mut self, entry: &Entry<'_>, parent_target: &str) -> Result<(), ReadError> {
        match entry.name {
            IGNORE => self.read_ignore_file(entry, parent_target),
            SCRIPTS_DIR if self.is_dir(&entry.path)? => scripts::walk(
                self.system,
                &entry.path,
                &entry.source_rel,
                &join(parent_target, SCRIPTS_DIR),
                &mut self.walk.found,
            ),
            _ => Ok(()),
        }
    }

    fn read_ignore_file(&mut self, entry: &Entry<'_>, dir_target: &str) -> Result<(), ReadError> {
        let data = self
            .system
            .read_file(&entry.path)
            .map_err(io_error(&entry.path))?;
        let data = String::from_utf8_lossy(&data);
        add_patterns(&mut self.walk.ignore, &data, dir_target).map_err(|e| ReadError::Pattern {
            path: entry.path.clone(),
            line: e.line,
            message: e.message,
        })
    }

    fn visit(&mut self, entry: &Entry<'_>, parent_target: &str) -> Result<(), ReadError> {
        match followed_metadata(self.system, &entry.path)?.file_type {
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
        if self.skip_ignored(&target) {
            return Ok(());
        }
        self.walk.found.push((
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
        let target = join(parent_target, &attr.target_name);
        if self.skip_ignored(&target) {
            return Ok(());
        }
        self.walk.found.push((
            target,
            SourceEntry::File {
                source_path: entry.path.clone(),
                source_rel: entry.source_rel.clone(),
                attr,
            },
        ));
        Ok(())
    }

    /// Whether `target` is ignored, noting it if so.
    fn skip_ignored(&mut self, target: &str) -> bool {
        let ignored = self.walk.is_ignored(target);
        if ignored {
            self.walk.ignored.insert(target.to_owned());
        }
        ignored
    }

    fn is_dir(&self, path: &Path) -> Result<bool, ReadError> {
        Ok(followed_metadata(self.system, path)?.file_type == FileType::Dir)
    }
}

/// A source entry being visited.
struct Entry<'a> {
    path: PathBuf,
    name: &'a str,
    source_rel: String,
}
