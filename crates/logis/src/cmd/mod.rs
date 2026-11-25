//! One module per command. Commands are added only when a BDD scenario for them is red.

mod dirs;
mod managed;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

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

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List the targets the source state manages.
    Managed(managed::Args),
}

/// Run the parsed command line. Errors are returned as the message to print.
pub fn run(cli: Cli) -> Result<(), String> {
    let dirs = dirs::Dirs::resolve(cli.source, cli.destination)?;
    match cli.command {
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
