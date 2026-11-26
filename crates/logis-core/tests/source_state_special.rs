//! Special files in the source directory (ADR-008): the ignore file and the scripts directory.
//! Ports of the matching `TestSourceStateRead` cases (`internal/chezmoi/sourcestate_test.go`)
//! keep the Go names; extra cases say where they come from. Out of scope for now:
//! `chezmoiignore_exact_dir` (needs exact-dir absents), the remove file, data, templates,
//! externals and the version file.

use std::path::Path;

use logis_core::source_state::{ReadError, SourceState};
use logis_core::special::{IGNORE, SCRIPTS_DIR};
use logis_core::system::MemSystem;

const SOURCE: &str = "/home/user/.local/share/logis";

fn source(rel: &str) -> String {
    format!("{SOURCE}/{rel}")
}

fn ignore_file(contents: &str) -> MemSystem {
    MemSystem::new().with_file(source(IGNORE), contents, 0o644)
}

fn read(system: &MemSystem) -> Result<SourceState, ReadError> {
    SourceState::read(system, Path::new(SOURCE))
}

fn targets(state: &SourceState) -> Vec<&str> {
    state.entries().map(|(target, _)| target).collect()
}

fn ignored(state: &SourceState) -> Vec<&str> {
    state.ignored().collect()
}

fn error(system: &MemSystem) -> String {
    read(system).unwrap_err().to_string()
}

#[test]
fn chezmoiignore() {
    let state = read(&ignore_file("README.md\n")).unwrap();
    assert!(state.is_empty());
    assert!(ignored(&state).is_empty());
}

#[test]
fn chezmoiignore_ignore_file() {
    let system = ignore_file("README.md\n").with_file(source("README.md"), "", 0o644);
    let state = read(&system).unwrap();
    assert!(state.is_empty());
    assert_eq!(ignored(&state), ["README.md"]);
}

#[test]
fn chezmoiignore_ignore_file_with_hash() {
    let system = ignore_file("README.md#\n").with_file(source("README.md#"), "", 0o644);
    let state = read(&system).unwrap();
    assert!(state.is_empty());
    assert_eq!(ignored(&state), ["README.md#"]);
}

/// Not a Go case: an ignored directory is not read, so its contents are neither managed nor
/// reported as ignored.
#[test]
fn ignored_dir_is_not_read() {
    let system = ignore_file(".dir\n")
        .with_file(source("dot_dir/file"), "", 0o644)
        .with_file(source("dot_file"), "", 0o644);
    let state = read(&system).unwrap();
    assert_eq!(targets(&state), [".file"]);
    assert_eq!(ignored(&state), [".dir"]);
}

/// Not a Go case (`addPatterns`): comments, blank lines and surrounding space are dropped, and
/// a trailing `/` is cleaned away.
#[test]
fn comments_blank_lines_and_space() {
    let system = ignore_file("# comment\n\n  .dir/subdir/   # trailing\n")
        .with_file(source("dot_dir/subdir/file"), "", 0o644)
        .with_file(source("dot_dir/file"), "", 0o644);
    let state = read(&system).unwrap();
    assert_eq!(targets(&state), [".dir", ".dir/file"]);
}

/// Not a Go case (`addPatterns`): `!` excludes a target from being ignored.
#[test]
fn excluded_pattern() {
    let system = ignore_file("*\n!keep\n")
        .with_file(source("keep"), "", 0o644)
        .with_file(source("drop"), "", 0o644);
    assert_eq!(targets(&read(&system).unwrap()), ["keep"]);
}

/// Not a Go case (`addPatterns`): patterns in a directory's ignore file are relative to that
/// directory's target.
#[test]
fn ignore_file_in_subdirectory() {
    let system = MemSystem::new()
        .with_file(source(&format!("dot_dir/{IGNORE}")), "file\n", 0o644)
        .with_file(source("dot_dir/file"), "", 0o644)
        .with_file(source("file"), "", 0o644);
    assert_eq!(targets(&read(&system).unwrap()), [".dir", "file"]);
}

/// Not a Go case (`Read`): targets are ignored before duplicates are checked.
#[test]
fn ignored_duplicates_are_not_inconsistent() {
    let system = ignore_file(".file\n")
        .with_file(source("dot_file"), "", 0o644)
        .with_file(source("executable_dot_file"), "", 0o755);
    assert!(read(&system).unwrap().is_empty());
}

