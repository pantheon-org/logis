use std::fmt;

/// Why a source name could not be parsed. Messages match chezmoi's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    EmptyDirName,
    EmptyFileName,
    /// The name decodes to an empty, `.` or `..` target name.
    InvalidPath(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDirName => f.write_str("empty directory name"),
            Self::EmptyFileName => f.write_str("empty filename"),
            Self::InvalidPath(name) => write!(f, "{name}: invalid path"),
        }
    }
}

impl std::error::Error for ParseError {}
