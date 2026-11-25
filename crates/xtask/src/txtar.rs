//! Parse a chezmoi txtar testscript into the parts a BDD skeleton needs.
//!
//! A testscript is a command section followed by an archive of files (`-- name --` markers).
//! Inside the command section, each `# test ...` comment starts one test; the lines before the
//! first one are setup.

/// One `# test ...` block of a testscript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Test {
    /// The comment text, with `# test`, an optional `that` and a leading `chezmoi ` removed.
    pub title: String,
    /// The script lines of the test, without the title comment or trailing blank lines.
    pub lines: Vec<String>,
}

/// A parsed testscript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    /// File stem, for example `applychmod_unix`.
    pub name: String,
    /// Tags derived from skip conditions and the file name, for example `unix`, `needs-git`.
    pub tags: Vec<String>,
    /// Lines before the first test, without blank lines.
    pub setup: Vec<String>,
    pub tests: Vec<Test>,
    /// Names of the files in the archive section.
    pub files: Vec<String>,
}

/// Parse testscript `text` from the file with stem `name`.
#[must_use]
pub fn parse(name: &str, text: &str) -> Script {
    let (commands, files) = split_archive(text);
    let (setup, mut tests) = split_tests(&commands);
    if tests.is_empty() {
        tests.push(Test {
            title: format!("{name} (no test comments in source)"),
            lines: trim_trailing_blank(setup.clone()),
        });
        return Script {
            name: name.to_owned(),
            tags: tags(name, &setup),
            setup: Vec::new(),
            tests,
            files,
        };
    }
    let tags = tags(name, &setup);
    let setup = setup.into_iter().filter(|l| !l.trim().is_empty()).collect();
    Script {
        name: name.to_owned(),
        tags,
        setup,
        tests,
        files,
    }
}

/// The txtar archive marker: `-- name --`.
fn archive_file_name(line: &str) -> Option<&str> {
    let name = line.strip_prefix("-- ")?.strip_suffix(" --")?.trim();
    (!name.is_empty()).then_some(name)
}

/// Split into command lines and the names of the archived files.
fn split_archive(text: &str) -> (Vec<&str>, Vec<String>) {
    let commands = text
        .lines()
        .take_while(|l| archive_file_name(l).is_none())
        .collect();
    let files = text
        .lines()
        .filter_map(archive_file_name)
        .map(str::to_owned)
        .collect();
    (commands, files)
}

/// The title of a `# test ...` comment, or `None` if `line` is not one.
fn test_title(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("# test")?;
    if !(rest.is_empty() || rest.starts_with(' ')) {
        return None; // e.g. `# testing`
    }
    let rest = rest.trim();
    let rest = rest.strip_prefix("that ").unwrap_or(rest);
    Some(rest.strip_prefix("chezmoi ").unwrap_or(rest))
}

/// Split command lines into setup (before the first test) and tests.
fn split_tests(commands: &[&str]) -> (Vec<String>, Vec<Test>) {
    let mut setup = Vec::new();
    let mut tests: Vec<Test> = Vec::new();
    for line in commands {
        if let Some(title) = test_title(line) {
            let title = if title.is_empty() {
                format!("untitled test {}", tests.len() + 1)
            } else {
                title.to_owned()
            };
            tests.push(Test {
                title,
                lines: Vec::new(),
            });
        } else if let Some(test) = tests.last_mut() {
            test.lines.push((*line).to_owned());
        } else {
            setup.push((*line).to_owned());
        }
    }
    for test in &mut tests {
        test.lines = trim_trailing_blank(std::mem::take(&mut test.lines));
    }
    (setup, tests)
}

fn trim_trailing_blank(mut lines: Vec<String>) -> Vec<String> {
    while lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }
    lines
}

/// Tags from top-level `[cond] skip` lines and the file name suffix, in order, deduplicated.
fn tags(name: &str, setup: &[String]) -> Vec<String> {
    let mut tags: Vec<String> = setup.iter().filter_map(|l| skip_tag(l)).collect();
    let suffix = [
        ("_unix", "unix"),
        ("_windows", "windows"),
        ("_darwin", "macos"),
    ]
    .into_iter()
    .find_map(|(suffix, tag)| name.ends_with(suffix).then_some(tag));
    tags.extend(suffix.map(str::to_owned));
    let mut seen = std::collections::HashSet::new();
    tags.retain(|t| seen.insert(t.clone()));
    tags
}

