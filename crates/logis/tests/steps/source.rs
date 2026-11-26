//! The source state: fixtures and assertions, in target-path terms.

#![allow(
    clippy::needless_pass_by_value,
    reason = "cucumber passes captured step parameters by value"
)]

use std::fs;

use cucumber::gherkin::Step;
use cucumber::{given, then};

use super::LogisWorld;
use super::Tool;
use super::source_names::Kind;

fn kind(word: &str) -> Kind {
    word.parse().unwrap_or_else(|e| panic!("{e}"))
}

/// The source file contents for `kind`: a trailing newline for files and symlink targets,
/// nothing for empty files.
fn source_contents(kind: Kind, contents: &str) -> String {
    match kind {
        Kind::Empty => String::new(),
        _ => format!("{contents}\n"),
    }
}

/// A table of `| target | kind | contents |`. Kinds: file, executable, private, empty, symlink,
/// create, directory, exact. List directories before the files inside them.
#[given("the source state manages:")]
fn source_manages(world: &mut LogisWorld, step: &Step) {
    let table = step.table.as_ref().expect("a table");
    for row in table.rows.iter().skip(1) {
        let kind = kind(&row[1]);
        let path = world.source_path(&row[0], kind);
        if kind.is_directory() {
            fs::create_dir_all(&path).expect("create source dir");
            continue;
        }
        fs::create_dir_all(path.parent().expect("parent")).expect("create source parent");
        let contents = row.get(2).map_or("", String::as_str);
        fs::write(&path, source_contents(kind, contents)).expect("write source file");
    }
}

/// A file in the source directory whose name starts with `.`: not part of the source state
/// unless it is one of the tool's special files.
#[given(expr = "the source directory contains a hidden file {string}")]
fn hidden_source_file(world: &mut LogisWorld, name: String) {
    assert!(
        name.starts_with('.') && !name.contains('/'),
        "hidden file names start with . and have no /: {name:?}"
    );
    fs::write(world.source_dir().join(name), "# hidden\n").expect("write hidden file");
}

/// Remove everything from the source state (for scenarios that override the Background).
#[given("the source state is empty")]
fn source_is_empty(world: &mut LogisWorld) {
    let dir = world.source_dir();
    fs::remove_dir_all(&dir).expect("clear source");
    fs::create_dir_all(&dir).expect("recreate source");
}

/// Targets matching `pattern` are ignored (the tool's ignore file).
#[given(expr = "the source state ignores {string}")]
fn source_ignores(world: &mut LogisWorld, pattern: String) {
    let path = world.source_dir().join(Tool::from_env().ignore_file_name());
    let mut contents = fs::read_to_string(&path).unwrap_or_default();
    contents.push_str(&pattern);
    contents.push('\n');
    fs::write(path, contents).expect("write ignore file");
}

/// The whole ignore file at the top of the source directory.
#[given("the ignore file contains:")]
fn ignore_file_contains(world: &mut LogisWorld, step: &Step) {
    let path = world.source_dir().join(Tool::from_env().ignore_file_name());
    fs::write(path, docstring(step)).expect("write ignore file");
}

/// The whole ignore file in the source directory of target directory `dir`, which must
/// already be in the source state.
#[given(expr = "the ignore file in {string} contains:")]
fn ignore_file_in_contains(world: &mut LogisWorld, dir: String, step: &Step) {
    let source_dir = world
        .find_source(&dir)
        .unwrap_or_else(|| panic!("{dir} is not in the source state"));
    let path = source_dir.join(Tool::from_env().ignore_file_name());
    fs::write(path, docstring(step)).expect("write ignore file");
}

/// A table of `| path | kind | contents |` for the tool's scripts directory. Paths are
/// relative to that directory; only the last component is encoded by kind.
#[given("the scripts directory contains:")]
fn scripts_dir_contains(world: &mut LogisWorld, step: &Step) {
    let scripts_dir = world.source_dir().join(Tool::from_env().scripts_dir_name());
    let table = step.table.as_ref().expect("a table");
    for row in table.rows.iter().skip(1) {
        let kind = kind(&row[1]);
        let (parent, name) = row[0].rsplit_once('/').unwrap_or(("", &row[0]));
        let dir = scripts_dir.join(parent);
        fs::create_dir_all(&dir).expect("create scripts dir");
        let contents = row.get(2).map_or("", String::as_str);
        fs::write(
            dir.join(super::source_names::encode(name, kind)),
            source_contents(kind, contents),
        )
        .expect("write script");
    }
}

/// The tool's ignore file, empty, inside its scripts directory, where special files are not
/// allowed.
#[given("the scripts directory contains an ignore file")]
fn scripts_dir_contains_ignore_file(world: &mut LogisWorld) {
    let tool = Tool::from_env();
    let scripts_dir = world.source_dir().join(tool.scripts_dir_name());
    fs::create_dir_all(&scripts_dir).expect("create scripts dir");
    fs::write(scripts_dir.join(tool.ignore_file_name()), "").expect("write ignore file");
}

/// A file whose name starts with `.` in the tool's scripts directory: skipped, like hidden files
/// elsewhere in the source directory.
#[given(expr = "the scripts directory contains a hidden file {string}")]
fn scripts_dir_hidden_file(world: &mut LogisWorld, name: String) {
    assert!(
        name.starts_with('.') && !name.contains('/'),
        "hidden file names start with . and have no /: {name:?}"
    );
    let scripts_dir = world.source_dir().join(Tool::from_env().scripts_dir_name());
    fs::create_dir_all(&scripts_dir).expect("create scripts dir");
    fs::write(scripts_dir.join(name), "").expect("write hidden file");
}

/// A docstring's text with a final newline, as a file's contents.
fn docstring(step: &Step) -> String {
    let body = step.docstring.as_ref().expect("a docstring");
    format!("{}\n", body.trim_start_matches('\n').trim_end_matches('\n'))
}

#[then(expr = "the source state manages {string} as a(n) {word}")]
fn manages_as(world: &mut LogisWorld, target: String, kind_word: String) {
    let kind = kind(&kind_word);
    let path = world.source_path(&target, kind);
    let meta = fs::symlink_metadata(&path).unwrap_or_else(|_| {
        panic!(
            "{target} is not in the source state as {kind:?}; found {:?}",
            world.find_source(&target)
        )
    });
    assert_eq!(
        meta.is_dir(),
        kind.is_directory(),
        "{target} at {}",
        path.display()
    );
}

#[then(expr = "the source state manages {string} as a(n) {word} containing {string}")]
fn manages_as_containing(
    world: &mut LogisWorld,
    target: String,
    kind_word: String,
    contents: String,
) {
    manages_as(world, target.clone(), kind_word.clone());
    let kind = kind(&kind_word);
    let actual = fs::read_to_string(world.source_path(&target, kind)).expect("read source");
    assert_eq!(actual, source_contents(kind, &contents), "{target}");
}

#[then(expr = "the source state manages {string} as a(n) {word} containing:")]
fn manages_as_containing_docstring(
    world: &mut LogisWorld,
    target: String,
    kind_word: String,
    step: &Step,
) {
    let docstring = step.docstring.as_ref().expect("a docstring");
    let body = docstring.trim_start_matches('\n').trim_end_matches('\n');
    manages_as_containing(world, target, kind_word, body.to_owned());
}

#[then(expr = "the source state does not manage {string}")]
fn does_not_manage(world: &mut LogisWorld, target: String) {
    if let Some(path) = world.find_source(&target) {
        panic!("{target} is in the source state at {}", path.display());
    }
}
