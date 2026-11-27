//! The changes apply would make, as a git diff: chezmoi's `diff` command.
//!
//! `system.rs` records each write of an apply as a file patch (`patch.rs`), from a line diff
//! (`lines.rs`), with git blob hashes (`sha1.rs`) and chezmoi's binary sniffing (`sniff.rs`).
//! Not supported yet: scripts (phase 5), colour, a pager and external diff tools.

mod lines;
mod patch;
mod sha1;
mod sniff;
mod system;

pub use lines::{Edit, line_edits};
pub use patch::{Side, file_patch};
pub use sha1::git_blob_hash;
pub use sniff::is_binary;
pub use system::GitDiffSystem;
