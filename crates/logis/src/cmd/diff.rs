//! `logis diff`: print, as a git diff, the changes `apply` would make.

use std::io::Write;
use std::path::PathBuf;

use logis_core::apply::{ApplyOptions, apply};
use logis_core::diff::GitDiffSystem;
use logis_core::entry_type::{EntryTypeFilter, EntryTypeSet};
use logis_core::source_state::SourceState;
use logis_core::system::{DryRunSystem, RealSystem};

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

    /// Also diff everything under the targets given.
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

    /// From the target state to the destination, instead of the other way.
    #[arg(long)]
    reverse: bool,

    /// Only these targets (all if none).
    targets: Vec<PathBuf>,
}

/// An apply through a dry run, with every write recorded as a patch. It changes nothing, so
/// it needs no `--force` (ADR-009 concerns writes only).
pub fn run(dirs: &Dirs, globals: Globals, args: &Args) -> Result<(), String> {
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
    let filter = EntryTypeFilter {
        include: args.include,
        exclude: args.exclude,
    };
    let dry_run = DryRunSystem::new(&RealSystem);
    let differ = GitDiffSystem::new(&dry_run, &dirs.destination, filter, args.reverse);
    let options = ApplyOptions {
        dest_dir: &dirs.destination,
        umask: globals.umask,
        filter,
    };
    let result = apply(&differ, &state, &targets, &options).map_err(|e| e.to_string());
    std::io::stdout()
        .write_all(&differ.into_output())
        .map_err(|e| format!("stdout: {e}"))?;
    result
}
