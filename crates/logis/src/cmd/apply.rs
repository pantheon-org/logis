//! `logis apply`: make the destination match the target state.

use std::path::PathBuf;

use logis_core::apply::{ApplyOptions, apply};
use logis_core::entry_type::{EntryTypeFilter, EntryTypeSet};
use logis_core::source_state::SourceState;
use logis_core::system::{DryRunSystem, RealSystem, System};

use super::dirs::Dirs;
use super::globals::Globals;
use super::targets::select;

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
    let targets = select(
        &state,
        dirs,
        globals.source_path,
        args.recursive,
        &args.targets,
    )?;
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
