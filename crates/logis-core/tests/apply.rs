//! Port of chezmoi's `TestSourceStateApplyAll` (`internal/chezmoi/sourcestate_test.go`), on
//! an in-memory filesystem. Case names are the Go names; `file_template` and
//! `symlink_template` wait for templates (phase 4). Extra cases follow `TargetState*.Apply`
//! (`internal/chezmoi/targetstateentry.go`) and say so.

use std::path::Path;

use logis_core::apply::{ApplyOptions, apply};
use logis_core::entry_type::EntryTypeFilter;
use logis_core::source_state::SourceState;
use logis_core::system::{DryRunSystem, FileType, MemSystem, System};

const DEST: &str = "/home/user";
const SOURCE: &str = "/home/user/.local/share/logis";
const UMASK: u32 = 0o022;

fn source(rel: &str) -> String {
    format!("{SOURCE}/{rel}")
}

fn dest(rel: &str) -> String {
    format!("{DEST}/{rel}")
}

fn options() -> ApplyOptions<'static> {
    ApplyOptions {
        dest_dir: Path::new(DEST),
        umask: UMASK,
        filter: EntryTypeFilter::default(),
    }
}

/// Apply every target of the source state in `system`.
fn apply_all(system: &dyn System) {
    let state = SourceState::read(system, Path::new(SOURCE), Path::new(DEST)).unwrap();
    let targets: Vec<&str> = state.entries().map(|(target, _)| target).collect();
    apply(system, &state, &targets, &options()).unwrap();
}

fn file_type(system: &dyn System, rel: &str) -> Option<FileType> {
    system
        .lstat(Path::new(&dest(rel)))
        .ok()
        .map(|m| m.file_type)
}

fn assert_file(system: &dyn System, rel: &str, contents: &str, mode: u32) {
    let path = dest(rel);
    let meta = system.lstat(Path::new(&path)).unwrap();
    assert_eq!((meta.file_type, meta.mode), (FileType::File, mode), "{rel}");
    assert_eq!(
        system.read_file(Path::new(&path)).unwrap(),
        contents.as_bytes(),
        "{rel}"
    );
}

fn assert_dir(system: &dyn System, rel: &str, mode: u32) {
    let meta = system.lstat(Path::new(&dest(rel))).unwrap();
    assert_eq!((meta.file_type, meta.mode), (FileType::Dir, mode), "{rel}");
}

#[test]
fn empty() {
    let system = MemSystem::new().with_dir(SOURCE, 0o777);
    apply_all(&system);
    assert_eq!(system.read_dir(Path::new(DEST)).unwrap(), [".local"]);
}

#[test]
fn dir() {
    let system = MemSystem::new().with_dir(source("dot_dir"), 0o777);
    apply_all(&system);
    assert_dir(&system, ".dir", 0o755);
}

#[test]
fn dir_exact() {
    let system = MemSystem::new()
        .with_file(dest(".dir/file"), "# contents of .dir/file\n", 0o644)
        .with_dir(source("exact_dot_dir"), 0o777);
    apply_all(&system);
    assert_dir(&system, ".dir", 0o755);
    assert_eq!(file_type(&system, ".dir/file"), None);
}

#[test]
fn file() {
    let system = MemSystem::new().with_file(source("dot_file"), "# contents of .file\n", 0o644);
    apply_all(&system);
    assert_file(&system, ".file", "# contents of .file\n", 0o644);
}

#[test]
fn file_remove_empty() {
    let system = MemSystem::new()
        .with_file(dest(".empty"), "# contents of .empty\n", 0o644)
        .with_file(source("dot_empty"), "", 0o644);
    apply_all(&system);
    assert_eq!(file_type(&system, ".empty"), None);
}

#[test]
fn file_create_empty() {
    let system = MemSystem::new().with_file(source("empty_dot_empty"), "", 0o644);
    apply_all(&system);
    assert_file(&system, ".empty", "", 0o644);
}

#[test]
fn create() {
    let system = MemSystem::new().with_file(
        source("create_dot_create"),
        "# contents of .create\n",
        0o644,
    );
    apply_all(&system);
    assert_file(&system, ".create", "# contents of .create\n", 0o644);
}

#[test]
fn create_no_replace() {
    let system = MemSystem::new()
        .with_file(
            source("create_dot_create"),
            "# contents of .create\n",
            0o644,
        )
        .with_file(dest(".create"), "# existing contents of .create\n", 0o644);
    apply_all(&system);
    assert_file(
        &system,
        ".create",
        "# existing contents of .create\n",
        0o644,
    );
}

#[test]
fn symlink() {
    let system =
        MemSystem::new().with_file(source("symlink_dot_symlink"), ".dir/subdir/file\n", 0o644);
    apply_all(&system);
    assert_eq!(file_type(&system, ".symlink"), Some(FileType::Symlink));
    assert_eq!(
        system.read_link(Path::new(&dest(".symlink"))).unwrap(),
        Path::new(".dir/subdir/file")
    );
}

