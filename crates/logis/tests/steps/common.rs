//! Sandbox, running the tool, and checking its output.

#![allow(
    clippy::needless_pass_by_value,
    reason = "cucumber passes captured step parameters by value"
)]

use cucumber::gherkin::Step;
use cucumber::{given, then, when};

use super::{LogisWorld, Tool};

#[given("an empty home directory")]
fn empty_home(world: &mut LogisWorld) {
    let dir = tempfile::tempdir().expect("create sandbox");
    for sub in [
        "home",
        "source",
        "config/chezmoi",
        "config/logis",
        "data",
        "cache",
        "state",
        "tmp",
    ] {
        std::fs::create_dir_all(dir.path().join(sub)).expect("create sandbox dir");
    }
    world.sandbox = Some(dir);
}

fn run_tool(world: &mut LogisWorld, args: &str) {
    let args: Vec<String> = args
        .split_whitespace()
        .map(|a| world.expand_arg(a))
        .collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = Tool::from_env()
        .command(world.root(), &args)
        .output()
        .expect("run tool");
    world.last = Some(output);
}

#[when(expr = "I run {string}")]
fn run(world: &mut LogisWorld, args: String) {
    run_tool(world, &args);
}

/// Setup that runs the tool, for example `Given I have run "apply --force"`. Fails the scenario
/// if the command fails.
#[given(expr = "I have run {string}")]
fn have_run(world: &mut LogisWorld, args: String) {
    run_tool(world, &args);
    succeeds(world);
}

fn describe(out: &std::process::Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[then("it succeeds")]
fn succeeds(world: &mut LogisWorld) {
    let out = world.last_output();
    assert!(out.status.success(), "{}", describe(out));
}

#[then("it fails")]
fn fails(world: &mut LogisWorld) {
    let out = world.last_output();
    assert!(!out.status.success(), "{}", describe(out));
}

#[then("the output names the tool")]
fn output_names_tool(world: &mut LogisWorld) {
    let stdout = String::from_utf8_lossy(&world.last_output().stdout);
    let name = Tool::from_env().name();
    assert!(
        stdout.contains(name),
        "expected {name:?} in stdout: {stdout}"
    );
}

/// stderr contains `text`; `~/...` is expanded to the sandbox path, as in command arguments.
#[then(expr = "the error output mentions {string}")]
fn error_mentions(world: &mut LogisWorld, text: String) {
    let text = world.expand_arg(&text);
    let stderr = String::from_utf8_lossy(&world.last_output().stderr);
    assert!(
        stderr.contains(&text),
        "expected {text:?} in stderr: {stderr}"
    );
}

#[then("the output is empty")]
fn output_is_empty(world: &mut LogisWorld) {
    let out = world.last_output();
    assert!(out.stdout.is_empty(), "{}", describe(out));
}

/// Exact stdout. `{home}` in the expected text stands for the sandbox home directory.
///
/// Gherkin strips the indentation shared by all docstring lines, so output whose lines all start
/// with a space (such as `status`) is written with a `|` margin: if every line starts with `|`,
/// that one character is removed from each line.
#[then("the output is:")]
fn output_is(world: &mut LogisWorld, step: &Step) {
    let docstring = step
        .docstring
        .as_ref()
        .expect("the output is: needs a docstring");
    let body = docstring.trim_start_matches('\n').trim_end_matches('\n');
    let lines: Vec<&str> = body.lines().collect();
    let margin = lines.iter().all(|l| l.starts_with('|'));
    let mut expected: String = lines
        .iter()
        .map(|l| if margin { &l[1..] } else { l })
        .collect::<Vec<_>>()
        .join("\n");
    expected.push('\n');
    let expected = expected.replace("{home}", &world.home().display().to_string());
    let actual = String::from_utf8_lossy(&world.last_output().stdout);
    assert_eq!(actual, expected, "{}", describe(world.last_output()));
}
