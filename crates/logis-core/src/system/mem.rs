use std::cell::RefCell;
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

type Tree = BTreeMap<PathBuf, Node>;

/// An in-memory filesystem for tests. Paths are absolute; missing parent directories are
/// created (mode 0o755) when an entry is added. There is no umask: `mkdir` uses `perm` as
/// given.
#[derive(Debug, Clone, Default)]
pub struct MemSystem {
    nodes: RefCell<Tree>,
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
        let nodes = self.nodes.get_mut();
        for ancestor in path.ancestors().skip(1) {
            nodes
                .entry(ancestor.to_owned())
                .or_insert(Node::Dir { mode: 0o755 });
        }
        nodes.insert(path.to_owned(), node);
        self
    }

    /// A copy of the node at `path`.
    fn node(&self, path: &Path, follow_last: bool) -> io::Result<Node> {
        let nodes = self.nodes.borrow();
        let real = resolve(&nodes, path, follow_last)?;
        nodes.get(&real).cloned().ok_or_else(|| not_found(path))
    }

    /// Insert `node` at `path` (not following a final symlink), unless a directory is there.
    fn replace(&self, path: &Path, node: Node) -> io::Result<()> {
        let mut nodes = self.nodes.borrow_mut();
        let real = entry_path(&nodes, path)?;
        if matches!(nodes.get(&real), Some(Node::Dir { .. })) {
            return Err(io::Error::new(
                io::ErrorKind::IsADirectory,
                format!("{}: is a directory", path.display()),
            ));
        }
        nodes.insert(real, node);
        Ok(())
    }
}

/// The real path of the entry `path` names: its parent resolved, which must be a directory,
/// and its own name.
fn entry_path(nodes: &Tree, path: &Path) -> io::Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::other(format!("{}: no file name", path.display())))?;
    let parent = resolve(nodes, path.parent().unwrap_or(Path::new("/")), true)?;
    match nodes.get(&parent) {
        Some(Node::Dir { .. }) => Ok(parent.join(name)),
        Some(_) => Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            format!("{}: not a directory", parent.display()),
        )),
        None => Err(not_found(&parent)),
    }
}

/// The real path of `path`: symlinks in parent components are always followed, the last
/// component only if `follow_last`.
fn resolve(nodes: &Tree, path: &Path, follow_last: bool) -> io::Result<PathBuf> {
    let mut pending: Vec<OsString> = components(path);
    pending.reverse();
    let mut current = PathBuf::from("/");
    let mut followed = 0;
    while let Some(name) = pending.pop() {
        let next = current.join(&name);
        let is_last = pending.is_empty();
        match nodes.get(&next) {
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
            Node::File { contents, .. } => Ok(contents),
            _ => Err(io::Error::other(format!("{}: not a file", path.display()))),
        }
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<OsString>> {
        let nodes = self.nodes.borrow();
        let real = resolve(&nodes, path, true)?;
        if !matches!(nodes.get(&real), Some(Node::Dir { .. })) {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                format!("{}: not a directory", path.display()),
            ));
        }
        // BTreeMap order gives children in sorted order.
        Ok(nodes
            .keys()
            .filter(|p| p.parent() == Some(real.as_path()))
            .filter_map(|p| p.file_name().map(ToOwned::to_owned))
            .collect())
    }

    fn lstat(&self, path: &Path) -> io::Result<Metadata> {
        self.node(path, false).map(|n| metadata(&n))
    }

    fn stat(&self, path: &Path) -> io::Result<Metadata> {
        self.node(path, true).map(|n| metadata(&n))
    }

    fn read_link(&self, path: &Path) -> io::Result<PathBuf> {
        match self.node(path, false)? {
            Node::Symlink { target } => Ok(target),
            _ => Err(io::Error::other(format!(
                "{}: not a symlink",
                path.display()
            ))),
        }
    }

    fn mkdir(&self, path: &Path, perm: u32) -> io::Result<()> {
        let mut nodes = self.nodes.borrow_mut();
        let real = entry_path(&nodes, path)?;
        if nodes.contains_key(&real) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("{}: already exists", path.display()),
            ));
        }
        nodes.insert(real, Node::Dir { mode: perm });
        Ok(())
    }

    fn chmod(&self, path: &Path, perm: u32) -> io::Result<()> {
        let mut nodes = self.nodes.borrow_mut();
        let real = resolve(&nodes, path, true)?;
        match nodes.get_mut(&real) {
            Some(Node::File { mode, .. } | Node::Dir { mode }) => {
                *mode = perm;
                Ok(())
            }
            Some(Node::Symlink { .. }) | None => Err(not_found(path)),
        }
    }

    fn write_file(&self, path: &Path, contents: &[u8], perm: u32) -> io::Result<()> {
        self.replace(
            path,
            Node::File {
                contents: contents.to_vec(),
                mode: perm,
            },
        )
    }

    fn write_symlink(&self, linkname: &Path, path: &Path) -> io::Result<()> {
        self.replace(
            path,
            Node::Symlink {
                target: linkname.to_owned(),
            },
        )
    }

    fn remove_all(&self, path: &Path) -> io::Result<()> {
        let mut nodes = self.nodes.borrow_mut();
        let real = match entry_path(&nodes, path) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
            other => other?,
        };
        nodes.retain(|p, _| !p.starts_with(&real));
        Ok(())
    }
}
