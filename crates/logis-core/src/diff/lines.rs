//! A minimal line diff: Myers' algorithm (1986), standing in for chezmoi's
//! `znkr.io/diff/textdiff` with `diff.Minimal()`. Lines keep their `\n`, so a last line
//! without one differs from the same line with one.
//!
//! Both find a shortest edit script; where several exist they may choose differently (for
//! example which of two equal lines is kept). Within a run of changes, deletions come before
//! insertions, as in chezmoi's output (verify for repeated-line edge cases).

/// One line of the edit script, borrowed from the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edit<'a> {
    Equal(&'a [u8]),
    Delete(&'a [u8]),
    Insert(&'a [u8]),
}

/// Split `data` into lines, each with its `\n` except perhaps the last.
fn lines(data: &[u8]) -> Vec<&[u8]> {
    data.split_inclusive(|&b| b == b'\n').collect()
}

/// A shortest edit script turning `from` into `to`.
#[must_use]
pub fn line_edits<'a>(from: &'a [u8], to: &'a [u8]) -> Vec<Edit<'a>> {
    let (a, b) = (lines(from), lines(to));
    if a.is_empty() && b.is_empty() {
        return Vec::new();
    }
    let trace = forward(&a, &b);
    backtrack(&a, &b, &trace)
}

/// The furthest-reaching x for each diagonal k (offset by `max`), after each number of edits.
#[allow(
    clippy::many_single_char_names,
    reason = "the names in Myers (1986), so the code can be checked against the paper"
)]
fn forward(a: &[&[u8]], b: &[&[u8]]) -> Vec<Vec<usize>> {
    let (n, m) = (a.len(), b.len());
    let max = n + m;
    let mut v = vec![0usize; 2 * max + 2];
    let mut trace = Vec::new();
    for d in 0..=max {
        trace.push(v.clone());
        for k in diagonals(d) {
            let idx = index(k, max);
            // Prefer a deletion (moving right) unless an insertion reaches further.
            let mut x = if k == -as_isize(d) || (k != as_isize(d) && v[idx - 1] < v[idx + 1]) {
                v[idx + 1]
            } else {
                v[idx - 1] + 1
            };
            let mut y = x.checked_add_signed(-k).expect("y on the grid");
            while x < n && y < m && a[x] == b[y] {
                x += 1;
                y += 1;
            }
            v[idx] = x;
            if x >= n && y >= m {
                return trace;
            }
        }
    }
    trace
}

#[allow(
    clippy::many_single_char_names,
    reason = "the names in Myers (1986), so the code can be checked against the paper"
)]
fn backtrack<'a>(a: &[&'a [u8]], b: &[&'a [u8]], trace: &[Vec<usize>]) -> Vec<Edit<'a>> {
    let max = a.len() + b.len();
    let (mut x, mut y) = (a.len(), b.len());
    let mut edits = Vec::new();
    for (d, v) in trace.iter().enumerate().rev() {
        let k = as_isize(x) - as_isize(y);
        let idx = index(k, max);
        let down = k == -as_isize(d) || (k != as_isize(d) && v[idx - 1] < v[idx + 1]);
        let prev_k = if down { k + 1 } else { k - 1 };
        let prev_x = v[index(prev_k, max)];
        let prev_y = prev_x.checked_add_signed(-prev_k).unwrap_or(0);
        while x > prev_x && y > prev_y {
            x -= 1;
            y -= 1;
            edits.push(Edit::Equal(a[x]));
        }
        if d == 0 {
            break;
        }
        if down {
            y -= 1;
            edits.push(Edit::Insert(b[y]));
        } else {
            x -= 1;
            edits.push(Edit::Delete(a[x]));
        }
    }
    edits.reverse();
    edits
}

/// Diagonals -d, -d+2, ..., d.
fn diagonals(d: usize) -> impl Iterator<Item = isize> {
    let d = as_isize(d);
    (-d..=d).step_by(2)
}

fn index(k: isize, max: usize) -> usize {
    (as_isize(max) + k + 1)
        .try_into()
        .expect("diagonal in range")
}

fn as_isize(n: usize) -> isize {
    isize::try_from(n).expect("fits in isize")
}
