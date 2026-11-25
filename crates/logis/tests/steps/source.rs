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
