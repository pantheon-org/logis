//! Which targets `apply` and `diff` work on.

use std::path::{Path, PathBuf};

use logis_core::source_state::SourceState;

use super::dirs::Dirs;

/// The targets a command works on, sorted and once each: every target if no arguments are
/// given, else the arguments' targets (found by source path with `--source-path`) and, with
/// `recursive`, everything under them. As chezmoi's `applyArgs`.
pub fn select(
    state: &SourceState,
    dirs: &Dirs,
    source_path: bool,
    recursive: bool,
    args: &[PathBuf],
) -> Result<Vec<String>, String> {
    if args.is_empty() {
        return Ok(state.entries().map(|(t, _)| t.to_owned()).collect());
    }
    let mut targets = Vec::new();
    for arg in args {
        let target = if source_path {
            by_source_path(state, dirs, arg)?
        } else {
            let target = dirs.target(arg)?;
            if state.get(&target).is_none() {
                return Err(format!("{}: not managed", arg.display()));
            }
            if recursive {
                let prefix = format!("{target}/");
                targets.extend(
                    state
                        .entries()
                        .map(|(t, _)| t)
                        .filter(|t| t.starts_with(&prefix))
                        .map(str::to_owned),
                );
            }
            target
        };
        targets.push(target);
    }
    targets.sort();
    targets.dedup();
    Ok(targets)
}

/// chezmoi's `targetRelPathsBySourcePath`: the target whose source entry is at `arg`.
fn by_source_path(state: &SourceState, dirs: &Dirs, arg: &Path) -> Result<String, String> {
    let rel = dirs.source_rel(arg)?;
    state
        .entries()
        .find(|(_, entry)| entry.source_rel() == rel)
        .map(|(target, _)| target.to_owned())
        .ok_or_else(|| format!("{}: not in source state", arg.display()))
}
