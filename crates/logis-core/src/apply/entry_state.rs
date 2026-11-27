//! Entry states: what a target is, reduced to what decides whether it changes. Ported from
//! chezmoi's `EntryState` (`internal/chezmoi/entrystate.go`) and the `EntryState` methods of
//! its target and actual state entries. chezmoi stores a SHA-256 of the contents; logis keeps
//! the contents, since it does not store entry states yet (ADR-009).

use std::path::Path;

use super::actual::Actual;
use super::error::{ApplyError, io_error};
use crate::system::System;
use crate::target_state::TargetEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    /// Nothing there, or nothing meant to be there.
    Remove,
    Dir,
    File,
    Symlink,
    Script,
}

/// chezmoi's `EntryState.Equal`: same kind, same permission bits, same contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EntryState {
    pub kind: Kind,
    pub perm: u32,
    pub contents: Vec<u8>,
}

impl EntryState {
    fn new(kind: Kind, perm: u32, contents: Vec<u8>) -> Self {
        Self {
            kind,
            perm,
            contents,
        }
    }

    /// The state `target` asks for. A file of only whitespace and a symlink to nothing are
    /// to be removed, as `apply` does with them.
    pub fn of_target(target: &TargetEntry, umask: u32) -> Self {
        match target {
            TargetEntry::Dir { perm } => Self::new(Kind::Dir, perm & !umask, Vec::new()),
            TargetEntry::File {
                contents,
                perm,
                empty,
            } => {
                if !empty && contents.trim_ascii().is_empty() {
                    Self::new(Kind::Remove, 0, Vec::new())
                } else {
                    Self::new(Kind::File, perm & !umask, contents.clone())
                }
            }
            TargetEntry::Symlink { linkname } if linkname.is_empty() => {
                Self::new(Kind::Remove, 0, Vec::new())
            }
            TargetEntry::Symlink { linkname } => {
                Self::new(Kind::Symlink, 0, linkname.as_bytes().to_vec())
            }
            TargetEntry::Script { contents, .. } => Self::new(Kind::Script, 0, contents.clone()),
            TargetEntry::Absent => Self::new(Kind::Remove, 0, Vec::new()),
        }
    }

    /// The state of what is at `path` now.
    pub fn of_actual(system: &dyn System, path: &Path) -> Result<Self, ApplyError> {
        Ok(match Actual::read(system, path)? {
            Actual::Absent => Self::new(Kind::Remove, 0, Vec::new()),
            Actual::Dir { perm } => Self::new(Kind::Dir, perm, Vec::new()),
            Actual::File { perm } => {
                let contents = system.read_file(path).map_err(io_error(path))?;
                Self::new(Kind::File, perm, contents)
            }
            Actual::Symlink => {
                let linkname = system.read_link(path).map_err(io_error(path))?;
                let linkname = linkname.as_os_str().as_encoded_bytes().to_vec();
                Self::new(Kind::Symlink, 0, linkname)
            }
        })
    }
}
