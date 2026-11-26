use std::path::{Path, PathBuf};

use crate::source_name::{DirAttr, FileAttr};

/// One entry of the source directory and what it says about its target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceEntry {
    Dir {
        /// Absolute path of the source entry (a symlink in the source directory stays the
        /// symlink's path).
        source_path: PathBuf,
        /// The same path relative to the source directory, `/`-separated.
        source_rel: String,
        attr: DirAttr,
    },
    File {
        source_path: PathBuf,
        source_rel: String,
        attr: FileAttr,
    },
    /// An entry in the destination, under an exact directory, that the source state does not
    /// manage: it is to be removed.
    Absent {
        /// Absolute source path of the exact directory that implies it.
        exact_dir: PathBuf,
    },
}

impl SourceEntry {
    /// The source entry's absolute path; for an absent entry, its exact directory's.
    #[must_use]
    pub fn source_path(&self) -> &Path {
        match self {
            Self::Dir { source_path, .. } | Self::File { source_path, .. } => source_path,
            Self::Absent { exact_dir } => exact_dir,
        }
    }

    #[must_use]
    pub fn source_rel(&self) -> &str {
        match self {
            Self::Dir { source_rel, .. } | Self::File { source_rel, .. } => source_rel,
            // Empty, as chezmoi's `SourceStateAbsent.SourceRelPath` is.
            Self::Absent { .. } => "",
        }
    }
}
