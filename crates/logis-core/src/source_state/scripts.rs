//! Reading the scripts directory, ported from chezmoi's `SourceState.readScriptsDir`. Every
//! file in it, at any depth, must be a script; its target path keeps the scripts directory.

use std::path::Path;

use super::entry::SourceEntry;
use super::error::ReadError;
use super::walk_util::{followed_metadata, io_error, join, utf8};
use crate::source_name::{DirAttr, FileAttr, TargetType};
use crate::special::PREFIX;
use crate::system::{FileType, System};

/// Add every script under `dir` to `found`. `source_rel` and `target` are the paths of `dir`
/// relative to the source and destination directories.
pub(super) fn walk(
    system: &dyn System,
    dir: &Path,
    source_rel: &str,
    target: &str,
    found: &mut Vec<(String, SourceEntry)>,
) -> Result<(), ReadError> {
    for name in system.read_dir(dir).map_err(io_error(dir))? {
        let path = dir.join(&name);
        let name = utf8(&name, &path)?;
        if name.starts_with(PREFIX) {
            return Err(ReadError::NotAllowedInScriptsDir(path));
        }
        if name.starts_with('.') {
            continue;
        }
        let source_rel = join(source_rel, name);
        match followed_metadata(system, &path)?.file_type {
            FileType::Dir => {
                let attr = DirAttr::parse(name).map_err(ReadError::InvalidName)?;
                walk(
                    system,
                    &path,
                    &source_rel,
                    &join(target, &attr.target_name),
                    found,
                )?;
            }
            FileType::File => {
                let attr = FileAttr::parse(name, "").map_err(ReadError::InvalidName)?;
                if attr.target_type != TargetType::Script {
                    return Err(ReadError::NotAScript(path));
                }
                found.push((
                    join(target, &attr.target_name),
                    SourceEntry::File {
                        source_path: path,
                        source_rel,
                        attr,
                    },
                ));
            }
            FileType::Symlink | FileType::Other => {
                return Err(ReadError::UnsupportedFileType(path));
            }
        }
    }
    Ok(())
}
