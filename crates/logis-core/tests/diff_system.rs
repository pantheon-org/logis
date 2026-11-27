//! `GitDiffSystem` under apply: the patches chezmoi's `diff` prints, on an in-memory
//! filesystem, through a dry run so nothing changes.

use std::path::Path;

use logis_core::apply::{ApplyOptions, apply};
use logis_core::diff::GitDiffSystem;
use logis_core::entry_type::EntryTypeFilter;
use logis_core::source_state::SourceState;
use logis_core::system::{DryRunSystem, FileType, MemSystem, System};

const DEST: &str = "/home/user";
const SOURCE: &str = "/home/user/.local/share/logis";

fn source(rel: &str) -> String {
    format!("{SOURCE}/{rel}")
}

fn dest(rel: &str) -> String {
    format!("{DEST}/{rel}")
}

fn diff(system: &MemSystem, filter: EntryTypeFilter, reverse: bool) -> String {
    let state = SourceState::read(system, Path::new(SOURCE), Path::new(DEST)).unwrap();
    let targets: Vec<&str> = state.entries().map(|(t, _)| t).collect();
    let dry_run = DryRunSystem::new(system);
    let differ = GitDiffSystem::new(&dry_run, Path::new(DEST), filter, reverse);
    let options = ApplyOptions {
        dest_dir: Path::new(DEST),
        umask: 0o022,
        filter,
    };
    apply(&differ, &state, &targets, &options).unwrap();
    String::from_utf8(differ.into_output()).unwrap()
}

/// From `diff.txtar`: a file where a symlink belongs is removed, then the symlink written; the
/// second patch still starts from the file, which the dry run left in place.
#[test]
fn file_replaced_by_a_symlink_is_two_patches() {
    let system = MemSystem::new()
        .with_file(dest(".symlink"), "# contents of .file\n", 0o644)
        .with_file(source("symlink_dot_symlink"), ".dir/subdir/file\n", 0o644);
    assert_eq!(
        diff(&system, EntryTypeFilter::default(), false),
        "diff --git a/.symlink b/.symlink\n\
         deleted file mode 100644\n\
         index 8a52cb9ce9551221716a53786ad74104c5902362..0000000000000000000000000000000000000000\n\
         --- a/.symlink\n\
         +++ /dev/null\n\
         @@ -1 +0,0 @@\n\
         -# contents of .file\n\
         diff --git a/.symlink b/.symlink\n\
         old mode 100644\n\
         new mode 120000\n\
         index 8a52cb9ce9551221716a53786ad74104c5902362..9b91fdbb83798a67fbbc5cc4f120c3f7726c0d70\n\
         --- a/.symlink\n\
         +++ b/.symlink\n\
         @@ -1 +1 @@\n\
         -# contents of .file\n\
         +.dir/subdir/file\n"
    );
    assert_eq!(
        system
            .lstat(Path::new(&dest(".symlink")))
            .unwrap()
            .file_type,
        FileType::File
    );
}

/// From `diff.txtar`: a mode change alone.
#[test]
fn a_directory_mode_change() {
    let system = MemSystem::new()
        .with_dir(dest(".dir"), 0o700)
        .with_dir(source("dot_dir"), 0o755);
    assert_eq!(
        diff(&system, EntryTypeFilter::default(), false),
        "diff --git a/.dir b/.dir\nold mode 40700\nnew mode 40755\n"
    );
}

/// After a removal, entries under the removed path read as missing, so nothing under an
/// exact directory's removed child is diffed again.
#[test]
fn removed_entries_hide_what_is_under_them() {
    let system = MemSystem::new()
        .with_file(dest(".dir/sub/file"), "x\n", 0o644)
        .with_dir(source("exact_dot_dir"), 0o755);
    let out = diff(&system, EntryTypeFilter::default(), false);
    assert_eq!(out.matches("diff --git").count(), 1, "{out}");
    assert!(out.starts_with("diff --git a/.dir/sub b/.dir/sub\ndeleted file mode 40755\n"));
}

#[test]
fn excluded_types_make_no_patch() {
    let system = MemSystem::new()
        .with_file(source("dot_file"), "x\n", 0o644)
        .with_dir(source("dot_dir"), 0o755);
    let filter = EntryTypeFilter {
        include: "all".parse().unwrap(),
        exclude: "files".parse().unwrap(),
    };
    let out = diff(&system, filter, false);
    assert!(
        out.starts_with("diff --git a/.dir b/.dir\nnew file mode 40755\n"),
        "{out}"
    );
    assert!(!out.contains(".file"), "{out}");
}

#[test]
fn reverse_swaps_the_sides() {
    let system = MemSystem::new().with_file(source("dot_file"), "x\n", 0o644);
    assert_eq!(
        diff(&system, EntryTypeFilter::default(), true),
        "diff --git a/.file b/.file\n\
         deleted file mode 100644\n\
         index 587be6b4c3f93f93c489c0111bba5596147a26cb..0000000000000000000000000000000000000000\n\
         --- a/.file\n\
         +++ /dev/null\n\
         @@ -1 +0,0 @@\n\
         -x\n"
    );
}
