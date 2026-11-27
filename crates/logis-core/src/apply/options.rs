//! What `apply` and `status` are given. In its own file so they do not depend on `mod.rs`.

use std::path::Path;

use crate::entry_type::EntryTypeFilter;

#[derive(Debug, Clone, Copy)]
pub struct ApplyOptions<'a> {
    pub dest_dir: &'a Path,
    /// Taken from the target permissions, as chezmoi does.
    pub umask: u32,
    /// Entries of other types are skipped.
    pub filter: EntryTypeFilter,
}
