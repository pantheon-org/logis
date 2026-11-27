//! The pieces of chezmoi's diff output (`internal/chezmoi/diff.go` and go-git's unified
//! encoder): git blob hashes, the content sniffer, the line diff and file patches. Expected
//! values come from running chezmoi 2.74.1 in a sandbox, and from git itself for hashes.

use logis_core::diff::{Edit, Side, file_patch, git_blob_hash, is_binary, line_edits};

const DIR: u32 = 0o040_000;
const FILE: u32 = 0o100_000;

fn patch(from: Option<Side<'_>>, to: Option<Side<'_>>) -> String {
    String::from_utf8(file_patch(".file", from, to)).unwrap()
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "file_patch takes an Option per side; tests read better without Some(...)"
)]
fn side(data: &str, mode: u32) -> Option<Side<'_>> {
    Some(Side {
        data: data.as_bytes(),
        mode,
    })
}

#[test]
fn blob_hashes_are_gits() {
    for (data, hash) in [
        ("", "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"),
        ("x\n", "587be6b4c3f93f93c489c0111bba5596147a26cb"),
        (
            "# contents of .file\n",
            "8a52cb9ce9551221716a53786ad74104c5902362",
        ),
    ] {
        assert_eq!(git_blob_hash(data.as_bytes()), hash, "{data:?}");
    }
    // Longer than one SHA-1 block, with the length crossing a block boundary.
    let long = "line\n".repeat(30);
    assert_eq!(git_blob_hash(long.as_bytes()).len(), 40);
}

#[test]
fn sniffing_follows_go_detect_content_type() {
    assert!(!is_binary(b""));
    assert!(!is_binary(b"plain text\n"));
    assert!(!is_binary("unicode \u{e9}\n".as_bytes()));
    assert!(!is_binary(b"<html><body>x</body></html>"));
    assert!(is_binary(b"\x00\x01bin"));
    assert!(is_binary(b"%PDF-1.4 text\n"));
    assert!(is_binary(b"GIF89a"));
    assert!(is_binary(b"\x89PNG\r\n\x1a\n"));
    assert!(is_binary(b"PK\x03\x04"));
}

#[test]
fn line_edits_are_minimal_and_delete_before_insert() {
    use Edit::{Delete, Equal, Insert};
    assert_eq!(
        line_edits(b"x\ny\nx\nx\n", b"x\nx\nx\n"),
        [
            Equal(&b"x\n"[..]),
            Delete(&b"y\n"[..]),
            Equal(&b"x\n"[..]),
            Equal(&b"x\n"[..]),
        ]
    );
    assert_eq!(
        line_edits(b"a\nb\n", b"c\nd\n"),
        [
            Delete(&b"a\n"[..]),
            Delete(&b"b\n"[..]),
            Insert(&b"c\n"[..]),
            Insert(&b"d\n"[..]),
        ]
    );
    assert_eq!(
        line_edits(b"a\nb", b"a\nb\n"),
        [Equal(&b"a\n"[..]), Delete(&b"b"[..]), Insert(&b"b\n"[..])]
    );
    assert!(line_edits(b"", b"").is_empty());
}

#[test]
fn a_new_file() {
    assert_eq!(
        patch(None, side("# contents of .file\n", FILE | 0o644)),
        "diff --git a/.file b/.file\n\
         new file mode 100644\n\
         index 0000000000000000000000000000000000000000..8a52cb9ce9551221716a53786ad74104c5902362\n\
         --- /dev/null\n\
         +++ b/.file\n\
         @@ -0,0 +1 @@\n\
         +# contents of .file\n"
    );
}

#[test]
fn a_deleted_directory_has_no_hunks() {
    assert_eq!(
        patch(side("", DIR | 0o755), None),
        "diff --git a/.file b/.file\n\
         deleted file mode 40755\n\
         index e69de29bb2d1d6434b8b29ae775ad8c2e48c5391..0000000000000000000000000000000000000000\n\
         --- a/.file\n\
         +++ /dev/null\n"
    );
}

#[test]
fn a_mode_change_alone_has_no_index() {
    assert_eq!(
        patch(side("x\n", FILE | 0o777), side("x\n", FILE | 0o644)),
        "diff --git a/.file b/.file\nold mode 100777\nnew mode 100644\n"
    );
}

#[test]
fn a_mode_and_contents_change() {
    assert_eq!(
        patch(side("other\n", FILE | 0o644), side("same\n", FILE | 0o755)),
        "diff --git a/.file b/.file\n\
         old mode 100644\n\
         new mode 100755\n\
         index e45c9c2666d44e0327c1f9c239a74c508336053e..1275430f1765c63e539cb0452565563bd6aef6a6\n\
         --- a/.file\n\
         +++ b/.file\n\
         @@ -1 +1 @@\n\
         -other\n\
         +same\n"
    );
}

#[test]
fn missing_final_newlines_are_marked() {
    assert_eq!(
        patch(side("a\nc", FILE | 0o644), side("a\nb", FILE | 0o644)),
        "diff --git a/.file b/.file\n\
         index 817f660e4423f7df2dfc7d4bff0e01b2092a8ce9..0a207c060e61f3b88eaee0a8cd0696f46fb155eb 100644\n\
         --- a/.file\n\
         +++ b/.file\n\
         @@ -1,2 +1,2 @@\n \
         a\n\
         -c\n\
         \\ No newline at end of file\n\
         +b\n\
         \\ No newline at end of file\n"
    );
}

#[test]
fn binary_files_are_not_shown_by_line() {
    assert_eq!(
        String::from_utf8(file_patch(
            ".bin",
            None,
            Some(Side {
                data: b"\x00\x01bin",
                mode: FILE | 0o644
            })
        ))
        .unwrap(),
        "diff --git a/.bin b/.bin\n\
         new file mode 100644\n\
         index 0000000000000000000000000000000000000000..88768efdf77ec78c9a995f94881793be6a41752b\n\
         Binary files /dev/null and b/.bin differ\n"
    );
}

fn numbered(changes: &[(usize, &str)]) -> String {
    (1..=20)
        .map(|n| {
            changes
                .iter()
                .find(|(line, _)| *line == n)
                .map_or_else(|| format!("line {n}\n"), |(_, text)| format!("{text}\n"))
        })
        .collect()
}

/// Hunk headers carry the line before the hunk; changes up to six unchanged lines apart share
/// a hunk.
#[test]
fn hunks_merge_within_twice_the_context() {
    let to = numbered(&[]);
    let near = patch(
        side(&numbered(&[(5, "LINE 5"), (12, "LINE 12")]), FILE | 0o644),
        side(&to, FILE | 0o644),
    );
    assert_eq!(near.matches("\n@@ ").count(), 1, "{near}");
    assert!(near.contains("\n@@ -2,14 +2,14 @@ line 1\n"), "{near}");

    let far = patch(
        side(&numbered(&[(5, "LINE 5"), (13, "LINE 13")]), FILE | 0o644),
        side(&to, FILE | 0o644),
    );
    assert!(far.contains("\n@@ -2,7 +2,7 @@ line 1\n"), "{far}");
    assert!(
        far.contains("\n@@ -10,7 +10,7 @@ line 9\n line 10\n"),
        "{far}"
    );
    assert!(far.ends_with(" line 16\n"), "{far}");
}

#[test]
fn a_hunk_at_the_start_has_no_section_text() {
    let out = patch(
        side(&numbered(&[(3, "LINE 3")]), FILE | 0o644),
        side(&numbered(&[]), FILE | 0o644),
    );
    assert!(out.contains("\n@@ -1,6 +1,6 @@\n line 1\n"), "{out}");
}
