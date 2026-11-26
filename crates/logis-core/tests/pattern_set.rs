//! Port of chezmoi's `TestPatternSet` (`internal/chezmoi/patternset_test.go`). Case names are
//! the Go names; extra cases say where they come from. `TestPatternSetGlob` belongs with the
//! remove file and is not ported yet.

use logis_core::pattern_set::{Include, PatternMatch, PatternSet};

fn set(patterns: &[(&str, Include)]) -> PatternSet {
    let mut set = PatternSet::default();
    for (pattern, include) in patterns {
        set.add(pattern, *include).unwrap();
    }
    set
}

fn assert_matches(set: &PatternSet, cases: &[(&str, PatternMatch)]) {
    for (name, expected) in cases {
        assert_eq!(set.matches(name), *expected, "{name}");
    }
}

#[test]
fn empty() {
    assert_matches(&PatternSet::default(), &[("foo", PatternMatch::Unknown)]);
}

#[test]
fn exact() {
    assert_matches(
        &set(&[("foo", Include::Include)]),
        &[
            ("foo", PatternMatch::Include),
            ("bar", PatternMatch::Exclude),
        ],
    );
}

#[test]
fn wildcard() {
    assert_matches(
        &set(&[("b*", Include::Include)]),
        &[
            ("foo", PatternMatch::Exclude),
            ("bar", PatternMatch::Include),
            ("baz", PatternMatch::Include),
        ],
    );
}

#[test]
fn exclude() {
    assert_matches(
        &set(&[("b*", Include::Include), ("baz", Include::Exclude)]),
        &[
            ("foo", PatternMatch::Unknown),
            ("bar", PatternMatch::Include),
            ("baz", PatternMatch::Exclude),
        ],
    );
}

#[test]
fn doublestar() {
    assert_matches(
        &set(&[("**/foo", Include::Include)]),
        &[
            ("foo", PatternMatch::Include),
            ("bar/foo", PatternMatch::Include),
            ("baz/bar/foo", PatternMatch::Include),
        ],
    );
}

/// Not a Go case: with only excluded patterns, everything else matches as included.
#[test]
fn only_exclude() {
    assert_matches(
        &set(&[("foo", Include::Exclude)]),
        &[
            ("foo", PatternMatch::Exclude),
            ("bar", PatternMatch::Include),
        ],
    );
}

/// Not a Go case: doublestar's `*` does not cross `/`, but matches a leading `.`.
#[test]
fn star_stays_in_one_component() {
    assert_matches(
        &set(&[("*/*.txt", Include::Include)]),
        &[
            ("dir/file.txt", PatternMatch::Include),
            (".dir/.file.txt", PatternMatch::Include),
            ("file.txt", PatternMatch::Exclude),
            ("dir/subdir/file.txt", PatternMatch::Exclude),
        ],
    );
}

/// Not a Go case: doublestar's alternation, classes and escapes.
#[test]
fn alternation_classes_and_escapes() {
    assert_matches(
        &set(&[
            ("{a,b}.txt", Include::Include),
            ("[xy]", Include::Include),
            ("q\\*", Include::Include),
        ]),
        &[
            ("a.txt", PatternMatch::Include),
            ("b.txt", PatternMatch::Include),
            ("c.txt", PatternMatch::Exclude),
            ("x", PatternMatch::Include),
            ("z", PatternMatch::Exclude),
            ("q*", PatternMatch::Include),
            ("qq", PatternMatch::Exclude),
        ],
    );
}

/// Not a Go case: `PatternSet.Add` rejects patterns that doublestar cannot parse, with
/// chezmoi's message.
#[test]
fn invalid_patterns_are_errors() {
    for pattern in ["[abc", "{a,b"] {
        let err = PatternSet::default()
            .add(pattern, Include::Include)
            .unwrap_err();
        assert_eq!(err.to_string(), format!("{pattern}: invalid pattern"));
    }
}

/// Not a Go case: adding a pattern twice is the same as adding it once.
#[test]
fn duplicates_are_one_pattern() {
    let set = set(&[("foo", Include::Include), ("foo", Include::Include)]);
    assert_eq!(set.len(), 1);
}
