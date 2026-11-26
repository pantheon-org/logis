//! The home (destination) directory: fixtures, edits and assertions.

#![allow(
    clippy::needless_pass_by_value,
    reason = "cucumber passes captured step parameters by value"
)]

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::Path;

use cucumber::gherkin::Step;
use cucumber::{given, then, when};

use super::LogisWorld;

fn set_mode(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("set mode");
}

/// Create `path` and any missing parents with mode 0755, whatever the caller's umask: the tool
/// reads these modes (a 0700 directory would be added as private).
fn make_dirs(path: &Path) {
    if path.is_dir() {
        return;
    }
    if let Some(parent) = path.parent() {
        make_dirs(parent);
    }
    fs::create_dir(path).expect("create dir");
    set_mode(path, 0o755);
}

fn write_file(path: &Path, contents: &str, mode: u32) {
    if let Some(parent) = path.parent() {
        make_dirs(parent);
    }
    fs::write(path, contents).expect("write file");
    set_mode(path, mode);
}

fn line(text: &str) -> String {
    format!("{text}\n")
}

/// A table of `| target | kind | contents |`, kinds: file, executable, private, empty, symlink,
/// directory. File contents get a trailing newline; symlink contents are the link target.
#[given("the home directory contains:")]
fn home_contains(world: &mut LogisWorld, step: &Step) {
    let table = step.table.as_ref().expect("a table");
    for row in table.rows.iter().skip(1) {
        let (target, kind, contents) = (
            &row[0],
            row[1].as_str(),
            row.get(2).map_or("", String::as_str),
        );
        let path = world.target(target);
        match kind {
            "file" => write_file(&path, &line(contents), 0o644),
            "executable" => write_file(&path, &line(contents), 0o755),
            "private" => write_file(&path, &line(contents), 0o600),
            "empty" => write_file(&path, "", 0o644),
            "directory" => {
                make_dirs(&path);
            }
            "symlink" => {
                make_dirs(path.parent().expect("parent"));
                symlink(contents, &path).expect("create symlink");
            }
            other => panic!("unknown home entry kind {other:?}"),
        }
    }
}

#[when(expr = "I append {string} to {string}")]
fn append_home(world: &mut LogisWorld, text: String, target: String) {
    append(&world.target(&target), &text);
}

#[when(expr = "I append {string} to the source state of {string}")]
fn append_source(world: &mut LogisWorld, text: String, target: String) {
    let path = world
        .find_source(&target)
        .unwrap_or_else(|| panic!("{target} is not in the source state"));
    append(&path, &text);
}

fn append(path: &Path, text: &str) {
    let mut contents = fs::read_to_string(path).expect("read file");
    contents.push_str(&line(text));
    fs::write(path, contents).expect("write file");
}

#[when(expr = "I remove {string}")]
fn remove(world: &mut LogisWorld, target: String) {
    let path = world.target(&target);
    let meta = fs::symlink_metadata(&path).expect("target exists");
    if meta.is_dir() {
        fs::remove_dir_all(&path).expect("remove dir");
    } else {
        fs::remove_file(&path).expect("remove file");
    }
}

#[when(expr = "I change the mode of {string} to {word}")]
fn change_mode(world: &mut LogisWorld, target: String, mode: String) {
    let mode = u32::from_str_radix(&mode, 8).expect("octal mode");
    set_mode(&world.target(&target), mode);
}

/// Replace the entry at `target`, a file, symlink or directory, with a regular file.
#[when(expr = "I replace {string} with a file containing {string}")]
fn replace_with_file(world: &mut LogisWorld, target: String, contents: String) {
    remove(world, target.clone());
    write_file(&world.target(&target), &line(&contents), 0o644);
}

#[when(expr = "I create a symlink {string} to {string}")]
fn create_symlink(world: &mut LogisWorld, target: String, link: String) {
    symlink(&link, world.target(&target)).expect("create symlink");
}

#[when(expr = "I create the directory {string}")]
fn create_dir(world: &mut LogisWorld, target: String) {
    make_dirs(&world.target(&target));
}

#[then(expr = "{string} exists")]
fn exists(world: &mut LogisWorld, target: String) {
    assert!(
        fs::symlink_metadata(world.target(&target)).is_ok(),
        "{target} does not exist"
    );
}

#[then(expr = "{string} does not exist")]
fn does_not_exist(world: &mut LogisWorld, target: String) {
    assert!(
        fs::symlink_metadata(world.target(&target)).is_err(),
        "{target} exists"
    );
}

#[then(expr = "{string} is a directory")]
fn is_directory(world: &mut LogisWorld, target: String) {
    let meta = fs::symlink_metadata(world.target(&target)).expect("exists");
    assert!(meta.is_dir(), "{target} is not a directory");
}

#[then(expr = "{string} is a symlink to {string}")]
fn is_symlink_to(world: &mut LogisWorld, target: String, link: String) {
    let actual = fs::read_link(world.target(&target)).expect("is a symlink");
    assert_eq!(actual, Path::new(&link));
}

#[then(expr = "the file {string} has contents {string}")]
fn has_contents(world: &mut LogisWorld, target: String, contents: String) {
    let actual = fs::read_to_string(world.target(&target)).expect("read file");
    assert_eq!(actual, line(&contents), "{target}");
}

/// Exact file contents; an empty docstring means an empty file.
#[then(expr = "the file {string} is:")]
fn file_is(world: &mut LogisWorld, target: String, step: &Step) {
    let docstring = step.docstring.as_ref().expect("a docstring");
    let body = docstring.trim_start_matches('\n').trim_end_matches('\n');
    let expected = if body.is_empty() {
        String::new()
    } else {
        line(body)
    };
    let actual = fs::read_to_string(world.target(&target)).expect("read file");
    assert_eq!(actual, expected, "{target}");
}

#[then(expr = "the file {string} is executable")]
fn is_executable(world: &mut LogisWorld, target: String) {
    let mode = fs::metadata(world.target(&target))
        .expect("exists")
        .permissions()
        .mode();
    assert_ne!(mode & 0o111, 0, "{target} mode {mode:o} is not executable");
}

#[then(expr = "the file {string} is private")]
fn is_private(world: &mut LogisWorld, target: String) {
    let mode = fs::metadata(world.target(&target))
        .expect("exists")
        .permissions()
        .mode();
    assert_eq!(
        mode & 0o077,
        0,
        "{target} mode {mode:o} is readable by others"
    );
}

#[then("the home directory is still empty")]
fn home_still_empty(world: &mut LogisWorld) {
    let entries: Vec<_> = fs::read_dir(world.home())
        .expect("read home")
        .map(|e| e.expect("dir entry").file_name())
        .collect();
    assert!(entries.is_empty(), "home is not empty: {entries:?}");
}
