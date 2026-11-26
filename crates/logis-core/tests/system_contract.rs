//! The same behaviour from every `System` implementation: `RealSystem` on a temporary
//! directory and `MemSystem` in memory. Each check builds the same tree in both.

#![cfg(unix)]

mod system_fixture;

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::Path;

use logis_core::system::FileType;
use system_fixture::both;

fn names(list: &[&str]) -> Vec<OsString> {
    list.iter().map(OsString::from).collect()
}

#[test]
fn read_file_returns_contents() {
    both(|sys, root| {
        assert_eq!(sys.read_file(&root.join("dir/file")).unwrap(), b"contents");
        assert_eq!(sys.read_file(&root.join("dir/link")).unwrap(), b"contents");
    });
}

#[test]
fn read_file_of_missing_path_is_not_found() {
    both(|sys, root| {
        let err = sys.read_file(&root.join("nope")).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::NotFound);
    });
}

#[test]
fn read_dir_lists_sorted_names() {
    both(|sys, root| {
        assert_eq!(
            sys.read_dir(root).unwrap(),
            names(&["dir", "dirlink", "exe"])
        );
        assert_eq!(
            sys.read_dir(&root.join("dir")).unwrap(),
            names(&["dangling", "file", "link", "sub"])
        );
        assert_eq!(sys.read_dir(&root.join("dir/sub")).unwrap(), names(&[]));
    });
}

#[test]
fn read_dir_of_missing_path_is_not_found() {
    both(|sys, root| {
        assert_eq!(
            sys.read_dir(&root.join("nope")).unwrap_err().kind(),
            ErrorKind::NotFound
        );
    });
}

/// Exact directories rely on this to tell "a file is in the way" from other errors.
#[test]
fn read_dir_of_a_file_is_not_a_directory() {
    both(|sys, root| {
        for path in ["dir/file", "dir/link"] {
            assert_eq!(
                sys.read_dir(&root.join(path)).unwrap_err().kind(),
                ErrorKind::NotADirectory,
                "{path}"
            );
        }
    });
}

#[test]
fn lstat_reports_types_and_modes_without_following_symlinks() {
    both(|sys, root| {
        let meta = |p: &str| sys.lstat(&root.join(p)).unwrap();
        assert_eq!(meta("dir").file_type, FileType::Dir);
        assert_eq!(meta("dir").mode, 0o755);
        assert_eq!(meta("dir/sub").mode, 0o700);
        assert_eq!(meta("dir/file").file_type, FileType::File);
        assert_eq!(meta("dir/file").mode, 0o644);
        assert_eq!(meta("exe").mode, 0o755);
        assert_eq!(meta("dir/link").file_type, FileType::Symlink);
        assert_eq!(meta("dir/dangling").file_type, FileType::Symlink);
    });
}

#[test]
fn stat_follows_symlinks() {
    both(|sys, root| {
        let meta = sys.stat(&root.join("dir/link")).unwrap();
        assert_eq!(meta.file_type, FileType::File);
        assert_eq!(meta.mode, 0o644);
        let err = sys.stat(&root.join("dir/dangling")).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::NotFound);
    });
}

#[test]
fn read_link_returns_the_target() {
    both(|sys, root| {
        assert_eq!(
            sys.read_link(&root.join("dir/link")).unwrap(),
            Path::new("file")
        );
        assert_eq!(
            sys.read_link(&root.join("dir/dangling")).unwrap(),
            Path::new("missing")
        );
        assert!(sys.read_link(&root.join("dir/file")).is_err());
    });
}

#[test]
fn missing_paths_are_not_found_everywhere() {
    both(|sys, root| {
        let p = root.join("nope/deeper");
        assert_eq!(sys.lstat(&p).unwrap_err().kind(), ErrorKind::NotFound);
        assert_eq!(sys.stat(&p).unwrap_err().kind(), ErrorKind::NotFound);
        assert_eq!(sys.read_link(&p).unwrap_err().kind(), ErrorKind::NotFound);
    });
}

#[test]
fn paths_through_a_symlinked_directory_resolve() {
    both(|sys, root| {
        assert_eq!(
            sys.read_file(&root.join("dirlink/file")).unwrap(),
            b"contents"
        );
        assert_eq!(
            sys.read_dir(&root.join("dirlink")).unwrap(),
            names(&["dangling", "file", "link", "sub"])
        );
        assert_eq!(
            sys.lstat(&root.join("dirlink")).unwrap().file_type,
            FileType::Symlink
        );
        assert_eq!(
            sys.stat(&root.join("dirlink")).unwrap().file_type,
            FileType::Dir
        );
        assert_eq!(
            sys.lstat(&root.join("dirlink/link")).unwrap().file_type,
            FileType::Symlink
        );
    });
}
