//! The source-name grammar: how a name in the source state encodes a target name and its
//! attributes, for example `private_executable_dot_script.tmpl`.
//!
//! Ported from chezmoi's `internal/chezmoi/attr.go` (ADR-007). Directories and files have
//! different attribute sets: [`DirAttr`] and [`FileAttr`]. Parsing and
//! [`DirAttr::source_name`] / [`FileAttr::source_name`] round-trip for every valid target name.
//! Unlike chezmoi, names starting `remove_`, `external_` or `onchange_` are protected with
//! `literal_` when written, so they round-trip too.

mod dir;
mod error;
mod file;
mod prefix;

pub use dir::DirAttr;
pub use error::ParseError;
pub use file::{FileAttr, ScriptCondition, ScriptOrder, TargetType};
