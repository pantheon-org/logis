//! `logis unmanaged`: list the entries in the destination that the source state does not
//! manage.

use std::path::PathBuf;

use logis_core::entry_type::{EntryTypeFilter, EntryTypeSet};
use logis_core::path_tree::tree;
use logis_core::source_state::SourceState;
use logis_core::system::RealSystem;
use logis_core::unmanaged::unmanaged;

use super::dirs::Dirs;

/// chezmoi's `unmanaged` takes only these two of `managed`'s path styles.
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

    /// Print the paths as a tree.
    #[arg(long, short = 't')]
    tree: bool,

    /// Only under these paths (the whole destination if none).
    paths: Vec<PathBuf>,
}

/// Paths that cannot be read below a root are reported and skipped; the rest is printed.
pub fn run(dirs: &Dirs, args: &Args) -> Result<(), String> {
    let state = SourceState::read(&RealSystem, &dirs.source, &dirs.destination)
        .map_err(|e| e.to_string())?;
    let roots: Vec<PathBuf> = if args.paths.is_empty() {
        vec![dirs.destination.clone()]
    } else {
        args.paths.iter().map(|p| dirs.absolute(p)).collect()
    };
    let filter = EntryTypeFilter {
        include: args.include,
        exclude: args.exclude,
    };
    let found = unmanaged(
        &RealSystem,
        &state,
        &dirs.destination,
        &roots,
        filter,
        &mut |warning| eprintln!("logis: {warning}"),
    )
    .map_err(|e| e.to_string())?;
    let paths: Vec<String> = match args.path_style {
        PathStyle::Relative => found,
        PathStyle::Absolute => found
            .iter()
            .map(|rel| dirs.destination.join(rel).display().to_string())
            .collect(),
    };
    if args.tree {
        print!("{}", tree(&paths));
    } else {
        for path in paths {
            println!("{path}");
        }
    }
    Ok(())
}
