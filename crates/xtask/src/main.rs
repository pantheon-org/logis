//! `cargo xtask <task>`.
//!
//! Tasks:
//! - `txtar2feature <scripts-dir> <out-dir> [--force]`: write one skeleton `.feature` per
//!   `.txtar` script. Existing files are left alone unless `--force` is given, so hand-written
//!   steps are never overwritten.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use xtask::{feature, txtar};

const USAGE: &str = "usage: cargo xtask txtar2feature <scripts-dir> <out-dir> [--force]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) != Some("txtar2feature") {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    }
    match txtar2feature(&args[1..]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("txtar2feature: {err}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, Default)]
struct Totals {
    scripts: usize,
    scenarios: usize,
    written: usize,
    skipped: usize,
}

fn txtar2feature(args: &[String]) -> Result<(), String> {
    let force = args.iter().any(|a| a == "--force");
    let paths: Vec<&String> = args.iter().filter(|a| *a != "--force").collect();
    let [scripts, out] = paths.as_slice() else {
        return Err(USAGE.to_owned());
    };
    let (scripts, out) = (Path::new(scripts.as_str()), Path::new(out.as_str()));
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;

    let mut totals = Totals::default();
    for path in txtar_files(scripts)? {
        convert_one(&path, out, force, &mut totals)?;
    }
    eprintln!(
        "{} scripts, {} scenarios: wrote {}, skipped {} existing",
        totals.scripts, totals.scenarios, totals.written, totals.skipped
    );
    Ok(())
}

fn txtar_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "txtar"))
        .collect();
    paths.sort();
    Ok(paths)
}

fn convert_one(path: &Path, out: &Path, force: bool, totals: &mut Totals) -> Result<(), String> {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("{}: not a UTF-8 file name", path.display()))?;
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let script = txtar::parse(name, &text);
    totals.scripts += 1;
    totals.scenarios += script.tests.len();

    let target = out.join(format!("{name}.feature"));
    if target.exists() && !force {
        totals.skipped += 1;
        return Ok(());
    }
    let source = format!("internal/cmd/testdata/scripts/{name}.txtar");
    std::fs::write(&target, feature::render(&script, &source))
        .map_err(|e| format!("{}: {e}", target.display()))?;
    totals.written += 1;
    Ok(())
}
