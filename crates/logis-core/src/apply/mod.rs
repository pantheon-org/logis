//! Making the destination match the target state, ported from chezmoi's `SourceState.Apply`
//! and `Config.applyArgs` (`internal/chezmoi/sourcestate.go`, `internal/cmd/config.go`).
//!
//! What is in the destination now is read in `actual.rs`, and each kind of target is applied
//! in `entry.rs`. `status.rs` reports what apply would change, from the entry states in
//! `entry_state.rs`. Not supported yet: scripts (phase 5), recording what was written (the state
//! store), and removing empty `remove_` directories afterwards (`SourceState.PostApply`).

mod actual;
mod entry;
mod entry_state;
mod error;
mod options;
mod status;

use crate::source_state::SourceState;
use crate::system::System;
use crate::target_state::target_entry;

pub use error::{ApplyError, TargetApplyError};
pub use options::ApplyOptions;
pub use status::{Status, status};

/// Apply `targets` (paths relative to the destination directory), in the order given, which
/// should be sorted so that directories come before their contents. Targets the source state
/// does not manage are skipped. The first error stops the run.
pub fn apply(
    system: &dyn System,
    state: &SourceState,
    targets: &[&str],
    options: &ApplyOptions<'_>,
) -> Result<(), TargetApplyError> {
    for target in targets {
        apply_target(system, state, target, options).map_err(|error| TargetApplyError {
            target: (*target).to_owned(),
            error,
        })?;
    }
    Ok(())
}

fn apply_target(
    system: &dyn System,
    state: &SourceState,
    target: &str,
    options: &ApplyOptions<'_>,
) -> Result<(), ApplyError> {
    let Some(entry) = state.get(target) else {
        return Ok(());
    };
    if !options.filter.includes(entry) {
        return Ok(());
    }
    let path = options.dest_dir.join(target);
    let target_entry = target_entry(system, entry, &path, options.umask)?;
    entry::apply_entry(system, &target_entry, &path)?;
    Ok(())
}
