//! The `managed` query: the targets the source state manages, filtered and formatted.

use std::path::Path;
use std::str::FromStr;

use crate::entry_type::EntryTypeFilter;
use crate::source_state::SourceState;

/// How each managed entry is printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PathStyle {
    /// The target path relative to the destination directory.
    #[default]
    Relative,
    /// The absolute target path.
    Absolute,
    /// The source path relative to the source directory.
    SourceRelative,
    /// The absolute source path.
    SourceAbsolute,
}

impl FromStr for PathStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "relative" => Ok(Self::Relative),
            "absolute" => Ok(Self::Absolute),
            "source-relative" => Ok(Self::SourceRelative),
            "source-absolute" => Ok(Self::SourceAbsolute),
            other => Err(format!("{other}: unknown path style")),
        }
    }
}

/// What to list.
#[derive(Debug, Clone, Copy)]
pub struct Query<'a> {
    pub filter: EntryTypeFilter,
    /// Target paths relative to the destination directory. Empty means all targets; otherwise
    /// each selects itself and its descendants, and targets that are not managed are ignored.
    pub targets: &'a [&'a str],
    pub style: PathStyle,
    pub source_dir: &'a Path,
    pub dest_dir: &'a Path,
}

/// The matching entries, formatted in `query.style` and sorted.
#[must_use]
pub fn managed(state: &SourceState, query: &Query<'_>) -> Vec<String> {
    let mut lines: Vec<String> = state
        .entries()
        .filter(|(target, entry)| selected(target, query.targets) && query.filter.includes(entry))
        .map(|(target, entry)| match query.style {
            PathStyle::Relative => target.to_owned(),
            PathStyle::Absolute => query.dest_dir.join(target).display().to_string(),
            PathStyle::SourceRelative => entry.source_rel().to_owned(),
            PathStyle::SourceAbsolute => source_absolute(query.source_dir, entry.source_rel()),
        })
        .collect();
    lines.sort();
    lines
}

/// The source directory joined with `source_rel`, as chezmoi's `AbsPath.Join`: an empty
/// relative path (an absent entry's) is the source directory itself.
fn source_absolute(source_dir: &Path, source_rel: &str) -> String {
    if source_rel.is_empty() {
        source_dir.display().to_string()
    } else {
        source_dir.join(source_rel).display().to_string()
    }
}

/// Whether `target` is one of `targets` or below one. No targets, or an empty target (the
/// destination directory itself), selects everything.
fn selected(target: &str, targets: &[&str]) -> bool {
    targets.is_empty()
        || targets.iter().any(|t| {
            t.is_empty()
                || target == *t
                || target
                    .strip_prefix(*t)
                    .is_some_and(|rest| rest.starts_with('/'))
        })
}
