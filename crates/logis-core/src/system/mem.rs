use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io;
use std::path::{Component, Path, PathBuf};

use super::port::{FileType, Metadata, System};

/// Symlinks followed before giving up, as Linux does (`ELOOP`).
const MAX_SYMLINKS: usize = 40;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    File { contents: Vec<u8>, mode: u32 },
    Dir { mode: u32 },
    Symlink { target: PathBuf },
}

/// An in-memory filesystem for tests. Paths are absolute; missing parent directories are
/// created (mode 0o755) when an entry is added.
#[derive(Debug, Clone, Default)]
pub struct MemSystem {
    nodes: BTreeMap<PathBuf, Node>,
}

impl MemSystem {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_dir(self, path: impl AsRef<Path>, mode: u32) -> Self {
        self.insert(path.as_ref(), Node::Dir { mode })
    }

    #[must_use]
    pub fn with_file(
        self,
        path: impl AsRef<Path>,
        contents: impl Into<Vec<u8>>,
        mode: u32,
    ) -> Self {
        let contents = contents.into();
        self.insert(path.as_ref(), Node::File { contents, mode })
    }

    #[must_use]
    pub fn with_symlink(self, path: impl AsRef<Path>, target: impl AsRef<Path>) -> Self {
        let target = target.as_ref().to_owned();
        self.insert(path.as_ref(), Node::Symlink { target })
    }

    fn insert(mut self, path: &Path, node: Node) -> Self {
        assert!(
            path.is_absolute(),
            "MemSystem paths are absolute: {}",
            path.display()
        );
        for ancestor in path.ancestors().skip(1) {
            self.nodes
                .entry(ancestor.to_owned())
                .or_insert(Node::Dir { mode: 0o755 });
        }
        self.nodes.insert(path.to_owned(), node);
        self
    }

    /// The real path of `path`: symlinks in parent components are always followed, the last
    /// component only if `follow_last`.
    fn resolve(&self, path: &Path, follow_last: bool) -> io::Result<PathBuf> {
        let mut pending: Vec<OsString> = components(path);
        pending.reverse();
        let mut current = PathBuf::from("/");
        let mut followed = 0;
        while let Some(name) = pending.pop() {
            let next = current.join(&name);
            let is_last = pending.is_empty();
            match self.nodes.get(&next) {
                Some(Node::Symlink { target }) if !is_last || follow_last => {
                    followed += 1;
                    if followed > MAX_SYMLINKS {
                        return Err(io::Error::other("too many levels of symbolic links"));
                    }
                    let target = current.join(target);
                    pending.extend(components(&target).into_iter().rev());
                    current = PathBuf::from("/");
                }
                Some(_) => current = next,
                None => return Err(not_found(&next)),
            }
        }
        Ok(current)
    }

    fn node(&self, path: &Path, follow_last: bool) -> io::Result<&Node> {
        let real = self.resolve(path, follow_last)?;
        self.nodes.get(&real).ok_or_else(|| not_found(path))
    }
}

/// The normal components of an absolute path, with `.` removed and `..` applied.
fn components(path: &Path) -> Vec<OsString> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(name) => parts.push(name.to_owned()),
            Component::ParentDir => {
                parts.pop();
            }
            Component::RootDir | Component::CurDir | Component::Prefix(_) => {}
        }
    }
    parts
}

fn not_found(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        format!("{}: not found", path.display()),
    )
}

fn metadata(node: &Node) -> Metadata {
    match node {
        Node::File { mode, .. } => Metadata {
            file_type: FileType::File,
            mode: *mode,
        },
        Node::Dir { mode } => Metadata {
            file_type: FileType::Dir,
            mode: *mode,
        },
        Node::Symlink { .. } => Metadata {
            file_type: FileType::Symlink,
            mode: 0o777,
        },
    }
}

impl System for MemSystem {
    fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        match self.node(path, true)? {
            Node::File { contents, .. } => Ok(contents.clone()),
            _ => Err(io::Error::other(format!("{}: not a file", path.display()))),
        }
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<OsString>> {
        let real = self.resolve(path, true)?;
        if !matches!(self.nodes.get(&real), Some(Node::Dir { .. })) {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                format!("{}: not a directory", path.display()),
            ));
        }
        // BTreeMap order gives children in sorted order.
        Ok(self
            .nodes
            .keys()
            .filter(|p| p.parent() == Some(real.as_path()))
            .filter_map(|p| p.file_name().map(ToOwned::to_owned))
            .collect())
    }

    fn lstat(&self, path: &Path) -> io::Result<Metadata> {
        self.node(path, false).map(metadata)
    }

    fn stat(&self, path: &Path) -> io::Result<Metadata> {
        self.node(path, true).map(metadata)
    }

    fn read_link(&self, path: &Path) -> io::Result<PathBuf> {
        match self.node(path, false)? {
            Node::Symlink { target } => Ok(target.clone()),
            _ => Err(io::Error::other(format!(
                "{}: not a symlink",
                path.display()
            ))),
        }
    }
}
