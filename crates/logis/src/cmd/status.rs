//! `logis status`: list what `apply` would change, one target per line.

use std::path::PathBuf;

use logis_core::apply::{ApplyOptions, Status, status};
use logis_core::entry_type::{EntryTypeFilter, EntryTypeSet};
use logis_core::source_state::SourceState;
use logis_core::system::RealSystem;

use super::dirs::Dirs;
use super::globals::Globals;
use super::targets::select;

/// chezmoi's `status` takes only these two of `managed`'s path styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum PathStyle {
    Relative,
    Absolute,
}

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Only these entry types (as for `managed`).
    #[arg(long, short = 'i', default_value = "all", value_name = "TYPES")]
    include: EntryTypeSet,

    /// Not these entry types.
    #[arg(long, short = 'x', default_value = "none", value_name = "TYPES")]
    exclude: EntryTypeSet,

    /// relative or absolute.
    #[arg(
        long,
        short = 'p',
        value_enum,
        default_value = "relative",
        value_name = "STYLE"
    )]
    path_style: PathStyle,

    /// Also list everything under the targets given.
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

/// Each line is two status letters and the path. The first letter, the change since logis
/// last wrote the target, stays blank until logis records what it wrote (ADR-009); status
/// changes nothing, so it needs no `--force`.
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
    let options = ApplyOptions {
        dest_dir: &dirs.destination,
        umask: globals.umask,
        filter: EntryTypeFilter {
            include: args.include,
            exclude: args.exclude,
        },
    };
    let lines = status(&RealSystem, &state, &targets, &options).map_err(|e| e.to_string())?;
    for Status { target, x, y } in lines {
        let path = match args.path_style {
            PathStyle::Relative => target,
            PathStyle::Absolute => dirs.destination.join(&target).display().to_string(),
        };
        println!("{x}{y} {path}");
    }
    Ok(())
}
