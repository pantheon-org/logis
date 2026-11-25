//! Step definitions in neutral domain vocabulary. Tool differences (binary, flags, config
//! format, special file names) live in the adapter, never in feature files.

mod adapter;
mod common;
pub mod selection;

pub use adapter::Tool;

use std::process::Output;

use tempfile::TempDir;

/// A sandbox per scenario: its own home, source and config directories.
#[derive(Debug, Default, cucumber::World)]
pub struct LogisWorld {
    pub sandbox: Option<TempDir>,
    pub last: Option<Output>,
}
