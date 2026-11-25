//! Unit tests for BDD support code that the cucumber runner cannot test itself.

#[path = "steps/source_names.rs"]
mod source_names;

use source_names::{Kind, decode, encode};

#[test]
fn kinds_parse_from_feature_words() {
    assert_eq!("file".parse(), Ok(Kind::File));
    assert_eq!("exact".parse(), Ok(Kind::ExactDirectory));
    assert_eq!("script".parse(), Ok(Kind::Script));
    assert!("bogus".parse::<Kind>().is_err());
}

#[test]
fn only_directory_kinds_are_directories() {
    assert!(Kind::Directory.is_directory());
    assert!(Kind::ExactDirectory.is_directory());
    assert!(!Kind::File.is_directory());
    assert!(!Kind::Symlink.is_directory());
}

#[test]
fn encodes_dot_and_kind_prefixes() {
    assert_eq!(encode(".file", Kind::File), "dot_file");
    assert_eq!(encode("file", Kind::File), "file");
    assert_eq!(
        encode(".executable", Kind::Executable),
        "executable_dot_executable"
    );
    assert_eq!(encode(".private", Kind::Private), "private_dot_private");
    assert_eq!(encode(".empty", Kind::Empty), "empty_dot_empty");
    assert_eq!(encode(".symlink", Kind::Symlink), "symlink_dot_symlink");
    assert_eq!(encode(".create", Kind::Create), "create_dot_create");
    assert_eq!(encode(".dir", Kind::Directory), "dot_dir");
    assert_eq!(encode("subdir", Kind::ExactDirectory), "exact_subdir");
    assert_eq!(encode("script", Kind::Script), "run_script");
}

#[test]
fn decodes_any_attribute_prefixes_and_template_suffix() {
    assert_eq!(decode("dot_file"), ".file");
    assert_eq!(decode("private_executable_dot_x"), ".x");
    assert_eq!(decode("exact_subdir"), "subdir");
    assert_eq!(decode("dot_template.tmpl"), ".template");
    assert_eq!(decode("plain"), "plain");
    assert_eq!(decode("run_once_before_script"), "script");
    assert_eq!(decode("run_once_before_script"), "script");
}

#[test]
fn decode_inverts_encode() {
    for kind in [
        Kind::File,
        Kind::Executable,
        Kind::Private,
        Kind::Empty,
        Kind::Symlink,
        Kind::Create,
        Kind::Script,
        Kind::Directory,
        Kind::ExactDirectory,
    ] {
        for name in [".file", "file", ".dot_in_name"] {
            assert_eq!(decode(&encode(name, kind)), name, "{kind:?} {name}");
        }
    }
}
