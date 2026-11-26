//! `logis apply`: make the destination match the target state.

use std::path::{Path, PathBuf};

use logis_core::apply::{ApplyOptions, apply};
use logis_core::entry_type::{EntryTypeFilter, EntryTypeSet};
use logis_core::source_state::SourceState;
use logis_core::system::{DryRunSystem, RealSystem, System};

use super::dirs::Dirs;
use super::globals::Globals;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Only these entry types (as for `managed`).
    #[arg(long, short = 'i', default_value = "all", value_name = "TYPES")]
    include: EntryTypeSet,

    /// Not these entry types.
    #[arg(long, short = 'x', default_value = "none", value_name = "TYPES")]
    exclude: EntryTypeSet,

    /// Also apply everything under the targets given.
    #[arg(
        long,
        short = 'r',
        default_value_t = true,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        value_name = "BOOL"
    )]
    recursive: bool,

    /// Only these targets (all if none).
    targets: Vec<PathBuf>,
}

/// Until logis records what it wrote, it cannot tell a target the user edited from one it
/// wrote, so it changes nothing without `--force` (ADR-009).
const NEEDS_FORCE: &str = "apply needs --force: logis does not yet record what it wrote, so it \
                           cannot tell your edits from its own (ADR-009)";

pub fn run(dirs: &Dirs, globals: Globals, args: &Args) -> Result<(), String> {
    if !globals.force && !globals.dry_run {
        return Err(NEEDS_FORCE.to_owned());
    }
    let state = SourceState::read(&RealSystem, &dirs.source, &dirs.destination)
        .map_err(|e| e.to_string())?;
    let targets = targets(&state, dirs, globals.source_path, args)?;
    let targets: Vec<&str> = targets.iter().map(String::as_str).collect();
    let options = ApplyOptions {
        dest_dir: &dirs.destination,
        umask: globals.umask,
        filter: EntryTypeFilter {
            include: args.include,
            exclude: args.exclude,
        },
    };
    let dry_run = DryRunSystem::new(&RealSystem);
    let system: &dyn System = if globals.dry_run {
        &dry_run
    } else {
        &RealSystem
    };
    apply(system, &state, &targets, &options).map_err(|e| e.to_string())
}

/// The targets to apply, sorted and once each: every target if no arguments are given, else
/// the arguments' targets (found by source path with `--source-path`) and, with
/// `--recursive`, everything under them.
fn targets(
    state: &SourceState,
    dirs: &Dirs,
    source_path: bool,
    args: &Args,
) -> Result<Vec<String>, String> {
    if args.targets.is_empty() {
        return Ok(state.entries().map(|(t, _)| t.to_owned()).collect());
    }
    let mut targets = Vec::new();
    for arg in &args.targets {
        let target = if source_path {
            by_source_path(state, dirs, arg)?
        } else {
            let target = dirs.target(arg)?;
            if state.get(&target).is_none() {
                return Err(format!("{}: not managed", arg.display()));
            }
            if args.recursive {
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
