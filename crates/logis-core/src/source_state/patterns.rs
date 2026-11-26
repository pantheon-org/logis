//! The lines of an ignore (or remove) file, ported from chezmoi's `SourceState.addPatterns`.

use crate::pattern_set::{Include, PatternSet};

/// Why a line of a pattern file was rejected: its 1-based number and chezmoi's message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LineError {
    pub line: usize,
    pub message: String,
}

/// Add every pattern in `data` to `set`. Patterns are relative to `dir`, the target path of
/// the directory holding the file (empty at the top). A `#` at the start of a line or after
/// whitespace starts a comment; a leading `!` makes an exclude pattern.
pub(super) fn add_patterns(set: &mut PatternSet, data: &str, dir: &str) -> Result<(), LineError> {
    for (index, line) in data.lines().enumerate() {
        let line = strip_comment(line).trim();
        if line.is_empty() {
            continue;
        }
        let (include, line) = match line.strip_prefix('!') {
            Some(rest) => (Include::Exclude, rest),
            None => (Include::Include, line),
        };
        let error = |message: String| LineError {
            line: index + 1,
            message,
        };
        if !is_untrusted_rel_path_ok(line) {
            return Err(error(format!("{line}: invalid path")));
        }
        let pattern = clean(&format!("{dir}/{line}"));
        set.add(&pattern, include)
            .map_err(|e| error(e.to_string()))?;
    }
    Ok(())
}

/// `line` up to its comment: chezmoi's `(?:\A|\s+)#.*$`, so `a#b` has no comment.
fn strip_comment(line: &str) -> &str {
    let mut previous_is_space = true;
    for (index, c) in line.char_indices() {
        if c == '#' && previous_is_space {
            return &line[..index];
        }
        previous_is_space = c.is_whitespace();
    }
    line
}

/// chezmoi's `NewUntrustedRelPath`: a non-empty relative path that does not leave its
/// directory.
fn is_untrusted_rel_path_ok(path: &str) -> bool {
    !(path.is_empty()
        || path == ".."
        || path.starts_with("../")
        || path.contains("/../")
        || path.ends_with("/..")
        || path.starts_with('/'))
}

/// Go's `path.Clean` for a path that `is_untrusted_rel_path_ok` accepted, with any leading `/`
/// from joining an empty directory removed.
fn clean(path: &str) -> String {
    let mut components: Vec<&str> = Vec::new();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            other => components.push(other),
        }
    }
    if components.is_empty() {
        ".".to_owned()
    } else {
        components.join("/")
    }
}
