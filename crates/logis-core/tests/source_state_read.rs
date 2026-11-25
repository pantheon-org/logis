//! Port of the in-scope cases of chezmoi's `TestSourceStateRead`
//! (`internal/chezmoi/sourcestate_test.go`), on an in-memory filesystem. Case names are the Go
//! names; extra cases say where they come from. Out of scope for now: .chezmoiignore,
//! .chezmoiremove, externals, .chezmoitemplates, templates and .chezmoiversion.

use std::path::Path;

use logis_core::source_name::{DirAttr, FileAttr, ScriptCondition, TargetType};
use logis_core::source_state::{ReadError, SourceEntry, SourceState};
use logis_core::system::MemSystem;

const SOURCE: &str = "/home/user/.local/share/chezmoi";

fn source(rel: &str) -> String {
    format!("{SOURCE}/{rel}")
}

fn read(system: &MemSystem) -> Result<SourceState, ReadError> {
    SourceState::read(system, Path::new(SOURCE))
}

/// The state's targets with a short description of each entry.
fn summary(state: &SourceState) -> Vec<(String, String)> {
    state
        .entries()
        .map(|(target, entry)| (target.to_owned(), entry.source_rel().to_owned()))
        .collect()
}

fn file_attr(target_name: &str, target_type: TargetType) -> FileAttr {
    FileAttr {
        target_name: target_name.to_owned(),
        target_type,
        ..FileAttr::default()
    }
}

#[test]
fn empty() {
    let state = read(&MemSystem::new().with_dir(SOURCE, 0o777)).unwrap();
    assert!(state.is_empty());
}

/// Not a Go table case: `Read` returns early when the source directory does not exist.
#[test]
fn missing_source_dir_is_empty() {
    let state = read(&MemSystem::new().with_dir("/home/user", 0o755)).unwrap();
    assert!(state.is_empty());
}

/// Not a Go table case: `Read` rejects a source path that is not a directory.
#[test]
fn source_dir_that_is_a_file_is_an_error() {
    let err = read(&MemSystem::new().with_file(SOURCE, "", 0o644)).unwrap_err();
    assert_eq!(err.to_string(), format!("{SOURCE}: not a directory"));
}

#[test]
fn dir() {
    let state = read(&MemSystem::new().with_dir(source("dir"), 0o755)).unwrap();
    assert_eq!(
        state.get("dir"),
        Some(&SourceEntry::Dir {
            source_path: source("dir").into(),
            source_rel: "dir".into(),
            attr: DirAttr {
                target_name: "dir".into(),
                ..DirAttr::default()
            },
        })
    );
    assert_eq!(state.entries().count(), 1);
}

#[test]
fn file() {
    let system = MemSystem::new().with_file(source("dot_file"), "# contents of .file\n", 0o644);
    let state = read(&system).unwrap();
    assert_eq!(
        state.get(".file"),
        Some(&SourceEntry::File {
            source_path: source("dot_file").into(),
            source_rel: "dot_file".into(),
            attr: file_attr(".file", TargetType::File),
        })
    );
    assert_eq!(state.entries().count(), 1);
}

fn inconsistent(system: &MemSystem) -> String {
    match read(system) {
        Err(err @ ReadError::Inconsistent(_)) => err.to_string(),
        other => panic!("expected an inconsistent state, got {other:?}"),
    }
}

#[test]
fn duplicate_target_file() {
    let system = MemSystem::new()
        .with_file(source("dot_file"), "# contents of .file\n", 0o644)
        .with_file(source("dot_file.tmpl"), "# contents of .file\n", 0o644);
    assert_eq!(
        inconsistent(&system),
        format!(
            ".file: inconsistent state ({}, {})",
            source("dot_file"),
            source("dot_file.tmpl")
        )
    );
}

#[test]
fn duplicate_target_dir() {
    let system = MemSystem::new()
        .with_dir(source("dir"), 0o755)
        .with_dir(source("exact_dir"), 0o755);
    assert_eq!(
        inconsistent(&system),
        format!(
            "dir: inconsistent state ({}, {})",
            source("dir"),
            source("exact_dir")
        )
    );
}

#[test]
fn duplicate_target_script() {
    let system = MemSystem::new()
        .with_file(source("run_script"), "#!/bin/sh\n", 0o644)
        .with_file(source("run_once_script"), "#!/bin/sh\n", 0o644);
    assert_eq!(
        inconsistent(&system),
        format!(
            "script: inconsistent state ({}, {})",
            source("run_once_script"),
            source("run_script")
        )
    );
}

