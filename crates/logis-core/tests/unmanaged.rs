//! The `unmanaged` walk: chezmoi's `runUnmanagedCmd` (`internal/cmd/unmanagedcmd.go`), on an
//! in-memory filesystem. chezmoi has no Go table for it; the cases follow its txtar script and
//! a sandbox run.

use std::path::{Path, PathBuf};

use logis_core::entry_type::EntryTypeFilter;
use logis_core::path_tree::tree;
use logis_core::source_state::SourceState;
use logis_core::special::IGNORE;
use logis_core::system::MemSystem;
use logis_core::unmanaged::unmanaged;

const DEST: &str = "/home/user";
const SOURCE: &str = "/src";

fn dest(rel: &str) -> String {
    format!("{DEST}/{rel}")
}

fn system() -> MemSystem {
    MemSystem::new()
        .with_file(format!("{SOURCE}/dot_managed/known"), "x\n", 0o644)
        .with_file(
            format!("{SOURCE}/{IGNORE}"),
            ".ignored\n.ign\n.managed/extra\n",
            0o644,
        )
        .with_file(dest(".managed/known"), "x\n", 0o644)
        .with_file(dest(".managed/extra"), "x\n", 0o644)
        .with_file(dest(".managed/sub/f"), "x\n", 0o644)
        .with_file(dest(".dir/file"), "x\n", 0o644)
        .with_symlink(dest(".dirlink"), ".dir")
        .with_symlink(dest(".dangling"), "nowhere")
        .with_file(dest(".ignored"), "x\n", 0o644)
        .with_file(dest(".ign/x"), "x\n", 0o644)
}

fn run(roots: &[&str], filter: EntryTypeFilter) -> Result<Vec<String>, String> {
    let system = system();
    let state = SourceState::read(&system, Path::new(SOURCE), Path::new(DEST)).unwrap();
    let roots: Vec<PathBuf> = roots.iter().map(PathBuf::from).collect();
    let mut warnings = Vec::new();
    let found = unmanaged(&system, &state, Path::new(DEST), &roots, filter, &mut |w| {
        warnings.push(w);
    })
    .map_err(|e| e.to_string())?;
    assert!(warnings.is_empty(), "{warnings:?}");
    Ok(found)
}

fn all() -> Vec<String> {
    run(&[DEST], EntryTypeFilter::default()).unwrap()
}

#[test]
fn unmanaged_entries_are_listed_but_not_entered() {
    assert_eq!(all(), [".dangling", ".dir", ".dirlink", ".managed/sub"]);
}

#[test]
fn filters_select_by_destination_type() {
    let only = |types: &str| EntryTypeFilter {
        include: types.parse().unwrap(),
        exclude: "none".parse().unwrap(),
    };
    assert_eq!(
        run(&[DEST], only("symlinks")).unwrap(),
        [".dangling", ".dirlink"]
    );
    assert_eq!(
        run(&[DEST], only("dirs")).unwrap(),
        [".dir", ".managed/sub"]
    );
    assert!(run(&[DEST], only("files")).unwrap().is_empty());
}

#[test]
fn roots_inside_the_destination_are_walked_once_each() {
    let found = run(
        &[&dest(".dir"), &dest(".managed"), &dest(".dir")],
        EntryTypeFilter::default(),
    )
    .unwrap();
    assert_eq!(found, [".dir", ".managed/sub"]);
}

/// The ignore pattern `.ign` names the directory, not what is in it.
#[test]
fn a_root_inside_an_ignored_directory_is_listed() {
    assert_eq!(
        run(&[&dest(".ign/x")], EntryTypeFilter::default()).unwrap(),
        [".ign/x"]
    );
}

#[test]
fn a_missing_root_is_an_error() {
    let err = run(&[&dest("absent")], EntryTypeFilter::default()).unwrap_err();
    assert!(err.starts_with(&dest("absent")), "{err}");
}

#[test]
fn a_root_outside_the_destination_is_an_error() {
    assert_eq!(
        run(&["/etc"], EntryTypeFilter::default()).unwrap_err(),
        format!("/etc: not in {DEST}")
    );
}

#[test]
fn trees_indent_children_under_their_parents() {
    let paths = [
        ".dangling",
        ".managed/sub",
        ".dir",
        ".managed/sub/deep",
        "a/b",
    ];
    assert_eq!(
        tree(&paths),
        ".dangling\n.dir\n.managed\n  sub\n    deep\na\n  b\n"
    );
}
