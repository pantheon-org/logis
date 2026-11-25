//! End-to-end behaviour of `cargo xtask txtar2feature`.

use std::path::Path;
use std::process::{Command, Output};

fn run(args: &[&Path], force: bool) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xtask"));
    cmd.arg("txtar2feature").args(args);
    if force {
        cmd.arg("--force");
    }
    cmd.output().expect("run xtask")
}

fn fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let scripts = dir.path().join("scripts");
    let out = dir.path().join("features");
    std::fs::create_dir_all(&scripts).unwrap();
    std::fs::write(
        scripts.join("apply.txtar"),
        "# test that apply works\nexec chezmoi apply\n",
    )
    .unwrap();
    std::fs::write(
        scripts.join("cd_windows.txtar"),
        "# test cd\nexec chezmoi cd\n",
    )
    .unwrap();
    std::fs::write(scripts.join("notes.txt"), "not a script").unwrap();
    (dir, scripts, out)
}

#[test]
fn writes_one_feature_per_txtar_script() {
    let (_dir, scripts, out) = fixture();
    let output = run(&[&scripts, &out], false);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut names: Vec<_> = std::fs::read_dir(&out)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, ["apply.feature", "cd_windows.feature"]);
    let apply = std::fs::read_to_string(out.join("apply.feature")).unwrap();
    assert!(apply.contains("Scenario: apply works"));
}

#[test]
fn never_overwrites_an_existing_feature_without_force() {
    let (_dir, scripts, out) = fixture();
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("apply.feature"), "hand written").unwrap();

    let output = run(&[&scripts, &out], false);
    assert!(output.status.success());
    assert_eq!(
        std::fs::read_to_string(out.join("apply.feature")).unwrap(),
        "hand written"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("skipped 1 existing"));

    run(&[&scripts, &out], true);
    assert_ne!(
        std::fs::read_to_string(out.join("apply.feature")).unwrap(),
        "hand written"
    );
}

#[test]
fn reports_scenario_totals() {
    let (_dir, scripts, out) = fixture();
    let output = run(&[&scripts, &out], false);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("2 scripts, 2 scenarios"), "{stderr}");
}

#[test]
fn missing_arguments_fail_with_usage() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask")).output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage"));
}
