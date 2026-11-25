//! The source state: every target the source directory manages, with the source entry and
//! attributes that describe it.
//!
//! Ported from chezmoi's `SourceState.Read` (`internal/chezmoi/sourcestate.go`), one concern
//! at a time instead of one function: walking the tree (`read.rs`), and finding targets with
//! more than one source entry (`SourceState::from_candidates`). Not yet supported: special
//! files (ignore, remove, data, templates, externals, scripts directory, version), which are
//! skipped like any other name starting with `.`.

mod entry;
mod error;
mod read;

use std::collections::BTreeMap;
use std::path::Path;

use crate::system::{FileType, System};

pub use entry::SourceEntry;
pub use error::ReadError;

/// Targets, as `/`-separated paths relative to the destination directory, mapped to the source
/// entry that manages each. Iteration is in target-path order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceState {
    entries: BTreeMap<String, SourceEntry>,
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
        Self::from_candidates(read::walk(system, source_dir)?)
    }

    /// Build the state from every candidate entry. A target with more than one source entry is
    /// an inconsistent state; all such targets are reported together, as chezmoi does.
    fn from_candidates(candidates: Vec<(String, SourceEntry)>) -> Result<Self, ReadError> {
        let mut by_target: BTreeMap<String, Vec<SourceEntry>> = BTreeMap::new();
        for (target, entry) in candidates {
            by_target.entry(target).or_default().push(entry);
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
        Ok(Self { entries })
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
