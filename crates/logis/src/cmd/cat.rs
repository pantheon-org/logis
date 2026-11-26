//! `logis cat`: print the target state of files, scripts and symlinks.

use std::io::Write;
use std::path::PathBuf;

use logis_core::source_state::SourceState;
use logis_core::system::RealSystem;
use logis_core::target_state::{TargetEntry, target_entry};

use super::dirs::Dirs;

/// `cat` prints no permissions, so the umask does not change its output. Commands that write
/// or compare permissions must use the real one.
const UMASK: u32 = 0o022;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// The targets to print.
    #[arg(required = true)]
    targets: Vec<PathBuf>,
}

/// Print every target, in target order and once each. Nothing is printed unless every target
/// can be, as in chezmoi.
pub fn run(dirs: &Dirs, args: &Args) -> Result<(), String> {
    let state = SourceState::read(&RealSystem, &dirs.source, &dirs.destination)
        .map_err(|e| e.to_string())?;
    let mut targets = Vec::with_capacity(args.targets.len());
    for arg in &args.targets {
        let target = dirs.target(arg)?;
        if state.get(&target).is_none() {
            return Err(format!("{}: not managed", arg.display()));
        }
        targets.push(target);
    }
    // chezmoi's CompareRelPaths compares the paths as strings.
    targets.sort();
    targets.dedup();

    let mut out = Vec::new();
    for target in &targets {
        let entry = state.get(target).expect("checked above");
        let dest_path = dirs.destination.join(target);
        match target_entry(&RealSystem, entry, &dest_path, UMASK)
            .map_err(|e| format!("{target}: {e}"))?
        {
            TargetEntry::File { contents, .. } | TargetEntry::Script { contents, .. } => {
                out.extend_from_slice(&contents);
            }
            TargetEntry::Symlink { linkname } => {
                out.extend_from_slice(linkname.as_bytes());
                out.push(b'\n');
            }
            TargetEntry::Dir { .. } | TargetEntry::Absent => {
                return Err(format!("{target}: not a file, script, or symlink"));
            }
        }
    }
    std::io::stdout()
        .write_all(&out)
        .map_err(|e| format!("stdout: {e}"))
}
