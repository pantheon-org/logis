//! What `status` reports for each target: chezmoi's `runStatusCmd` pre-apply function and
//! `statusRune` (`internal/cmd/statuscmd.go`), with the entry states of
//! `internal/chezmoi/entrystate.go`. chezmoi has no Go table for these. The first column is
//! always blank until logis records what it wrote (ADR-009).

use std::path::Path;

use logis_core::apply::{ApplyOptions, Status, status};
use logis_core::entry_type::EntryTypeFilter;
use logis_core::source_state::SourceState;
use logis_core::system::{MemSystem, System as _};

const DEST: &str = "/home/user";
const SOURCE: &str = "/home/user/.local/share/logis";

fn source(rel: &str) -> String {
    format!("{SOURCE}/{rel}")
}

fn dest(rel: &str) -> String {
    format!("{DEST}/{rel}")
}

/// `XY path` lines for every target.
fn lines(system: &MemSystem, filter: EntryTypeFilter) -> Vec<String> {
    let state = SourceState::read(system, Path::new(SOURCE), Path::new(DEST)).unwrap();
    let targets: Vec<&str> = state.entries().map(|(t, _)| t).collect();
    let options = ApplyOptions {
        dest_dir: Path::new(DEST),
        umask: 0o022,
        filter,
    };
    status(system, &state, &targets, &options)
        .unwrap()
        .iter()
        .map(|Status { target, x, y }| format!("{x}{y} {target}"))
        .collect()
}

fn all(system: &MemSystem) -> Vec<String> {
    lines(system, EntryTypeFilter::default())
}

#[test]
fn new_targets_are_added() {
    let system = MemSystem::new()
        .with_dir(source("dot_dir"), 0o755)
        .with_file(source("dot_dir/file"), "x\n", 0o644)
        .with_file(source("symlink_dot_link"), "target\n", 0o644);
    assert_eq!(all(&system), [" A .dir", " A .dir/file", " A .link"]);
}

#[test]
fn targets_that_match_are_not_listed() {
    let system = MemSystem::new()
        .with_dir(dest(".dir"), 0o755)
        .with_file(dest(".dir/file"), "x\n", 0o644)
        .with_symlink(dest(".link"), "target")
        .with_dir(source("dot_dir"), 0o755)
        .with_file(source("dot_dir/file"), "x\n", 0o644)
        .with_file(source("symlink_dot_link"), "target\n", 0o644);
    assert!(all(&system).is_empty());
}

#[test]
fn contents_mode_kind_and_link_changes_are_modified() {
    let system = MemSystem::new()
        .with_file(dest(".contents"), "old\n", 0o644)
        .with_file(dest(".mode"), "x\n", 0o644)
        .with_file(dest(".kind"), "x\n", 0o644)
        .with_symlink(dest(".link"), "old")
        .with_file(source("dot_contents"), "new\n", 0o644)
        .with_file(source("executable_dot_mode"), "x\n", 0o644)
        .with_dir(source("dot_kind"), 0o755)
        .with_file(source("symlink_dot_link"), "new\n", 0o644);
    assert_eq!(
        all(&system),
        [" M .contents", " M .kind", " M .link", " M .mode"]
    );
}

/// Setuid, setgid and sticky bits are not compared (`EntryState.Equal` uses `Mode.Perm()`).
#[test]
fn special_mode_bits_are_not_a_change() {
    let system = MemSystem::new()
        .with_file(dest(".file"), "x\n", 0o4644)
        .with_file(source("dot_file"), "x\n", 0o644);
    assert!(all(&system).is_empty());
}

#[test]
fn removals_are_deleted() {
    let system = MemSystem::new()
        .with_file(dest(".gone"), "x\n", 0o644)
        .with_file(dest(".empty"), "x\n", 0o644)
        .with_file(dest(".dir/extra"), "x\n", 0o644)
        .with_file(source("remove_dot_gone"), "", 0o644)
        .with_file(source("remove_dot_absent"), "", 0o644)
        .with_file(source("dot_empty"), " \n", 0o644)
        .with_file(source("dot_nothing"), "", 0o644)
        .with_dir(source("exact_dot_dir"), 0o755);
    assert_eq!(all(&system), [" D .dir/extra", " D .empty", " D .gone"]);
}

#[test]
fn an_existing_create_target_is_unchanged() {
    let system = MemSystem::new()
        .with_file(dest(".create"), "# other\n", 0o644)
        .with_file(source("create_dot_create"), "# source\n", 0o644);
    assert!(all(&system).is_empty());
}

/// `TargetStateScript.SkipApply` skips empty scripts; every other script would run.
#[test]
fn scripts_are_run_unless_empty() {
    let system = MemSystem::new()
        .with_file(source("run_script"), "#!/bin/sh\n", 0o644)
        .with_file(source("run_once_once"), "#!/bin/sh\n", 0o644)
        .with_file(source("run_empty"), "", 0o644);
    assert_eq!(all(&system), [" R once", " R script"]);
}

#[test]
fn the_filter_selects_entry_types() {
    let system = MemSystem::new()
        .with_file(dest(".gone"), "x\n", 0o644)
        .with_dir(source("dot_dir"), 0o755)
        .with_file(source("dot_file"), "x\n", 0o644)
        .with_file(source("remove_dot_gone"), "", 0o644)
        .with_file(source("run_script"), "#!/bin/sh\n", 0o644);
    let files = EntryTypeFilter {
        include: "files".parse().unwrap(),
        exclude: "none".parse().unwrap(),
    };
    assert_eq!(lines(&system, files), [" A .file"]);
}

#[test]
fn status_changes_nothing() {
    let system = MemSystem::new()
        .with_file(dest(".gone"), "x\n", 0o644)
        .with_file(source("remove_dot_gone"), "", 0o644)
        .with_file(source("dot_new"), "x\n", 0o644);
    all(&system);
    assert!(system.read_file(Path::new(&dest(".gone"))).is_ok());
    assert!(system.lstat(Path::new(&dest(".new"))).is_err());
}
