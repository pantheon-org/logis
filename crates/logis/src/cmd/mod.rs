//! One module per command. Commands are added only when a BDD scenario for them is red.

use clap::Parser;

/// Manage your dotfiles across machines.
#[derive(Debug, Parser)]
#[command(name = "logis", version, about)]
pub struct Cli {}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