/// Not a Go case (`TargetStateFile.Apply`): only whitespace counts as empty.
#[test]
fn file_of_only_whitespace_is_removed() {
    let system = MemSystem::new()
        .with_file(dest(".file"), "# old\n", 0o644)
        .with_file(source("dot_file"), " \n\t\n", 0o644);
    apply_all(&system);
    assert_eq!(file_type(&system, ".file"), None);
}

/// Not a Go case (`TargetStateFile.Apply`): same contents, other mode, is a chmod.
#[test]
fn file_with_the_right_contents_gets_its_mode() {
    let system = MemSystem::new()
        .with_file(dest(".file"), "#!\n", 0o644)
        .with_file(source("executable_dot_file"), "#!\n", 0o644);
    apply_all(&system);
    assert_file(&system, ".file", "#!\n", 0o755);
}

/// Not a Go case (`TargetStateFile.Apply`): a directory in the way is removed.
#[test]
fn file_replaces_a_directory() {
    let system = MemSystem::new()
        .with_file(dest(".file/sub"), "", 0o644)
        .with_file(source("dot_file"), "# file\n", 0o644);
    apply_all(&system);
    assert_file(&system, ".file", "# file\n", 0o644);
}

/// Not a Go case (`TargetStateFile.Apply` compares `Mode().Perm()`): a setgid bit is not a
/// difference.
#[test]
fn special_mode_bits_are_not_a_difference() {
    let system = MemSystem::new()
        .with_file(dest(".file"), "# same\n", 0o2644)
        .with_file(source("dot_file"), "# same\n", 0o644);
    apply_all(&system);
    assert_eq!(
        system.lstat(Path::new(&dest(".file"))).unwrap().mode,
        0o2644
    );
}

/// Not a Go case (`TargetStateDir.Apply`): a file in the way is removed, and a directory
/// with another mode gets its mode.
#[test]
fn dir_replaces_a_file_and_fixes_modes() {
    let system = MemSystem::new()
        .with_file(dest(".dir"), "", 0o644)
        .with_dir(dest(".private"), 0o755)
        .with_dir(source("dot_dir"), 0o755)
        .with_dir(source("private_dot_private"), 0o755);
    apply_all(&system);
    assert_dir(&system, ".dir", 0o755);
    assert_dir(&system, ".private", 0o700);
}

/// Not a Go case (`TargetStateSymlink.Apply`): a symlink to elsewhere is replaced; one to the
/// right place is left alone.
#[test]
fn symlink_is_retargeted() {
    let system = MemSystem::new()
        .with_symlink(dest(".old"), "elsewhere")
        .with_symlink(dest(".same"), "target")
        .with_file(source("symlink_dot_old"), "target\n", 0o644)
        .with_file(source("symlink_dot_same"), "target\n", 0o644);
    apply_all(&system);
    for name in [".old", ".same"] {
        assert_eq!(
            system.read_link(Path::new(&dest(name))).unwrap(),
            Path::new("target"),
            "{name}"
        );
    }
}

/// Not a Go case (`TargetStateAbsent.Apply`): remove targets go, whatever they are.
#[test]
fn remove_target_is_removed() {
    let system = MemSystem::new()
        .with_file(dest(".gone/file"), "", 0o644)
        .with_file(source("remove_dot_gone"), "", 0o644);
    apply_all(&system);
    assert_eq!(file_type(&system, ".gone"), None);
}

/// Not a Go case: scripts run in phase 5; until then applying one is an error.
#[test]
fn scripts_are_not_supported_yet() {
    let system = MemSystem::new().with_file(source("run_script"), "#!/bin/sh\n", 0o644);
    let state = SourceState::read(&system, Path::new(SOURCE), Path::new(DEST)).unwrap();
    let err = apply(&system, &state, &["script"], &options()).unwrap_err();
    assert_eq!(err.to_string(), "script: scripts are not supported yet");
}

/// Not a Go case: a dry run reads the real state but changes nothing.
#[test]
fn dry_run_changes_nothing() {
    let system = MemSystem::new()
        .with_file(dest(".file"), "# old\n", 0o644)
        .with_file(dest(".dir/extra"), "", 0o644)
        .with_file(source("dot_file"), "# new\n", 0o755)
        .with_dir(source("exact_dot_dir"), 0o755)
        .with_file(source("dot_new"), "# new\n", 0o644);
    let before = system.clone();
    apply_all(&DryRunSystem::new(&system));
    for rel in [".file", ".dir", ".dir/extra", ".new"] {
        let path = dest(rel);
        assert_eq!(
            system.lstat(Path::new(&path)).ok(),
            before.lstat(Path::new(&path)).ok(),
            "{rel}"
        );
    }
    assert_file(&system, ".file", "# old\n", 0o644);
}