/// `[windows] skip ...` means the script runs everywhere except Windows, so it is `@unix`.
fn skip_tag(line: &str) -> Option<String> {
    let rest = line.strip_prefix('[')?;
    let (cond, after) = rest.split_once(']')?;
    if !after.trim_start().starts_with("skip") {
        return None;
    }
    let tag = match cond {
        "windows" => "unix".to_owned(),
        "unix" | "!windows" => "windows".to_owned(),
        "!darwin" => "macos".to_owned(),
        "!linux" => "linux".to_owned(),
        other => format!("needs-{}", slug(other.trim_start_matches('!'))),
    };
    Some(tag)
}

fn slug(text: &str) -> String {
    let mapped: String = text
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = mapped.trim_matches('-');
    let parts: Vec<&str> = trimmed.split('-').filter(|p| !p.is_empty()).collect();
    let joined = parts.join("-");
    joined.strip_prefix("exec-").unwrap_or(&joined).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles(script: &Script) -> Vec<&str> {
        script.tests.iter().map(|t| t.title.as_str()).collect()
    }

    #[test]
    fn splits_tests_on_test_comments() {
        let script = parse(
            "apply",
            "mksourcedir\n\n# test that chezmoi apply works\nexec chezmoi apply\n\n# test diff\nexec chezmoi diff\n",
        );
        assert_eq!(script.setup, ["mksourcedir"]);
        assert_eq!(titles(&script), ["apply works", "diff"]);
        assert_eq!(script.tests[0].lines, ["exec chezmoi apply"]);
        assert_eq!(script.tests[1].lines, ["exec chezmoi diff"]);
    }

    #[test]
    fn keeps_blank_lines_inside_a_test_but_trims_trailing_ones() {
        let script = parse("t", "# test x\na\n\nb\n\n\n");
        assert_eq!(script.tests[0].lines, ["a", "", "b"]);
    }

    #[test]
    fn ignores_comments_inside_the_archive() {
        let script = parse(
            "t",
            "# test real\nexec chezmoi apply\n-- home/user/.local/share/chezmoi/run_x.sh --\n#!/bin/sh\n# test not a test\n-- golden/.file --\n# contents\n",
        );
        assert_eq!(titles(&script), ["real"]);
        assert_eq!(
            script.files,
            ["home/user/.local/share/chezmoi/run_x.sh", "golden/.file"]
        );
    }

    #[test]
    fn script_without_test_comments_has_one_untitled_test() {
        let script = parse("version", "exec chezmoi --version\nstdout chezmoi\n");
        assert!(script.setup.is_empty());
        assert_eq!(titles(&script), ["version (no test comments in source)"]);
        assert_eq!(
            script.tests[0].lines,
            ["exec chezmoi --version", "stdout chezmoi"]
        );
    }

    #[test]
    fn bare_test_comment_gets_a_numbered_title() {
        let script = parse("t", "# test\na\n# test\nb\n");
        assert_eq!(titles(&script), ["untitled test 1", "untitled test 2"]);
    }

    #[test]
    fn top_level_skip_conditions_become_tags() {
        let script = parse(
            "t",
            "[windows] skip 'UNIX only'\n[!exec:git] skip 'git not found in $PATH'\n[!umask:022] skip\n# test a\nb\n",
        );
        assert_eq!(script.tags, ["unix", "needs-git", "needs-umask-022"]);
        // The skip lines stay in setup as provenance.
        assert_eq!(script.setup.len(), 3);
    }

    #[test]
    fn os_skip_conditions_map_to_os_tags() {
        let tags = |cond: &str| parse("t", &format!("{cond} skip\n# test a\nb\n")).tags;
        assert_eq!(tags("[windows]"), ["unix"]);
        assert_eq!(tags("[unix]"), ["windows"]);
        assert_eq!(tags("[!windows]"), ["windows"]);
        assert_eq!(tags("[!darwin]"), ["macos"]);
        assert_eq!(tags("[!linux]"), ["linux"]);
    }

    #[test]
    fn file_name_suffix_becomes_an_os_tag_without_duplicates() {
        assert_eq!(parse("cd_windows", "# test a\nb\n").tags, ["windows"]);
        assert_eq!(parse("mackup_darwin", "# test a\nb\n").tags, ["macos"]);
        assert_eq!(
            parse("modify_unix", "[windows] skip\n# test a\nb\n").tags,
            ["unix"]
        );
    }

    #[test]
    fn skip_lines_inside_a_test_are_kept_and_not_tagged() {
        let script = parse("t", "# test a\n[windows] skip\nb\n");
        assert!(script.tags.is_empty());
        assert_eq!(script.tests[0].lines, ["[windows] skip", "b"]);
    }
}
