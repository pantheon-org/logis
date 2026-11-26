//! Changing one destination entry into its target state, ported from the `Apply` methods of
//! chezmoi's target state entries (`internal/chezmoi/targetstateentry.go`). Each returns
//! whether it changed anything.

use std::path::Path;

use super::actual::Actual;
use super::error::{ApplyError, io_error};
use crate::system::System;
use crate::target_state::TargetEntry;

pub(super) fn apply_entry(
    system: &dyn System,
    target: &TargetEntry,
    path: &Path,
) -> Result<bool, ApplyError> {
    let actual = Actual::read(system, path)?;
    match target {
        TargetEntry::Dir { perm } => dir(system, *perm, actual, path),
        TargetEntry::File {
            contents,
            perm,
            empty,
        } => file(system, contents, *perm, *empty, actual, path),
        TargetEntry::Symlink { linkname } => symlink(system, linkname, actual, path),
        TargetEntry::Script { .. } => Err(ApplyError::Unsupported("scripts")),
        TargetEntry::Absent => absent(system, actual, path),
    }
}

fn dir(system: &dyn System, perm: u32, actual: Actual, path: &Path) -> Result<bool, ApplyError> {
    if let Actual::Dir { perm: actual_perm } = actual {
        if actual_perm == perm {
            return Ok(false);
        }
        system.chmod(path, perm).map_err(io_error(path))?;
        return Ok(true);
    }
    actual.remove(system, path)?;
    system.mkdir(path, perm).map_err(io_error(path))?;
    Ok(true)
}

/// A file whose contents are only whitespace is removed, unless it is meant to be empty.
/// chezmoi's `isEmpty` uses Go's `bytes.TrimSpace`, which also trims Unicode spaces; only
/// ASCII whitespace counts here.
fn file(
    system: &dyn System,
    contents: &[u8],
    perm: u32,
    empty: bool,
    actual: Actual,
    path: &Path,
) -> Result<bool, ApplyError> {
    if !empty && contents.trim_ascii().is_empty() {
        return absent(system, actual, path);
    }
    if let Actual::File { perm: actual_perm } = actual {
        let actual_contents = system.read_file(path).map_err(io_error(path))?;
        if actual_contents == contents {
            if actual_perm == perm {
                return Ok(false);
            }
            system.chmod(path, perm).map_err(io_error(path))?;
            return Ok(true);
        }
    } else {
        actual.remove(system, path)?;
    }
    system
        .write_file(path, contents, perm)
        .map_err(io_error(path))?;
    Ok(true)
}

/// An empty link target removes the entry.
fn symlink(
    system: &dyn System,
    linkname: &str,
    actual: Actual,
    path: &Path,
) -> Result<bool, ApplyError> {
    if linkname.is_empty() {
        return absent(system, actual, path);
    }
    if actual == Actual::Symlink {
        let actual_linkname = system.read_link(path).map_err(io_error(path))?;
        if actual_linkname == Path::new(linkname) {
            return Ok(false);
        }
    }
    actual.remove(system, path)?;
    system
        .write_symlink(Path::new(linkname), path)
        .map_err(io_error(path))?;
    Ok(true)
}

fn absent(system: &dyn System, actual: Actual, path: &Path) -> Result<bool, ApplyError> {
    if actual == Actual::Absent {
        return Ok(false);
    }
    actual.remove(system, path)?;
    Ok(true)
}
