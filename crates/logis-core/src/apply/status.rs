//! What `status` reports: chezmoi's `runStatusCmd` pre-apply function and `statusRune`
//! (`internal/cmd/statuscmd.go`). Nothing is changed.

use super::entry_state::{EntryState, Kind};
use super::error::{ApplyError, TargetApplyError};
use super::options::ApplyOptions;
use crate::source_state::SourceState;
use crate::system::System;
use crate::target_state::{TargetEntry, target_entry};

/// One line of `status`: the target and its two status letters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub target: String,
    /// The change in the destination since logis last wrote the target. Always `' '` until
    /// logis records what it wrote (ADR-009).
    pub x: char,
    /// The change apply would make: `A` add, `M` modify, `D` delete or `R` run.
    pub y: char,
}

/// The status of each of `targets` that apply would change, in the order given. Targets the
/// source state does not manage, or that the filter leaves out, are skipped.
pub fn status(
    system: &dyn System,
    state: &SourceState,
    targets: &[&str],
    options: &ApplyOptions<'_>,
) -> Result<Vec<Status>, TargetApplyError> {
    let mut lines = Vec::new();
    for target in targets {
        let wrap = |error| TargetApplyError {
            target: (*target).to_owned(),
            error,
        };
        if let Some(y) = change(system, state, target, options).map_err(wrap)? {
            lines.push(Status {
                target: (*target).to_owned(),
                x: ' ',
                y,
            });
        }
    }
    Ok(lines)
}

/// The letter for what apply would do to `target`, if anything.
fn change(
    system: &dyn System,
    state: &SourceState,
    target: &str,
    options: &ApplyOptions<'_>,
) -> Result<Option<char>, ApplyError> {
    let Some(entry) = state.get(target) else {
        return Ok(None);
    };
    if !options.filter.includes(entry) {
        return Ok(None);
    }
    let path = options.dest_dir.join(target);
    let target_entry = target_entry(system, entry, &path, options.umask)?;
    if let TargetEntry::Script { contents, .. } = &target_entry {
        // `TargetStateScript.SkipApply`: an empty script is never run.
        return Ok((!contents.is_empty()).then_some('R'));
    }
    let wanted = EntryState::of_target(&target_entry, options.umask);
    let actual = EntryState::of_actual(system, &path)?;
    Ok((wanted != actual).then(|| letter(&actual, &wanted)))
}

/// chezmoi's `statusRune` for states that differ.
fn letter(from: &EntryState, to: &EntryState) -> char {
    match (from.kind, to.kind) {
        (_, Kind::Remove) => 'D',
        (_, Kind::Script) => 'R',
        (Kind::Remove, _) => 'A',
        _ => 'M',
    }
}
