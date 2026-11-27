//! Step definitions in neutral domain vocabulary. Tool differences (binary, flags, config
//! format, special file names, source-state naming) live in the adapter and `source_names`,
//! never in feature files.
//!
//! Paths in features are target paths written as `~/...`, relative to the sandbox home.

mod adapter;
mod common;
mod contents;
mod home;
pub mod selection;
mod source;
pub mod source_names;

pub use adapter::Tool;

use std::path::{Path, PathBuf};
use std::process::Output;

use tempfile::TempDir;

use source_names::{Kind, decode, encode};

/// A sandbox per scenario: its own home, source and config directories.
#[derive(Debug, Default, cucumber::World)]
pub struct LogisWorld {
    pub sandbox: Option<TempDir>,
    pub last: Option<Output>,
}

impl LogisWorld {
    pub fn root(&self) -> &Path {
        self.sandbox
            .as_ref()
            .expect("scenario must start with a sandbox step")
            .path()
    }

    pub fn home(&self) -> PathBuf {
        self.root().join("home")
    }

    pub fn source_dir(&self) -> PathBuf {
        self.root().join("source")
    }

    /// `~/a/b` as an absolute path in the sandbox home.
    pub fn target(&self, path: &str) -> PathBuf {
        self.home().join(relative(path))
    }

    /// The source-state path for target `path` of `kind`. Parent directories already in the
    /// source state are matched by their decoded names (so `exact_` parents are found); missing
    /// parents get plain directory names.
    pub fn source_path(&self, path: &str, kind: Kind) -> PathBuf {
        let components: Vec<&str> = relative(path).split('/').collect();
        let (last, parents) = components.split_last().expect("non-empty target path");
        let mut dir = self.source_dir();
        for parent in parents {
            dir = existing_entry(&dir, parent)
                .unwrap_or_else(|| dir.join(encode(parent, Kind::Directory)));
        }
        dir.join(encode(last, kind))
    }

    /// The source entry, of any kind, that stands for target `path`.
    pub fn find_source(&self, path: &str) -> Option<PathBuf> {
        let mut dir = self.source_dir();
        for component in relative(path).split('/') {
            dir = existing_entry(&dir, component)?;
        }
        Some(dir)
    }

    pub fn last_output(&self) -> &Output {
        self.last.as_ref().expect("a command must have run")
    }

    /// Replace the placeholders in expected output: `{home}` and `{source}` are the sandbox
    /// directories, and `{scripts-dir}` and `{ignore-file}` are the tool's special names.
    pub fn expand_placeholders(&self, text: &str) -> String {
        let tool = Tool::from_env();
        text.replace("{home}", &self.home().display().to_string())
            .replace("{source}", &self.source_dir().display().to_string())
            .replace("{scripts-dir}", tool.scripts_dir_name())
            .replace("{ignore-file}", tool.ignore_file_name())
    }

    /// Replace `~/` with the sandbox home in a command-line argument.
    pub fn expand_arg(&self, arg: &str) -> String {
        if let Some(inner) = arg
            .strip_prefix("{source-of:")
            .and_then(|a| a.strip_suffix('}'))
        {
            return self.source_path(inner, Kind::File).display().to_string();
        }
        match arg.strip_prefix("~/") {
            Some(rest) => self.home().join(rest).display().to_string(),
            None if arg == "~" => self.home().display().to_string(),
            None => arg.to_owned(),
        }
    }
}

fn relative(path: &str) -> &str {
    path.strip_prefix("~/")
        .unwrap_or_else(|| panic!("target paths must start with ~/, got {path:?}"))
}

fn existing_entry(dir: &Path, target_name: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .find(|e| decode(&e.file_name().to_string_lossy()) == target_name)
        .map(|e| e.path())
}
