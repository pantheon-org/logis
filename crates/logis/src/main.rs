//! The `logis` command-line interface.

mod cmd;

use clap::Parser;

fn main() {
    let _cli = cmd::Cli::parse();
}
