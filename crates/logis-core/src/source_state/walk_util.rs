//! Helpers shared by the source-directory walk and the scripts-directory walk.

use std::ffi::OsStr;
use std::path::Path;

use super::error::ReadError;
use crate::system::{FileType, Metadata, System};

/// Metadata of `path`, following a symlink: symlinks in the source directory stand for what
/// they point to.
pub(super) fn followed_metadata(system: &dyn System, path: &Path) -> Result<Metadata, ReadError> {
    let meta = system.lstat(path).map_err(io_error(path))?;
    if meta.file_type == FileType::Symlink {
        return system.stat(path).map_err(io_error(path));
    }
    Ok(meta)
}

pub(super) fn join(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_owned()
    } else {
        format!("{parent}/{name}")
    }
}

pub(super) fn utf8<'a>(name: &'a OsStr, path: &Path) -> Result<&'a str, ReadError> {
    name.to_str()
        .ok_or_else(|| ReadError::NonUtf8Name(path.to_owned()))
}

pub(super) fn io_error(path: &Path) -> impl FnOnce(std::io::Error) -> ReadError + '_ {
    move |source| ReadError::Io {
        path: path.to_owned(),
        source,
    }
}
