//! One git file patch, as chezmoi's `DiffPatch` and go-git's `UnifiedEncoder` write it. The
//! rules come from chezmoi's output in a sandbox:
//!
//! - `new file mode`, `deleted file mode`, or `old mode`/`new mode` when the modes differ.
//! - When the contents differ: an `index` line, with the mode when both sides have the same
//!   one, then either a `Binary files` line or `---`/`+++` lines and the hunks.
//! - Hunks have three lines of context, and changes up to six unchanged lines apart share a
//!   hunk. A hunk header carries the line before the hunk.

use super::lines::{Edit, line_edits};
use super::sha1::git_blob_hash;
use super::sniff::is_binary;

/// Lines of context around each change (go-git's `DefaultContextLines`).
const CONTEXT: usize = 3;

const ZERO_HASH: &str = "0000000000000000000000000000000000000000";

/// One side of a patch: its contents (empty for a directory) and git mode (type bits, such as
/// `0o100000` for a file, with the permission bits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Side<'a> {
    pub data: &'a [u8],
    pub mode: u32,
}

/// The patch turning `from` into `to` at `path`; `None` is a side that does not exist.
#[must_use]
pub fn file_patch(path: &str, from: Option<Side<'_>>, to: Option<Side<'_>>) -> Vec<u8> {
    let mut out = format!("diff --git a/{path} b/{path}\n").into_bytes();
    match (from, to) {
        (None, Some(to)) => out.extend(format!("new file mode {:o}\n", to.mode).bytes()),
        (Some(from), None) => out.extend(format!("deleted file mode {:o}\n", from.mode).bytes()),
        (Some(from), Some(to)) if from.mode != to.mode => {
            out.extend(format!("old mode {:o}\nnew mode {:o}\n", from.mode, to.mode).bytes());
        }
        _ => {}
    }
    let hash = |side: Option<Side<'_>>| {
        side.map_or_else(|| ZERO_HASH.to_owned(), |s| git_blob_hash(s.data))
    };
    let (from_hash, to_hash) = (hash(from), hash(to));
    if from_hash == to_hash {
        return out;
    }
    out.extend(format!("index {from_hash}..{to_hash}").bytes());
    if let (Some(from), Some(to)) = (from, to)
        && from.mode == to.mode
    {
        out.extend(format!(" {:o}", from.mode).bytes());
    }
    out.push(b'\n');

    let from_name = from.map_or_else(|| "/dev/null".to_owned(), |_| format!("a/{path}"));
    let to_name = to.map_or_else(|| "/dev/null".to_owned(), |_| format!("b/{path}"));
    let (from_data, to_data) = (
        from.map_or(&[][..], |s| s.data),
        to.map_or(&[][..], |s| s.data),
    );
    if is_binary(from_data) || is_binary(to_data) {
        out.extend(format!("Binary files {from_name} and {to_name} differ\n").bytes());
        return out;
    }
    out.extend(format!("--- {from_name}\n+++ {to_name}\n").bytes());
    hunks(&line_edits(from_data, to_data), &mut out);
    out
}

/// Write the hunks of `edits` to `out`.
fn hunks(edits: &[Edit<'_>], out: &mut Vec<u8>) {
    let changes: Vec<usize> = edits
        .iter()
        .enumerate()
        .filter(|(_, e)| !matches!(e, Edit::Equal(_)))
        .map(|(i, _)| i)
        .collect();
    let mut groups: Vec<(usize, usize)> = Vec::new();
    for &i in &changes {
        match groups.last_mut() {
            Some((_, last)) if i - *last - 1 <= 2 * CONTEXT => *last = i,
            _ => groups.push((i, i)),
        }
    }
    for (first, last) in groups {
        let lo = first.saturating_sub(CONTEXT);
        let hi = (last + CONTEXT).min(edits.len() - 1);
        hunk(edits, lo, hi, out);
    }
}

/// The hunk of `edits[lo..=hi]`.
fn hunk(edits: &[Edit<'_>], lo: usize, hi: usize, out: &mut Vec<u8>) {
    let before = &edits[..lo];
    let from_before = before
        .iter()
        .filter(|e| !matches!(e, Edit::Insert(_)))
        .count();
    let to_before = before
        .iter()
        .filter(|e| !matches!(e, Edit::Delete(_)))
        .count();
    let lines = &edits[lo..=hi];
    let from_count = lines
        .iter()
        .filter(|e| !matches!(e, Edit::Insert(_)))
        .count();
    let to_count = lines
        .iter()
        .filter(|e| !matches!(e, Edit::Delete(_)))
        .count();
    out.extend(
        format!(
            "@@ -{} +{} @@",
            range(from_before, from_count),
            range(to_before, to_count)
        )
        .bytes(),
    );
    if let Some(Edit::Equal(line)) = before.last() {
        out.push(b' ');
        out.extend_from_slice(line.strip_suffix(b"\n").unwrap_or(line));
    }
    out.push(b'\n');
    for edit in lines {
        let (prefix, line) = match edit {
            Edit::Equal(line) => (b' ', line),
            Edit::Delete(line) => (b'-', line),
            Edit::Insert(line) => (b'+', line),
        };
        out.push(prefix);
        out.extend_from_slice(line);
        if !line.ends_with(b"\n") {
            out.extend_from_slice(b"\n\\ No newline at end of file\n");
        }
    }
}

/// `start,count` for a hunk after `before` lines, with `,1` left out; an empty range starts at
/// the line before it.
fn range(before: usize, count: usize) -> String {
    match count {
        0 => format!("{before},0"),
        1 => format!("{}", before + 1),
        _ => format!("{},{count}", before + 1),
    }
}
