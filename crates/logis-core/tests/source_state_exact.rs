//! Absent entries for exact directories: every entry in the destination under an exact
//! directory that the source state does not manage. Ports of the matching
//! `TestSourceStateRead` cases (`internal/chezmoi/sourcestate_test.go`) keep the Go names;
//! extra cases come from the exact-directory part of `SourceState.Read`.

use std::path::{Path, PathBuf};

use logis_core::source_state::{ReadError, SourceEntry, SourceState};
use logis_core::special::IGNORE;
use logis_core::system::MemSystem;

const DEST: &str = "/home/user";
const SOURCE: &str = "/home/user/.local/share/logis";

fn source(rel: &str) -> String {
    format!("{SOURCE}/{rel}")
}

fn dest(rel: &str) -> String {
    format!("{DEST}/{rel}")
}

fn read(system: &MemSystem) -> Result<SourceState, ReadError> {
    SourceState::read(system, Path::new(SOURCE), Path::new(DEST))
}

/// Each target with `absent` for absent entries and its source path otherwise.
fn summary(state: &SourceState) -> Vec<(&str, String)> {
    state
        .entries()
        .map(|(target, entry)| match entry {
            SourceEntry::Absent { .. } => (target, "absent".to_owned()),
            _ => (target, entry.source_rel().to_owned()),
        })
        .collect()
}

#[test]
fn chezmoiignore_exact_dir() {
    let system = MemSystem::new()
        .with_file(dest("dir/file1"), "# contents of dir/file1\n", 0o644)
        .with_file(dest("dir/file2"), "# contents of dir/file2\n", 0o644)
        .with_file(dest("dir/file3"), "# contents of dir/file3\n", 0o644)
        .with_file(source(IGNORE), "dir/file3\n", 0o644)
        .with_file(
            source("exact_dir/file1"),
            "# contents of dir/file1\n",
            0o644,
        );
    let state = read(&system).unwrap();
    assert_eq!(
        summary(&state),
        [
            ("dir", "exact_dir".to_owned()),
            ("dir/file1", "exact_dir/file1".to_owned()),
            ("dir/file2", "absent".to_owned()),
        ]
    );
    assert_eq!(state.ignored().collect::<Vec<_>>(), ["dir/file3"]);
}

/// The absent entry's origin is the exact directory; its source path is empty, as chezmoi's
/// `SourceStateAbsent.SourceRelPath` is.
#[test]
fn absent_origin_is_the_exact_dir() {
    let system = MemSystem::new()
        .with_file(dest("dir/extra"), "", 0o644)
        .with_dir(source("exact_dir"), 0o755);
    let state = read(&system).unwrap();
    let entry = state.get("dir/extra").unwrap();
    assert_eq!(
        entry,
        &SourceEntry::Absent {
            exact_dir: PathBuf::from(source("exact_dir")),
        }
    );
    assert_eq!(entry.source_rel(), "");
}

/// An unmanaged directory in the destination is one absent entry; its contents are not read.
#[test]
fn absent_dir_is_one_entry() {
    let system = MemSystem::new()
        .with_file(dest("dir/sub/deep/file"), "", 0o644)
        .with_dir(source("exact_dir"), 0o755);
    let state = read(&system).unwrap();
    assert_eq!(
        summary(&state),
        [
            ("dir", "exact_dir".to_owned()),
            ("dir/sub", "absent".to_owned())
        ]
    );
}

/// Destination names are not filtered: hidden names and dangling symlinks are absent too.
#[test]
fn hidden_names_and_symlinks_are_absent() {
    let system = MemSystem::new()
        .with_file(dest("dir/.hidden"), "", 0o644)
        .with_symlink(dest("dir/link"), "nowhere")
        .with_dir(source("exact_dir"), 0o755);
    let state = read(&system).unwrap();
    assert_eq!(
        summary(&state),
        [
            ("dir", "exact_dir".to_owned()),
            ("dir/.hidden", "absent".to_owned()),
            ("dir/link", "absent".to_owned()),
        ]
    );
}

/// Nested exact directories each add the entries they do not manage.
#[test]
fn nested_exact_dirs() {
    let system = MemSystem::new()
        .with_file(dest("dir/sub/gone"), "", 0o644)
        .with_file(dest("dir/gone"), "", 0o644)
        .with_dir(source("exact_dir/exact_sub"), 0o755);
    let state = read(&system).unwrap();
    assert_eq!(
        summary(&state),
        [
            ("dir", "exact_dir".to_owned()),
            ("dir/gone", "absent".to_owned()),
            ("dir/sub", "exact_dir/exact_sub".to_owned()),
            ("dir/sub/gone", "absent".to_owned()),
        ]
    );
}

/// An unmanaged entry under a directory that is not exact is left alone.
#[test]
fn dir_that_is_not_exact_adds_nothing() {
    let system = MemSystem::new()
        .with_file(dest("dir/extra"), "", 0o644)
        .with_dir(source("dir"), 0o755);
    let state = read(&system).unwrap();
    assert_eq!(summary(&state), [("dir", "dir".to_owned())]);
}

/// A missing destination directory, or a file where the directory would be, adds nothing.
#[test]
fn missing_or_file_destination_adds_nothing() {
    let missing = MemSystem::new().with_dir(source("exact_dir"), 0o755);
    let file = MemSystem::new()
        .with_file(dest("dir"), "", 0o644)
        .with_dir(source("exact_dir"), 0o755);
    for system in [missing, file] {
        let state = read(&system).unwrap();
        assert_eq!(summary(&state), [("dir", "exact_dir".to_owned())]);
    }
}

/// An ignored exact directory adds nothing, and neither do its ignored entries.
#[test]
fn ignored_exact_dir_adds_nothing() {
    let system = MemSystem::new()
        .with_file(dest("dir/extra"), "", 0o644)
        .with_file(source(IGNORE), "dir\n", 0o644)
        .with_dir(source("exact_dir"), 0o755);
    let state = read(&system).unwrap();
    assert!(state.is_empty());
}
