use std::io;
use std::path::Path;

use super::port::System;

/// The real machine. The only place allowed to use `std::fs` directly.
#[derive(Debug, Default, Clone, Copy)]
pub struct RealSystem;

impl System for RealSystem {
    fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_an_existing_file() {
        let dir = std::env::temp_dir().join(format!("logis-real-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("f");
        std::fs::write(&file, b"contents").unwrap();

        assert_eq!(RealSystem.read_file(&file).unwrap(), b"contents");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_file_is_not_found() {
        let err = RealSystem
            .read_file(Path::new("/nonexistent/logis/file"))
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}