/// Not a Go table case: every inconsistent target is reported, one per line, in target order.
#[test]
fn every_inconsistent_target_is_reported() {
    let system = MemSystem::new()
        .with_file(source("dot_b"), "", 0o644)
        .with_file(source("private_dot_b"), "", 0o644)
        .with_file(source("dot_a"), "", 0o644)
        .with_file(source("executable_dot_a"), "", 0o644);
    let message = inconsistent(&system);
    let lines: Vec<&str> = message.lines().collect();
    assert_eq!(lines.len(), 2, "{message}");
    assert!(lines[0].starts_with(".a: inconsistent state"));
    assert!(lines[1].starts_with(".b: inconsistent state"));
}

#[test]
fn symlink_with_attr() {
    let system = MemSystem::new()
        .with_file(source(".file"), "# contents of .file\n", 0o644)
        .with_symlink(source("executable_dot_file"), ".file");
    let state = read(&system).unwrap();
    assert_eq!(
        state.get(".file"),
        Some(&SourceEntry::File {
            source_path: source("executable_dot_file").into(),
            source_rel: "executable_dot_file".into(),
            attr: FileAttr {
                executable: true,
                ..file_attr(".file", TargetType::File)
            },
        })
    );
    assert_eq!(state.entries().count(), 1);
}

#[test]
fn symlink_script() {
    let system = MemSystem::new()
        .with_file(source(".script"), "# contents of .script\n", 0o644)
        .with_symlink(source("run_script"), ".script");
    let state = read(&system).unwrap();
    assert_eq!(
        state.get("script"),
        Some(&SourceEntry::File {
            source_path: source("run_script").into(),
            source_rel: "run_script".into(),
            attr: FileAttr {
                condition: ScriptCondition::Always,
                ..file_attr("script", TargetType::Script)
            },
        })
    );
    assert_eq!(state.entries().count(), 1);
}

#[test]
fn script() {
    let system = MemSystem::new().with_file(source("run_script"), "# contents of script\n", 0o644);
    let state = read(&system).unwrap();
    let Some(SourceEntry::File { attr, .. }) = state.get("script") else {
        panic!("script missing: {state:?}");
    };
    assert_eq!(attr.target_type, TargetType::Script);
    assert_eq!(attr.condition, ScriptCondition::Always);
}

#[test]
fn symlink() {
    let system =
        MemSystem::new().with_file(source("symlink_dot_symlink"), ".dir/subdir/file", 0o644);
    let state = read(&system).unwrap();
    assert_eq!(
        state.get(".symlink"),
        Some(&SourceEntry::File {
            source_path: source("symlink_dot_symlink").into(),
            source_rel: "symlink_dot_symlink".into(),
            attr: file_attr(".symlink", TargetType::Symlink),
        })
    );
}

#[test]
fn file_in_dir() {
    let system = MemSystem::new().with_dir(source("dir"), 0o755).with_file(
        source("dir/file"),
        "# contents of .dir/file\n",
        0o644,
    );
    let state = read(&system).unwrap();
    assert_eq!(
        summary(&state),
        [
            ("dir".to_owned(), "dir".to_owned()),
            ("dir/file".to_owned(), "dir/file".to_owned()),
        ]
    );
}

/// Not a Go table case: parent attributes do not leak into child target paths.
#[test]
fn nested_attributes_map_each_component() {
    let system = MemSystem::new()
        .with_dir(source("dot_dir/exact_subdir"), 0o755)
        .with_file(source("dot_dir/exact_subdir/private_dot_file"), "x", 0o600);
    let state = read(&system).unwrap();
    assert_eq!(
        summary(&state),
        [
            (".dir".to_owned(), "dot_dir".to_owned()),
            (".dir/subdir".to_owned(), "dot_dir/exact_subdir".to_owned()),
            (
                ".dir/subdir/.file".to_owned(),
                "dot_dir/exact_subdir/private_dot_file".to_owned()
            ),
        ]
    );
}

#[test]
fn ignore_dir() {
    let system = MemSystem::new().with_dir(source(".dir"), 0o755).with_file(
        source(".dir/file"),
        "# contents of .dir/file\n",
        0o644,
    );
    assert!(read(&system).unwrap().is_empty());
}

#[test]
fn ignore_file() {
    let system = MemSystem::new().with_file(source(".file"), "# contents of .file\n", 0o644);
    assert!(read(&system).unwrap().is_empty());
}

/// Not a Go table case: a name that does not parse is an error naming the source name.
#[test]
fn invalid_source_name_is_an_error() {
    let system = MemSystem::new().with_file(source("dot_"), "", 0o644);
    assert_eq!(read(&system).unwrap_err().to_string(), "dot_: invalid path");
}

/// Not a Go table case: chezmoi checks for ignored names before following a symlink, so a
/// dangling hidden symlink (an editor lock file) is ignored, not an error.
#[test]
fn dangling_hidden_symlink_is_ignored() {
    let system = MemSystem::new()
        .with_dir(SOURCE, 0o755)
        .with_symlink(source(".#lock"), "user@host.1234");
    assert!(read(&system).unwrap().is_empty());
}
