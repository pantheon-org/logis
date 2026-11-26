//! `logis managed`: list the targets the source state manages.

use std::path::PathBuf;

use logis_core::entry_type::{EntryTypeFilter, EntryTypeSet};
use logis_core::managed::{PathStyle, Query, managed};
use logis_core::source_state::SourceState;
use logis_core::system::RealSystem;

use super::dirs::Dirs;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Only these entry types (comma-separated: all, dirs, files, remove, scripts, symlinks,
    /// encrypted, externals, templates, always; `noX` removes X).
    #[arg(long, short = 'i', default_value = "all", value_name = "TYPES")]
    include: EntryTypeSet,

    /// Not these entry types.
    #[arg(long, short = 'x', default_value = "none", value_name = "TYPES")]
    exclude: EntryTypeSet,

    /// relative, absolute, source-relative or source-absolute.
    #[arg(long, short = 'p', default_value = "relative", value_name = "STYLE")]
    path_style: PathStyle,

    /// Only these targets and their descendants.
    targets: Vec<PathBuf>,
}

pub fn run(dirs: &Dirs, args: &Args) -> Result<(), String> {
    let state = SourceState::read(&RealSystem, &dirs.source, &dirs.destination)
        .map_err(|e| e.to_string())?;
    let targets = args
        .targets
        .iter()
        .map(|arg| dirs.target(arg))
        .collect::<Result<Vec<_>, _>>()?;
    let targets: Vec<&str> = targets.iter().map(String::as_str).collect();
    let query = Query {
        filter: EntryTypeFilter {
            include: args.include,
            exclude: args.exclude,
        },
        targets: &targets,
        style: args.path_style,
        source_dir: &dirs.source,
        dest_dir: &dirs.destination,
    };
    for line in managed(&state, &query) {
        println!("{line}");
    }
    Ok(())
}
