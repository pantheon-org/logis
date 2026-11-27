//! Files with exact contents, in the home directory and the source state: numbered lines, no
//! final newline, binary data. For diffs, where every byte shows.

#![allow(
    clippy::needless_pass_by_value,
    reason = "cucumber passes captured step parameters by value"
)]

use std::fs;

use cucumber::gherkin::Step;
use cucumber::given;

use super::LogisWorld;
use super::home::{make_dirs, write_file};
use super::source_names::Kind;

/// `line 1\n` to `line {last}\n`, with the replacements in `except` (a `| line | text |` table).
fn numbered(last: usize, except: Option<&Step>) -> String {
    let mut lines: Vec<String> = (1..=last).map(|n| format!("line {n}")).collect();
    if let Some(table) = except.and_then(|step| step.table.as_ref()) {
        for row in table.rows.iter().skip(1) {
            let n: usize = row[0].parse().expect("a line number");
            lines[n - 1].clone_from(&row[1]);
        }
    }
    lines.iter().flat_map(|l| [l.as_str(), "\n"]).collect()
}

fn docstring(step: &Step) -> String {
    let body = step.docstring.as_ref().expect("a docstring");
    body.trim_start_matches('\n')
        .trim_end_matches('\n')
        .to_owned()
}

/// Write the source state's file for `target`, a plain file, with exactly `contents`.
fn write_source(world: &LogisWorld, target: &str, contents: &[u8]) {
    let path = world.source_path(target, Kind::File);
    make_dirs(path.parent().expect("parent"));
    fs::write(path, contents).expect("write source file");
}

#[given(expr = "the source state manages {string} with lines 1 to {int}")]
fn source_numbered(world: &mut LogisWorld, target: String, last: usize) {
    write_source(world, &target, numbered(last, None).as_bytes());
}

#[given(expr = "the home file {string} has lines 1 to {int}, except:")]
fn home_numbered(world: &mut LogisWorld, target: String, last: usize, step: &Step) {
    write_file(&world.target(&target), &numbered(last, Some(step)), 0o644);
}

#[given(expr = "the source state manages {string} containing, without a final newline:")]
fn source_without_newline(world: &mut LogisWorld, target: String, step: &Step) {
    write_source(world, &target, docstring(step).as_bytes());
}

#[given(expr = "the home file {string} contains:")]
fn home_contains_docstring(world: &mut LogisWorld, target: String, step: &Step) {
    write_file(
        &world.target(&target),
        &format!("{}\n", docstring(step)),
        0o644,
    );
}

#[given(expr = "the home file {string} contains, without a final newline:")]
fn home_without_newline(world: &mut LogisWorld, target: String, step: &Step) {
    write_file(&world.target(&target), &docstring(step), 0o644);
}

/// Bytes that no sniffer takes for text.
#[given(expr = "the source state manages {string} containing binary data")]
fn source_binary(world: &mut LogisWorld, target: String) {
    write_source(world, &target, b"\x00\x01bin");
}