/// Not a Go case (`Read`): an entry that sorts before the ignore file is still ignored.
#[test]
fn entries_before_the_ignore_file_are_ignored() {
    let system = ignore_file("-early\n").with_file(source("-early"), "", 0o644);
    let state = read(&system).unwrap();
    assert!(state.is_empty());
    assert_eq!(ignored(&state), ["-early"]);
}

#[test]
fn invalid_pattern_is_an_error() {
    let system = ignore_file("ok\n[abc\n");
    assert_eq!(
        error(&system),
        format!("{}:2: [abc: invalid pattern", source(IGNORE))
    );
}

#[test]
fn pattern_outside_the_destination_is_an_error() {
    for line in ["../x", "..", "a/../b", "a/..", "/abs"] {
        let system = ignore_file(&format!("{line}\n"));
        assert_eq!(
            error(&system),
            format!("{}:1: {line}: invalid path", source(IGNORE)),
            "{line}"
        );
    }
}

/// Not a Go case: chezmoi's own special files are hidden names to logis (ADR-008).
#[test]
fn chezmoi_special_files_are_hidden() {
    let system = MemSystem::new()
        .with_file(source(".chezmoiignore"), "file\n", 0o644)
        .with_file(source(".chezmoiscripts/dot_file"), "", 0o644)
        .with_file(source("file"), "", 0o644);
    assert_eq!(targets(&read(&system).unwrap()), ["file"]);
}

fn scripts(rel: &str) -> String {
    source(&format!("{SCRIPTS_DIR}/{rel}"))
}

/// Not a Go case (`readScriptsDir`): scripts keep the scripts directory in their target path.
#[test]
fn scripts_dir() {
    let system = MemSystem::new()
        .with_file(scripts("run_script.sh"), "", 0o644)
        .with_file(scripts("subdir/run_once_script.sh"), "", 0o644)
        .with_file(scripts(".hidden"), "", 0o644)
        .with_file(scripts(".hidden-dir/dot_file"), "", 0o644);
    let state = read(&system).unwrap();
    assert_eq!(
        targets(&state),
        [
            format!("{SCRIPTS_DIR}/script.sh"),
            format!("{SCRIPTS_DIR}/subdir/script.sh"),
        ]
    );
}

/// Not a Go case (`readScriptsDir`).
#[test]
fn scripts_dir_non_script_is_an_error() {
    let system = MemSystem::new().with_file(scripts("dot_file"), "", 0o644);
    assert_eq!(
        error(&system),
        format!("{}: not a script", scripts("dot_file"))
    );
}

/// Not a Go case (`readScriptsDir`).
#[test]
fn scripts_dir_special_file_is_an_error() {
    let system = MemSystem::new().with_file(scripts(IGNORE), "", 0o644);
    assert_eq!(
        error(&system),
        format!(
            "{}: not allowed in {SCRIPTS_DIR} directory",
            scripts(IGNORE)
        )
    );
}

/// Not a Go case (`readScriptsDir`).
#[test]
fn scripts_dir_duplicates_are_inconsistent() {
    let system = MemSystem::new()
        .with_file(scripts("run_script.sh"), "", 0o644)
        .with_file(scripts("run_once_script.sh"), "", 0o644);
    assert!(matches!(read(&system), Err(ReadError::Inconsistent(_))));
}

/// From `TestSourceStateReadScriptsConcurrent` `with_ignore`, with two scripts per directory.
#[test]
fn with_ignore() {
    let system = ignore_file(&format!("{SCRIPTS_DIR}/linux/**\n"))
        .with_file(scripts("linux/run_a"), "", 0o644)
        .with_file(scripts("linux/run_b"), "", 0o644)
        .with_file(scripts("darwin/run_a"), "", 0o644)
        .with_file(scripts("darwin/run_b"), "", 0o644);
    let state = read(&system).unwrap();
    assert_eq!(
        targets(&state),
        [
            format!("{SCRIPTS_DIR}/darwin/a"),
            format!("{SCRIPTS_DIR}/darwin/b"),
        ]
    );
}
