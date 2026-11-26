//! The source state: every target the source directory manages, with the source entry and
//! attributes that describe it.
//!
//! Ported from chezmoi's `SourceState.Read` (`internal/chezmoi/sourcestate.go`), one concern
//! at a time instead of one function: walking the tree (`read.rs`), the ignore file's lines
//! (`patterns.rs`), the scripts directory (`scripts.rs`), and dropping ignored targets and
//! finding targets with more than one source entry (`SourceState::from_walk`). Not yet
//! supported: the remove file, data, templates, externals and the version file, which are
//! skipped like any other name starting with `.`.

mod entry;
mod error;
mod patterns;
mod read;
mod scripts;
mod walk_util;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::system::{FileType, System};

pub use entry::SourceEntry;
pub use error::ReadError;

/// Targets, as `/`-separated paths relative to the destination directory, mapped to the source
/// entry that manages each. Iteration is in target-path order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceState {
    entries: BTreeMap<String, SourceEntry>,
    ignored: BTreeSet<String>,
}

impl SourceState {
    /// Read the source directory. A missing source directory is an empty source state.
    pub fn read(system: &dyn System, source_dir: &Path) -> Result<Self, ReadError> {
        match system.stat(source_dir) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => {
                return Err(ReadError::Io {
                    path: source_dir.to_owned(),
                    source,
                });
            }
            Ok(meta) if meta.file_type != FileType::Dir => {
                return Err(ReadError::NotADirectory(source_dir.to_owned()));
            }
            Ok(_) => {}
        }
        Self::from_walk(read::walk(system, source_dir)?)
    }

    /// Build the state from what the walk found. Ignored targets are dropped first, so they
    /// cannot conflict. A target with more than one source entry is an inconsistent state; all
    /// such targets are reported together, as chezmoi does.
    fn from_walk(mut walk: read::Walk) -> Result<Self, ReadError> {
        let found = std::mem::take(&mut walk.found);
        let mut ignored = std::mem::take(&mut walk.ignored);
        let mut by_target: BTreeMap<String, Vec<SourceEntry>> = BTreeMap::new();
        for (target, entry) in found {
            if walk.is_ignored(&target) {
                ignored.insert(target);
            } else {
                by_target.entry(target).or_default().push(entry);
            }
        }
        let inconsistent: Vec<(String, Vec<String>)> = by_target
            .iter()
            .filter(|(_, entries)| entries.len() > 1)
            .map(|(target, entries)| {
                let mut origins: Vec<String> = entries
                    .iter()
                    .map(|e| e.source_path().display().to_string())
                    .collect();
                origins.sort();
                (target.clone(), origins)
            })
            .collect();
        if !inconsistent.is_empty() {
            return Err(ReadError::Inconsistent(inconsistent));
        }
        let entries = by_target
            .into_iter()
            .filter_map(|(target, mut entries)| entries.pop().map(|entry| (target, entry)))
            .collect();
        Ok(Self { entries, ignored })
    }

    /// Targets the source state has entries or ignore patterns for but does not manage
    /// because they are ignored, in target-path order. The contents of an ignored directory
    /// are not read, so they are not listed.
    pub fn ignored(&self) -> impl Iterator<Item = &str> {
        self.ignored.iter().map(String::as_str)
    }

    pub fn entries(&self) -> impl Iterator<Item = (&str, &SourceEntry)> {
        self.entries
            .iter()
            .map(|(target, entry)| (target.as_str(), entry))
    }

    #[must_use]
    pub fn get(&self, target: &str) -> Option<&SourceEntry> {
        self.entries.get(target)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
