//! The target state of each kind of source entry, ported from chezmoi's
//! `newSourceStateDir`, `newSourceStateFile` and the `new*TargetStateEntryFunc` functions
//! (`internal/chezmoi/sourcestate.go`). chezmoi has no table test for these on their own; its
//! tests reach them through apply. Templates, encryption and modify scripts are not supported
//! yet and are errors.

use std::path::Path;

use logis_core::source_name::ScriptCondition;
use logis_core::source_state::{SourceEntry, SourceState};
use logis_core::system::MemSystem;
use logis_core::target_state::{TargetEntry, TargetError, target_entry};

const DEST: &str = "/home/user";
const SOURCE: &str = "/home/user/.local/share/logis";
const UMASK: u32 = 0o022;

fn source(rel: &str) -> String {
    format!("{SOURCE}/{rel}")
}

/// The target state of `target`, read from `system`.
fn target(system: &MemSystem, target: &str) -> Result<TargetEntry, TargetError> {
    let state = SourceState::read(system, Path::new(SOURCE), Path::new(DEST)).unwrap();
    let entry = state
        .get(target)
        .unwrap_or_else(|| panic!("{target} is not managed"));
    target_entry(system, entry, &Path::new(DEST).join(target), UMASK)
}

fn file(contents: &str, perm: u32) -> TargetEntry {
    TargetEntry::File {
        contents: contents.as_bytes().to_vec(),
        perm,
        empty: false,
    }
}

#[test]
fn dirs_have_their_attributes_perm_less_the_umask() {
    let system = MemSystem::new()
        .with_dir(source("dot_dir"), 0o755)
        .with_dir(source("private_dot_private"), 0o755)
        .with_dir(source("readonly_dot_readonly"), 0o755)
        .with_dir(source("exact_dot_exact"), 0o755);
    for (name, perm) in [
        (".dir", 0o755),
        (".private", 0o700),
        (".readonly", 0o555),
        (".exact", 0o755),
    ] {
        assert_eq!(
            target(&system, name).unwrap(),
            TargetEntry::Dir { perm },
            "{name}"
        );
    }
}

#[test]
fn files_have_their_contents_and_perm() {
    let system = MemSystem::new()
        .with_file(source("dot_file"), "# file\n", 0o644)
        .with_file(source("executable_dot_exe"), "#!\n", 0o644)
        .with_file(source("private_dot_private"), "secret\n", 0o644)
        .with_file(source("readonly_dot_readonly"), "ro\n", 0o644);
    assert_eq!(target(&system, ".file").unwrap(), file("# file\n", 0o644));
    assert_eq!(target(&system, ".exe").unwrap(), file("#!\n", 0o755));
    assert_eq!(
        target(&system, ".private").unwrap(),
        file("secret\n", 0o600)
    );
    assert_eq!(target(&system, ".readonly").unwrap(), file("ro\n", 0o444));
}

#[test]
fn empty_attribute_is_kept() {
    let system = MemSystem::new().with_file(source("empty_dot_empty"), "", 0o644);
    assert_eq!(
        target(&system, ".empty").unwrap(),
        TargetEntry::File {
            contents: Vec::new(),
            perm: 0o644,
            empty: true,
        }
    );
}

#[test]
fn create_uses_the_source_when_the_target_is_missing() {
    let system = MemSystem::new().with_file(source("create_dot_create"), "# create\n", 0o644);
    assert_eq!(
        target(&system, ".create").unwrap(),
        file("# create\n", 0o644)
    );
}

#[test]
fn create_keeps_an_existing_target() {
    let system = MemSystem::new()
        .with_file(source("create_dot_create"), "# create\n", 0o644)
        .with_file(format!("{DEST}/.create"), "# existing\n", 0o600);
    assert_eq!(
        target(&system, ".create").unwrap(),
        file("# existing\n", 0o644)
    );
}

#[test]
fn create_fails_when_the_target_cannot_be_read() {
    let system = MemSystem::new()
        .with_file(source("create_dot_create"), "# create\n", 0o644)
        .with_dir(format!("{DEST}/.create"), 0o755);
    assert!(matches!(
        target(&system, ".create"),
        Err(TargetError::Io { .. })
    ));
}

/// `newSymlinkTargetStateEntryFunc` trims space; `normalizeLinkname` changes nothing on Unix.
#[test]
fn symlinks_link_to_their_trimmed_contents() {
    let system = MemSystem::new().with_file(source("symlink_dot_link"), "  .dir/file \n", 0o644);
    assert_eq!(
        target(&system, ".link").unwrap(),
        TargetEntry::Symlink {
            linkname: ".dir/file".to_owned(),
        }
    );
}

#[test]
fn scripts_have_their_contents_and_condition() {
    let system = MemSystem::new()
        .with_file(source("run_always"), "#!/bin/sh\n", 0o644)
        .with_file(source("run_once_once"), "#!/bin/sh\necho once\n", 0o644);
    assert_eq!(
        target(&system, "always").unwrap(),
        TargetEntry::Script {
            contents: b"#!/bin/sh\n".to_vec(),
            condition: ScriptCondition::Always,
        }
    );
    assert_eq!(
        target(&system, "once").unwrap(),
        TargetEntry::Script {
            contents: b"#!/bin/sh\necho once\n".to_vec(),
            condition: ScriptCondition::Once,
        }
    );
}

#[test]
fn remove_and_absent_entries_are_absent() {
    let system = MemSystem::new()
        .with_file(source("remove_dot_gone"), "", 0o644)
        .with_dir(source("exact_dot_dir"), 0o755)
        .with_file(format!("{DEST}/.dir/extra"), "", 0o644);
    assert_eq!(target(&system, ".gone").unwrap(), TargetEntry::Absent);
    assert_eq!(target(&system, ".dir/extra").unwrap(), TargetEntry::Absent);
}

#[test]
fn templates_encryption_and_modify_scripts_are_not_supported_yet() {
    let system = MemSystem::new()
        .with_file(source("dot_template.tmpl"), "{{ x }}\n", 0o644)
        .with_file(source("encrypted_dot_secret"), "x", 0o644)
        .with_file(source("modify_dot_modified"), "#!/bin/sh\n", 0o644);
    for (name, what) in [
        (".template", "templates"),
        (".secret", "encrypted files"),
        (".modified", "modify scripts"),
    ] {
        assert_eq!(
            target(&system, name).unwrap_err().to_string(),
            format!("{what} are not supported yet"),
            "{name}"
        );
    }
}

#[test]
fn unreadable_source_files_are_errors() {
    let entry = SourceEntry::File {
        source_path: source("dot_gone").into(),
        source_rel: "dot_gone".to_owned(),
        attr: logis_core::source_name::FileAttr::parse("dot_gone", "").unwrap(),
    };
    let err = target_entry(
        &MemSystem::new(),
        &entry,
        Path::new("/home/user/.gone"),
        UMASK,
    )
    .unwrap_err();
    assert!(err.to_string().starts_with(&source("dot_gone")), "{err}");
}
