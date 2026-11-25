use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::source_name::ParseError;

/// Why the source state could not be read. Messages match chezmoi's where it has one.
#[derive(Debug)]
pub enum ReadError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    NotADirectory(PathBuf),
    /// A source name that does not parse, for example `dot_`.
    InvalidName(ParseError),
    NonUtf8Name(PathBuf),
    UnsupportedFileType(PathBuf),
    /// Targets with more than one source entry: each target with its sorted source paths.
    Inconsistent(Vec<(String, Vec<String>)>),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::NotADirectory(path) => write!(f, "{}: not a directory", path.display()),
            Self::InvalidName(err) => err.fmt(f),
            Self::NonUtf8Name(path) => write!(f, "{}: not valid UTF-8", path.display()),
            Self::UnsupportedFileType(path) => {
                write!(f, "{}: unsupported file type", path.display())
            }
            Self::Inconsistent(targets) => {
                let lines: Vec<String> = targets
                    .iter()
                    .map(|(target, origins)| {
                        format!("{target}: inconsistent state ({})", origins.join(", "))
                    })
                    .collect();
                f.write_str(&lines.join("\n"))
            }
        }
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidName(err) => Some(err),
            _ => None,
        }
    }
}
