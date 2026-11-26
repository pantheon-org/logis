//! Where the source and destination directories are, and how target arguments map onto the
//! destination directory.

use std::path::{Component, Path, PathBuf};

/// The directories every command works on, as absolute, normalized paths.
#[derive(Debug, Clone)]
pub struct Dirs {
    pub source: PathBuf,
    pub destination: PathBuf,
    cwd: PathBuf,
}

impl Dirs {
    /// Flags win; otherwise the defaults. The default source directory follows the XDG base
    /// directory spec (proposed in DECISIONS.md, not yet an ADR).
    pub fn resolve(source: Option<PathBuf>, destination: Option<PathBuf>) -> Result<Self, String> {
        let cwd = std::env::current_dir().map_err(|e| format!("current directory: {e}"))?;
        let home = || {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .ok_or_else(|| "HOME is not set".to_owned())
        };
        let destination = match destination {
            Some(dir) => dir,
            None => home()?,
        };
        let source = match source {
            Some(dir) => dir,
            None => match std::env::var_os("XDG_DATA_HOME") {
                Some(data) => PathBuf::from(data).join("logis"),
                None => home()?.join(".local/share/logis"),
            },
        };
        Ok(Self {
            source: absolute(&cwd, &source),
            destination: absolute(&cwd, &destination),
            cwd,
        })
    }

    /// The target path of a command-line argument: `/`-separated and relative to the
    /// destination directory, empty for the destination directory itself.
    pub fn target(&self, arg: &Path) -> Result<String, String> {
        let path = absolute(&self.cwd, arg);
        let rel = path.strip_prefix(&self.destination).map_err(|_| {
            format!(
                "{}: not in destination directory ({})",
                path.display(),
                self.destination.display()
            )
        })?;
        Ok(rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/"))
    }
}

impl Dirs {
    /// The source path of a command-line argument, `/`-separated and relative to the source
    /// directory (for `--source-path`).
    pub fn source_rel(&self, arg: &Path) -> Result<String, String> {
        let path = absolute(&self.cwd, arg);
        let rel = path.strip_prefix(&self.source).map_err(|_| {
            format!(
                "{}: not in source directory ({})",
                path.display(),
                self.source.display()
            )
        })?;
        Ok(rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/"))
    }
}

/// `path` made absolute against `cwd`, with `.` and `..` resolved lexically.
fn absolute(cwd: &Path, path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in cwd.join(path).components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dirs() -> Dirs {
        Dirs {
            source: "/home/u/.local/share/logis".into(),
            destination: "/home/u".into(),
            cwd: "/home/u/work".into(),
        }
    }

    #[test]
    fn source_paths_are_relative_to_the_source_directory() {
        assert_eq!(
            dirs().source_rel(Path::new("/home/u/.local/share/logis/dot_dir/file")),
            Ok("dot_dir/file".into())
        );
        assert_eq!(
            dirs().source_rel(Path::new("/home/u/.file")),
            Err("/home/u/.file: not in source directory (/home/u/.local/share/logis)".into())
        );
    }

    #[test]
    fn targets_are_relative_to_the_destination() {
        assert_eq!(
            dirs().target(Path::new("/home/u/.dir/file")),
            Ok(".dir/file".into())
        );
        assert_eq!(dirs().target(Path::new("/home/u")), Ok(String::new()));
    }

    #[test]
    fn relative_arguments_resolve_against_the_working_directory() {
        assert_eq!(dirs().target(Path::new("../.file")), Ok(".file".into()));
        assert_eq!(dirs().target(Path::new("./x/../y")), Ok("work/y".into()));
    }

    #[test]
    fn paths_outside_the_destination_are_errors() {
        assert_eq!(
            dirs().target(Path::new("/etc/hosts")),
            Err("/etc/hosts: not in destination directory (/home/u)".into())
        );
    }
}
