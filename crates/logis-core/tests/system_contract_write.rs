//! The write half of the `System` contract: every implementation changes the tree the same
//! way. Modes are chosen so that the test process umask (022 or stricter on group/other write)
//! does not change them.

#![cfg(unix)]

mod system_fixture;

use std::io::ErrorKind;
use std::path::Path;

use logis_core::system::{FileType, System};
use system_fixture::both;

fn kind(sys: &dyn System, path: &Path) -> Option<FileType> {
    sys.lstat(path).ok().map(|m| m.file_type)
}

#[test]
fn mkdir_creates_a_directory() {
    both(|sys, root| {
        let dir = root.join("new");
        sys.mkdir(&dir, 0o700).unwrap();
        let meta = sys.lstat(&dir).unwrap();
        assert_eq!((meta.file_type, meta.mode), (FileType::Dir, 0o700));
        assert!(sys.read_dir(&dir).unwrap().is_empty());
    });
}

#[test]
fn mkdir_fails_if_the_entry_exists_or_the_parent_is_missing() {
    both(|sys, root| {
        assert_eq!(
            sys.mkdir(&root.join("dir"), 0o755).unwrap_err().kind(),
            ErrorKind::AlreadyExists
        );
        assert_eq!(
            sys.mkdir(&root.join("nope/new"), 0o755).unwrap_err().kind(),
            ErrorKind::NotFound
        );
    });
}

#[test]
fn chmod_sets_the_mode_through_symlinks() {
    both(|sys, root| {
        sys.chmod(&root.join("dir/file"), 0o600).unwrap();
        assert_eq!(sys.lstat(&root.join("dir/file")).unwrap().mode, 0o600);
        sys.chmod(&root.join("dir/link"), 0o640).unwrap();
        assert_eq!(sys.lstat(&root.join("dir/file")).unwrap().mode, 0o640);
        assert_eq!(
            sys.chmod(&root.join("nope"), 0o600).unwrap_err().kind(),
            ErrorKind::NotFound
        );
    });
}

#[test]
fn write_file_creates_a_file_with_exactly_its_mode() {
    both(|sys, root| {
        let path = root.join("dir/new");
        sys.write_file(&path, b"new", 0o664).unwrap();
        assert_eq!(sys.read_file(&path).unwrap(), b"new");
        assert_eq!(sys.lstat(&path).unwrap().mode, 0o664);
    });
}

#[test]
fn write_file_replaces_a_file_or_a_symlink() {
    both(|sys, root| {
        sys.write_file(&root.join("dir/file"), b"replaced", 0o600)
            .unwrap();
        let meta = sys.lstat(&root.join("dir/file")).unwrap();
        assert_eq!((meta.file_type, meta.mode), (FileType::File, 0o600));
        assert_eq!(sys.read_file(&root.join("dir/file")).unwrap(), b"replaced");

        sys.write_file(&root.join("dir/link"), b"not a link", 0o644)
            .unwrap();
        assert_eq!(kind(sys, &root.join("dir/link")), Some(FileType::File));
        assert_eq!(sys.read_file(&root.join("dir/file")).unwrap(), b"replaced");
    });
}

#[test]
fn write_file_fails_over_a_directory_or_without_a_parent() {
    both(|sys, root| {
        assert!(sys.write_file(&root.join("dir/sub"), b"x", 0o644).is_err());
        assert_eq!(kind(sys, &root.join("dir/sub")), Some(FileType::Dir));
        assert_eq!(
            sys.write_file(&root.join("nope/file"), b"x", 0o644)
                .unwrap_err()
                .kind(),
            ErrorKind::NotFound
        );
    });
}

#[test]
fn write_symlink_creates_or_replaces_a_symlink() {
    both(|sys, root| {
        sys.write_symlink(Path::new("dir/file"), &root.join("new"))
            .unwrap();
        assert_eq!(
            sys.read_link(&root.join("new")).unwrap(),
            Path::new("dir/file")
        );
        sys.write_symlink(Path::new("sub"), &root.join("dir/link"))
            .unwrap();
        assert_eq!(
            sys.read_link(&root.join("dir/link")).unwrap(),
            Path::new("sub")
        );
        sys.write_symlink(Path::new("x"), &root.join("exe"))
            .unwrap();
        assert_eq!(kind(sys, &root.join("exe")), Some(FileType::Symlink));
    });
}

#[test]
fn remove_all_removes_any_entry_and_ignores_missing_ones() {
    both(|sys, root| {
        sys.remove_all(&root.join("dir/link")).unwrap();
        assert_eq!(kind(sys, &root.join("dir/link")), None);
        assert_eq!(kind(sys, &root.join("dir/file")), Some(FileType::File));

        sys.remove_all(&root.join("dirlink")).unwrap();
        assert_eq!(kind(sys, &root.join("dirlink")), None);
        assert_eq!(kind(sys, &root.join("dir")), Some(FileType::Dir));

        sys.remove_all(&root.join("dir")).unwrap();
        assert_eq!(kind(sys, &root.join("dir")), None);
        assert_eq!(kind(sys, &root.join("dir/sub")), None);

        sys.remove_all(&root.join("nope")).unwrap();
        assert_eq!(kind(sys, &root.join("exe")), Some(FileType::File));
    });
}
