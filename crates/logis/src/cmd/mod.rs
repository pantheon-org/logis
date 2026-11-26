//! One module per command. Commands are added only when a BDD scenario for them is red.

mod apply;
mod cat;
mod dirs;
mod globals;
mod managed;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use globals::Globals;

/// Manage your dotfiles across machines.
#[derive(Debug, Parser)]
#[command(name = "logis", version, about)]
pub struct Cli {
    /// The source directory (default: `$XDG_DATA_HOME/logis`, else `~/.local/share/logis`).
    #[arg(long, short = 'S', global = true, value_name = "DIR")]
    source: Option<PathBuf>,

    /// The destination directory (default: your home directory).
    #[arg(long, short = 'D', global = true, value_name = "DIR")]
    destination: Option<PathBuf>,

    /// Change nothing; report as if changing.
    #[arg(long, short = 'n', global = true)]
    dry_run: bool,

    /// Change targets without asking.
    #[arg(long, global = true)]
    force: bool,

    /// Arguments are source paths, not target paths.
    #[arg(long, global = true)]
    source_path: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Make the destination match the target state.
    Apply(apply::Args),
    /// Print the target state of files, scripts and symlinks.
    Cat(cat::Args),
    /// List the targets the source state manages.
    Managed(managed::Args),
}

/// Run the parsed command line. Errors are returned as the message to print.
///
/// The umask is read here, once, while logis has a single thread (see `process_umask`).
pub fn run(cli: Cli) -> Result<(), String> {
    let globals = Globals {
        dry_run: cli.dry_run,
        force: cli.force,
        source_path: cli.source_path,
        umask: logis_core::system::process_umask(),
    };
    let dirs = dirs::Dirs::resolve(cli.source, cli.destination)?;
    match cli.command {
        Some(Command::Apply(args)) => apply::run(&dirs, globals, &args),
        Some(Command::Cat(args)) => cat::run(&dirs, globals.umask, &args),
        Some(Command::Managed(args)) => managed::run(&dirs, &args),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
