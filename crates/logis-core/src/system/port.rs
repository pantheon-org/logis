use std::io;
use std::path::Path;

/// Filesystem operations logis needs. Grows test-first, one method per failing scenario.
pub trait System: std::fmt::Debug {
    /// Read a file's full contents.
    fn read_file(&self, path: &Path) -> io::Result<Vec<u8>>;
}
