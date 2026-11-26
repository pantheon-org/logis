//! Sets of include and exclude glob patterns over `/`-separated target paths, ported from
//! chezmoi's `PatternSet` (`internal/chezmoi/patternset.go`).
//!
//! chezmoi matches with doublestar v4: `*`, `?` and classes stay within one path component,
//! `**` spans components, and `{a,b}` and `\` escapes are supported. `globset` with a literal
//! separator and backslash escapes gives the same semantics for the patterns chezmoi's tests
//! and scripts use. Known gap (verify): doublestar accepts nested alternation such as
//! `{a,{b,c}}`, which `globset` rejects as an invalid pattern.

use std::collections::BTreeMap;
use std::fmt;

use globset::{GlobBuilder, GlobMatcher};

/// Whether a pattern includes or excludes the paths it matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Include {
    Include,
    Exclude,
}

/// How a path matches a set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternMatch {
    Include,
    Unknown,
    Exclude,
}

/// A pattern that is not a valid glob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidPattern(pub String);

impl fmt::Display for InvalidPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: invalid pattern", self.0)
    }
}

impl std::error::Error for InvalidPattern {}

#[derive(Debug, Clone, Default)]
pub struct PatternSet {
    include: BTreeMap<String, GlobMatcher>,
    exclude: BTreeMap<String, GlobMatcher>,
}

impl PatternSet {
    /// Add `pattern`. Adding a pattern that is already in the set changes nothing.
    pub fn add(&mut self, pattern: &str, include: Include) -> Result<(), InvalidPattern> {
        let matcher = GlobBuilder::new(pattern)
            .literal_separator(true)
            .backslash_escape(true)
            .build()
            .map_err(|_| InvalidPattern(pattern.to_owned()))?
            .compile_matcher();
        let patterns = match include {
            Include::Include => &mut self.include,
            Include::Exclude => &mut self.exclude,
        };
        patterns.insert(pattern.to_owned(), matcher);
        Ok(())
    }

    /// Exclude patterns win over include patterns. A path that matches neither is excluded
    /// when the set has only include patterns, included when it has only exclude patterns,
    /// and unknown otherwise.
    #[must_use]
    pub fn matches(&self, name: &str) -> PatternMatch {
        if self.exclude.values().any(|m| m.is_match(name)) {
            return PatternMatch::Exclude;
        }
        if self.include.values().any(|m| m.is_match(name)) {
            return PatternMatch::Include;
        }
        match (self.include.is_empty(), self.exclude.is_empty()) {
            (false, true) => PatternMatch::Exclude,
            (true, false) => PatternMatch::Include,
            _ => PatternMatch::Unknown,
        }
    }

    /// The number of distinct patterns.
    #[must_use]
    pub fn len(&self) -> usize {
        self.include.len() + self.exclude.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
