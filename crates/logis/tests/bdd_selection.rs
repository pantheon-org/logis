//! Unit tests for the BDD tag selection in `steps/selection.rs`. They live here because the
//! cucumber runner (`tests/bdd.rs`) uses its own harness and compiles `#[test]`s out.

#[path = "steps/selection.rs"]
mod selection;

use selection::{Os, Tool, selected};

#[test]
fn current_os_matches_the_build_target() {
    let expected = if cfg!(target_os = "macos") {
        Os::Mac
    } else if cfg!(windows) {
        Os::Windows
    } else {
        Os::Linux
    };
    assert_eq!(Os::current(), expected);
}

#[test]
fn untagged_scenarios_run_on_both_tools() {
    assert!(selected(Tool::Chezmoi, Os::Mac, &[]));
    assert!(selected(Tool::Logis, Os::Mac, &[]));
}

#[test]
fn skeletons_and_excluded_never_run() {
    for tool in [Tool::Chezmoi, Tool::Logis] {
        assert!(!selected(tool, Os::Mac, &["skeleton"]));
        assert!(!selected(tool, Os::Mac, &["excluded"]));
    }
}

#[test]
fn wip_runs_on_chezmoi_only() {
    assert!(selected(Tool::Chezmoi, Os::Mac, &["wip"]));
    assert!(!selected(Tool::Logis, Os::Mac, &["wip"]));
}

#[test]
fn tool_only_tags_skip_the_other_tool() {
    assert!(!selected(Tool::Logis, Os::Mac, &["chezmoi-only"]));
    assert!(!selected(Tool::Chezmoi, Os::Mac, &["logis-only"]));
}

#[test]
fn os_tags_restrict_where_a_scenario_runs() {
    assert!(selected(Tool::Logis, Os::Mac, &["unix"]));
    assert!(selected(Tool::Logis, Os::Linux, &["unix"]));
    assert!(!selected(Tool::Logis, Os::Windows, &["unix"]));
    assert!(!selected(Tool::Logis, Os::Mac, &["windows"]));
    assert!(!selected(Tool::Logis, Os::Linux, &["macos"]));
    assert!(!selected(Tool::Logis, Os::Mac, &["linux"]));
}

#[test]
fn requirement_tags_do_not_skip() {
    assert!(selected(Tool::Chezmoi, Os::Mac, &["needs-git"]));
}
