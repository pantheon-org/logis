//! Which scenarios run against which tool on which OS.
//!
//! Unit-tested through `tests/bdd_selection.rs`, because the cucumber runner (`tests/bdd.rs`)
//! uses its own harness and cannot run `#[test]`s.

/// The tool the BDD features run against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Chezmoi,
    Logis,
}

/// The operating system the run happens on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Mac,
    Linux,
    Windows,
}

impl Os {
    #[must_use]
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Mac
        } else if cfg!(windows) {
            Self::Windows
        } else {
            Self::Linux
        }
    }

    fn matches(self, tag: &str) -> bool {
        match tag {
            "unix" => self != Self::Windows,
            "windows" => self == Self::Windows,
            "macos" => self == Self::Mac,
            "linux" => self == Self::Linux,
            _ => true,
        }
    }
}

const OS_TAGS: [&str; 4] = ["unix", "windows", "macos", "linux"];

/// Whether a scenario with `tags` (feature, rule and scenario tags combined) runs.
#[must_use]
pub fn selected(tool: Tool, os: Os, tags: &[&str]) -> bool {
    let has = |t: &str| tags.contains(&t);
    if has("excluded") || has("skeleton") {
        return false;
    }
    let os_ok = tags
        .iter()
        .filter(|t| OS_TAGS.contains(t))
        .all(|t| os.matches(t));
    if !os_ok {
        return false;
    }
    match tool {
        Tool::Chezmoi => !has("logis-only"),
        Tool::Logis => !has("chezmoi-only") && !has("wip"),
    }
}
