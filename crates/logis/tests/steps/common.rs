//! Steps shared by every feature.

use cucumber::{given, then, when};

use super::{LogisWorld, Tool};

fn sandbox(world: &LogisWorld) -> &std::path::Path {
    world
        .sandbox
        .as_ref()
        .expect("scenario must start with a sandbox step")
        .path()
}

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

#[when(expr = "I run {string}")]
#[expect(
    clippy::needless_pass_by_value,
    reason = "cucumber passes captured parameters by value"
)]
fn run(world: &mut LogisWorld, args: String) {
    let args: Vec<&str> = args.split_whitespace().collect();
    let output = Tool::from_env()
        .command(sandbox(world), &args)
        .output()
        .expect("run tool");
    world.last = Some(output);
}

fn last(world: &LogisWorld) -> &std::process::Output {
    world.last.as_ref().expect("a command must have run")
}

#[then("it succeeds")]
fn succeeds(world: &mut LogisWorld) {
    let out = last(world);
    assert!(
        out.status.success(),
        "exit {:?}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[then("the output names the tool")]
fn output_names_tool(world: &mut LogisWorld) {
    let stdout = String::from_utf8_lossy(&last(world).stdout);
    let name = Tool::from_env().name();
    assert!(
        stdout.contains(name),
        "expected {name:?} in stdout: {stdout}"
    );
}

#[then("the home directory is still empty")]
fn home_still_empty(world: &mut LogisWorld) {
    let home = sandbox(world).join("home");
    let entries: Vec<_> = std::fs::read_dir(&home)
        .expect("read home")
        .map(|e| e.expect("dir entry").file_name())
        .collect();
    assert!(entries.is_empty(), "home is not empty: {entries:?}");
}
