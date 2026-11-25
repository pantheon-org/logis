//! Map target names to source-state file names, so features never mention them.
//!
//! This is chezmoi's naming (`dot_`, `private_`, `executable_`, ...). logis keeps it for now
//! (proposed ADR: keep the source-name attributes); if that changes, only this file changes.
//! Unit-tested through `tests/bdd_support.rs`.

use std::str::FromStr;

/// What kind of entry the source state holds for a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    File,
    Executable,
    Private,
    Empty,
    Symlink,
    Create,
    Directory,
    ExactDirectory,
}

impl FromStr for Kind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "file" => Self::File,
            "executable" => Self::Executable,
            "private" => Self::Private,
            "empty" => Self::Empty,
            "symlink" => Self::Symlink,
            "create" => Self::Create,
            "directory" => Self::Directory,
            "exact" => Self::ExactDirectory,
            other => return Err(format!("unknown kind {other:?}")),
        })
    }
}

impl Kind {
    #[must_use]
    pub fn is_directory(self) -> bool {
        matches!(self, Self::Directory | Self::ExactDirectory)
    }

    fn prefix(self) -> &'static str {
        match self {
            Self::File | Self::Directory => "",
            Self::Executable => "executable_",
            Self::Private => "private_",
            Self::Empty => "empty_",
            Self::Symlink => "symlink_",
            Self::Create => "create_",
            Self::ExactDirectory => "exact_",
        }
    }
}

/// Attribute prefixes a source name may carry, in any order chezmoi accepts.
const PREFIXES: [&str; 10] = [
    "create_",
    "modify_",
    "remove_",
    "encrypted_",
    "private_",
    "readonly_",
    "empty_",
    "executable_",
    "symlink_",
    "exact_",
];

/// The source name for target name `name` (one path component) of `kind`.
#[must_use]
pub fn encode(name: &str, kind: Kind) -> String {
    let base = name
        .strip_prefix('.')
        .map_or_else(|| name.to_owned(), |rest| format!("dot_{rest}"));
    format!("{}{base}", kind.prefix())
}

/// The target name a source name stands for.
#[must_use]
pub fn decode(source_name: &str) -> String {
    let mut name = source_name;
    while let Some(rest) = PREFIXES.iter().find_map(|p| name.strip_prefix(p)) {
        name = rest;
    }
    let name = name.strip_suffix(".tmpl").unwrap_or(name);
    name.strip_prefix("dot_")
        .map_or_else(|| name.to_owned(), |rest| format!(".{rest}"))
}
