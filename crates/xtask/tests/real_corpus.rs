//! Every real chezmoi testscript converts to valid Gherkin with one scenario per test.
//!
//! Needs the local chezmoi checkout at `chezmoi/` (gitignored, absent in CI), so it is ignored by
//! default. Run it with `mise run spec:check`.

use std::path::Path;

use xtask::{feature, txtar};

#[test]
#[ignore = "needs the local chezmoi checkout at chezmoi/"]
fn every_chezmoi_script_renders_valid_gherkin() {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../chezmoi/internal/cmd/testdata/scripts");
    let mut checked = 0;
    for entry in std::fs::read_dir(&dir).expect("chezmoi checkout present") {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "txtar") {
            continue;
        }
        let name = path.file_stem().unwrap().to_str().unwrap();
        let script = txtar::parse(name, &std::fs::read_to_string(&path).unwrap());
        let text = feature::render(&script, name);
        let parsed = gherkin::Feature::parse(&text, gherkin::GherkinEnv::default())
            .unwrap_or_else(|e| panic!("{name}: invalid Gherkin: {e}"));
        assert_eq!(parsed.scenarios.len(), script.tests.len(), "{name}");
        assert!(
            parsed
                .scenarios
                .iter()
                .all(|s| s.steps.is_empty() && s.tags == ["skeleton"]),
            "{name}"
        );
        checked += 1;
    }
    assert!(
        checked > 300,
        "only {checked} scripts found in {}",
        dir.display()
    );
}
