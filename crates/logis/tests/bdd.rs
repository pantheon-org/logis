//! BDD runner. The same features run against chezmoi (what we have) and logis (what we must
//! achieve), selected by `BDD_TOOL=chezmoi|logis` (default: logis).
//!
//! Tags:
//! - `@shared` (default): must pass on both tools
//! - `@chezmoi-only`: current behaviour logis drops or changes; skipped on logis
//! - `@logis-only`: new or changed behaviour; skipped on chezmoi
//! - `@wip`: not yet passing on logis; skipped on logis, still run on chezmoi
//! - `@excluded`: never run (network, TTY, secret managers)

mod steps;

use cucumber::World as _;
use cucumber::gherkin::{Feature, Rule, Scenario};
use steps::{LogisWorld, Tool};

fn tags<'a>(feature: &'a Feature, rule: Option<&'a Rule>, scenario: &'a Scenario) -> Vec<&'a str> {
    feature
        .tags
        .iter()
        .chain(rule.into_iter().flat_map(|r| r.tags.iter()))
        .chain(scenario.tags.iter())
        .map(String::as_str)
        .collect()
}

fn selected(tool: Tool, tags: &[&str]) -> bool {
    let has = |t: &str| tags.contains(&t);
    if has("excluded") {
        return false;
    }
    match tool {
        Tool::Chezmoi => !has("logis-only"),
        Tool::Logis => !has("chezmoi-only") && !has("wip"),
    }
}

#[tokio::main]
async fn main() {
    let tool = Tool::from_env();
    eprintln!("BDD: running features against {tool:?}");
    LogisWorld::cucumber()
        .filter_run_and_exit(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/features"),
            move |feature, rule, scenario| selected(tool, &tags(feature, rule, scenario)),
        )
        .await;
}
