//! Names of the special files in the source directory (ADR-008): chezmoi's `.chezmoi*` names
//! with the prefix `.logis`.

/// Every special file name starts with this.
pub const PREFIX: &str = ".logis";

/// Patterns of targets that the source state does not manage.
pub const IGNORE: &str = ".logisignore";

/// A directory of scripts that are not tied to a target directory.
pub const SCRIPTS_DIR: &str = ".logisscripts";
