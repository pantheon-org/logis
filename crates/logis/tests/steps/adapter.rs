//! How each tool is invoked inside the sandbox.

use std::path::{Path, PathBuf};
use std::process::Command;

pub use super::selection::Tool;

impl Tool {
    pub fn from_env() -> Self {
        match std::env::var("BDD_TOOL").as_deref() {
            Ok("chezmoi") => Self::Chezmoi,
            Ok("logis") | Err(_) => Self::Logis,
            Ok(other) => panic!("BDD_TOOL must be chezmoi or logis, got {other:?}"),
        }
    }

    /// The tool's own name, as it appears in its output.
    pub fn name(self) -> &'static str {
        match self {
            Self::Chezmoi => "chezmoi",
            Self::Logis => "logis",
        }
    }

    fn binary(self) -> PathBuf {
        match self {
            // The pinned oracle from mise.toml, never whatever chezmoi is first on PATH.
            Self::Chezmoi => {
                let out = Command::new("mise")
                    .args(["which", "chezmoi"])
                    .output()
                    .expect("mise must be installed to locate the pinned chezmoi");
                assert!(out.status.success(), "mise which chezmoi failed");
                PathBuf::from(String::from_utf8(out.stdout).unwrap().trim())
            }
            Self::Logis => PathBuf::from(env!("CARGO_BIN_EXE_logis")),
        }
    }

    /// Build a command that runs the tool against the sandbox only.
    pub fn command(self, sandbox: &Path, args: &[&str]) -> Command {
        let home = sandbox.join("home");
        let mut cmd = Command::new(self.binary());
        cmd.env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", &home)
            .env("XDG_CONFIG_HOME", sandbox.join("config"))
            .env("XDG_DATA_HOME", sandbox.join("data"))
            .env("XDG_CACHE_HOME", sandbox.join("cache"))
            .env("XDG_STATE_HOME", sandbox.join("state"))
            .env("TMPDIR", sandbox.join("tmp"))
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .current_dir(&home);
        if self == Self::Chezmoi {
            cmd.arg("--source")
                .arg(sandbox.join("source"))
                .arg("--destination")
                .arg(&home)
                .arg("--config")
                .arg(sandbox.join("config/chezmoi/chezmoi.toml"))
                .arg("--no-tty");
        }
        cmd.args(args);
        cmd
    }
}
