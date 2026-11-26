//! The target state: what each source entry says its target should be. Ported from chezmoi's
//! `newSourceStateDir`, `newSourceStateFile` and the `new*TargetStateEntryFunc` functions
//! (`internal/chezmoi/sourcestate.go`).
//!
//! chezmoi computes contents lazily; logis reads them when the entry is computed, so ask for
//! the target state only of entries whose contents are needed. Not supported yet: templates
//! (phase 4), encrypted files and modify scripts (phase 5), and chezmoi's symlink mode.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::source_name::{FileAttr, ScriptCondition, TargetType};
use crate::source_state::SourceEntry;
use crate::system::System;

/// What a target should be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetEntry {
    Dir {
        perm: u32,
    },
    File {
        contents: Vec<u8>,
        perm: u32,
        /// The file is kept even when empty (`empty_`).
        empty: bool,
    },
    Symlink {
        linkname: String,
    },
    Script {
        contents: Vec<u8>,
        condition: ScriptCondition,
    },
    /// The target should not exist.
    Absent,
}

#[derive(Debug)]
pub enum TargetError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    /// A source attribute that logis cannot compute a target for yet.
    Unsupported(&'static str),
}

impl fmt::Display for TargetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Unsupported(what) => write!(f, "{what} are not supported yet"),
        }
    }
}

impl std::error::Error for TargetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Unsupported(_) => None,
        }
    }
}

/// The target state of `entry`, whose target is at `dest_path`. Permissions are the source
/// attributes' less `umask`.
pub fn target_entry(
    system: &dyn System,
    entry: &SourceEntry,
    dest_path: &Path,
    umask: u32,
) -> Result<TargetEntry, TargetError> {
    match entry {
        SourceEntry::Dir { attr, .. } => Ok(TargetEntry::Dir {
            perm: attr.perm() & !umask,
        }),
        SourceEntry::File {
            source_path, attr, ..
        } => file_entry(system, source_path, attr, dest_path, umask),
        SourceEntry::Absent { .. } => Ok(TargetEntry::Absent),
    }
}

fn file_entry(
    system: &dyn System,
    source_path: &Path,
    attr: &FileAttr,
    dest_path: &Path,
    umask: u32,
) -> Result<TargetEntry, TargetError> {
    unsupported(attr)?;
    let source = || read(system, source_path);
    let file = |contents| TargetEntry::File {
        contents,
        perm: attr.perm() & !umask,
        empty: attr.empty,
    };
    match attr.target_type {
        TargetType::File => Ok(file(source()?)),
        TargetType::Create => match system.read_file(dest_path) {
            Ok(existing) => Ok(file(existing)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(file(source()?)),
            Err(source) => Err(TargetError::Io {
                path: dest_path.to_owned(),
                source,
            }),
        },
        TargetType::Symlink => Ok(TargetEntry::Symlink {
            linkname: String::from_utf8_lossy(&source()?).trim().to_owned(),
        }),
        TargetType::Script => Ok(TargetEntry::Script {
            contents: source()?,
            condition: attr.condition,
        }),
        TargetType::Remove => Ok(TargetEntry::Absent),
        TargetType::Modify => Err(TargetError::Unsupported("modify scripts")),
    }
}

/// Templates and encryption change the contents of every target type, so they are checked
/// first.
fn unsupported(attr: &FileAttr) -> Result<(), TargetError> {
    if attr.template {
        return Err(TargetError::Unsupported("templates"));
    }
    if attr.encrypted {
        return Err(TargetError::Unsupported("encrypted files"));
    }
    Ok(())
}

fn read(system: &dyn System, path: &Path) -> Result<Vec<u8>, TargetError> {
    system.read_file(path).map_err(|source| TargetError::Io {
        path: path.to_owned(),
        source,
    })
}
