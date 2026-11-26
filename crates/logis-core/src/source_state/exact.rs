//! Absent entries for exact directories, ported from the "Generate `SourceStateAbsents` for exact
//! directories" step of chezmoi's `SourceState.Read`.

use std::collections::{BTreeMap, BTreeSet};
use std::io::ErrorKind;
use std::path::Path;

use super::entry::SourceEntry;
use super::error::ReadError;
use super::walk_util::{io_error, join, utf8};
use crate::pattern_set::{PatternMatch, PatternSet};
use crate::system::System;

/// An absent entry for every destination entry directly under an exact directory in
/// `entries` that `entries` does not manage and `ignore` does not ignore. Ignored ones are
/// added to `ignored`. A destination directory that is missing, or is not a directory, adds
/// nothing.
pub(super) fn absents(
    system: &dyn System,
    dest_dir: &Path,
    entries: &BTreeMap<String, SourceEntry>,
    ignore: &PatternSet,
    ignored: &mut BTreeSet<String>,
) -> Result<Vec<(String, SourceEntry)>, ReadError> {
    let mut found = Vec::new();
    for (target, entry) in entries {
        let SourceEntry::Dir {
            source_path, attr, ..
        } = entry
        else {
            continue;
        };
        if !attr.exact {
            continue;
        }
        for name in dest_names(system, &dest_dir.join(target))? {
            let child = join(target, &name);
            if entries.contains_key(&child) {
                continue;
            }
            if ignore.matches(&child) == PatternMatch::Include {
                ignored.insert(child);
                continue;
            }
            found.push((
                child,
                SourceEntry::Absent {
                    exact_dir: source_path.clone(),
                },
            ));
        }
    }
    Ok(found)
}

/// The names in destination directory `dir`, or none if it is missing or not a directory.
fn dest_names(system: &dyn System, dir: &Path) -> Result<Vec<String>, ReadError> {
    match system.read_dir(dir) {
        Ok(names) => names
            .iter()
            .map(|name| utf8(name, &dir.join(name)).map(str::to_owned))
            .collect(),
        Err(err) if matches!(err.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {
            Ok(Vec::new())
        }
        Err(err) => Err(io_error(dir)(err)),
    }
}
