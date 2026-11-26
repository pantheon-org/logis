//! Settings every command may use. In its own file so that commands do not depend on
//! `cmd/mod.rs`, which depends on them.

#[derive(Debug, Clone, Copy)]
pub struct Globals {
    pub dry_run: bool,
    pub force: bool,
    pub source_path: bool,
    /// The process umask, read once at start-up.
    pub umask: u32,
}
