//! The `logis` command-line interface.

mod cmd;

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    match cmd::run(cmd::Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("logis: {message}");
            ExitCode::FAILURE
        }
    }
}
