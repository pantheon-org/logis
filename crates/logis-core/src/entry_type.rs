//! Entry types for `--include` / `--exclude`, ported from chezmoi's
//! `internal/chezmoi/entrytypeset.go` and `entrytypefilter.go`.

use std::fmt;
use std::str::FromStr;

use crate::source_name::{FileAttr, ScriptCondition, TargetType};
use crate::source_state::SourceEntry;

/// One kind of entry a command can be limited to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Dirs,
    Files,
    Remove,
    Scripts,
    Symlinks,
    Encrypted,
    Externals,
    Templates,
    /// Scripts that run every time (`run_` without `once_` or `onchange_`).
    Always,
}

impl EntryType {
    pub const ALL: [Self; 9] = [
        Self::Dirs,
        Self::Files,
        Self::Remove,
        Self::Scripts,
        Self::Symlinks,
        Self::Encrypted,
        Self::Externals,
        Self::Templates,
        Self::Always,
    ];

    /// The name used on the command line.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Dirs => "dirs",
            Self::Files => "files",
            Self::Remove => "remove",
            Self::Scripts => "scripts",
            Self::Symlinks => "symlinks",
            Self::Encrypted => "encrypted",
            Self::Externals => "externals",
            Self::Templates => "templates",
            Self::Always => "always",
        }
    }

    fn bit(self) -> u16 {
        1 << (self as u16)
    }

    fn from_name(name: &str) -> Option<EntryTypeSet> {
        if name == "all" {
            return Some(EntryTypeSet::all());
        }
        Self::ALL
            .into_iter()
            .find(|t| t.name() == name)
            .map(|t| EntryTypeSet { bits: t.bit() })
    }
}

/// A set of entry types, written as a comma-separated list: `dirs,files`, `all`, `none`, or
/// `noscripts` (everything but scripts).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryTypeSet {
    bits: u16,
}

impl EntryTypeSet {
    #[must_use]
    pub fn all() -> Self {
        EntryType::ALL.into_iter().collect()
    }

    #[must_use]
    pub fn none() -> Self {
        Self { bits: 0 }
    }

    #[must_use]
    pub fn without(self, entry_type: EntryType) -> Self {
        Self {
            bits: self.bits & !entry_type.bit(),
        }
    }

    fn has(self, entry_type: EntryType) -> bool {
        self.bits & entry_type.bit() != 0
    }

    /// Whether `entry` is of any type in the set.
    #[must_use]
    pub fn contains(&self, entry: &SourceEntry) -> bool {
        match entry {
            SourceEntry::Dir { .. } => self.has(EntryType::Dirs),
            SourceEntry::File { attr, .. } => self.contains_file(attr),
            SourceEntry::Absent { .. } => self.has(EntryType::Remove),
        }
    }

    fn contains_file(self, attr: &FileAttr) -> bool {
        let by_type = match attr.target_type {
            TargetType::Create | TargetType::File | TargetType::Modify => EntryType::Files,
            TargetType::Remove => EntryType::Remove,
            TargetType::Script => EntryType::Scripts,
            TargetType::Symlink => EntryType::Symlinks,
        };
        self.has(by_type)
            || (attr.encrypted && self.has(EntryType::Encrypted))
            || (attr.template && self.has(EntryType::Templates))
            || (attr.condition == ScriptCondition::Always && self.has(EntryType::Always))
    }
}

impl FromIterator<EntryType> for EntryTypeSet {
    fn from_iter<I: IntoIterator<Item = EntryType>>(iter: I) -> Self {
        Self {
            bits: iter.into_iter().fold(0, |bits, t| bits | t.bit()),
        }
    }
}

impl FromStr for EntryTypeSet {
    type Err = String;

    /// A list starting with an exclusion (`noscripts`) starts from `all`, as in chezmoi.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "none" {
            return Ok(Self::none());
        }
        let mut set = Self::none();
        for (i, element) in s.split(',').enumerate() {
            if element.is_empty() {
                continue;
            }
            let (name, exclude) = element
                .strip_prefix("no")
                .map_or((element, false), |rest| (rest, true));
            let types =
                EntryType::from_name(name).ok_or_else(|| format!("{name}: unknown entry type"))?;
            if i == 0 && exclude {
                set = Self::all();
            }
            set.bits = if exclude {
                set.bits & !types.bits
            } else {
                set.bits | types.bits
            };
        }
        Ok(set)
    }
}

impl fmt::Display for EntryTypeSet {
    /// `all`, `none`, or the member names in alphabetical order, as chezmoi prints them.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Self::all() {
            return f.write_str("all");
        }
        if *self == Self::none() {
            return f.write_str("none");
        }
        let mut names: Vec<&str> = EntryType::ALL
            .into_iter()
            .filter(|t| self.has(*t))
            .map(EntryType::name)
            .collect();
        names.sort_unstable();
        f.write_str(&names.join(","))
    }
}

/// Entries of an included type and of no excluded type. The default includes everything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryTypeFilter {
    pub include: EntryTypeSet,
    pub exclude: EntryTypeSet,
}

impl Default for EntryTypeFilter {
    fn default() -> Self {
        Self {
            include: EntryTypeSet::all(),
            exclude: EntryTypeSet::none(),
        }
    }
}

impl EntryTypeFilter {
    #[must_use]
    pub fn includes(&self, entry: &SourceEntry) -> bool {
        self.include.contains(entry) && !self.exclude.contains(entry)
    }

    /// Whether `entry_type` is included and not excluded (chezmoi's `IncludeEntryTypeBits`).
    #[must_use]
    pub fn includes_type(&self, entry_type: EntryType) -> bool {
        self.include.has(entry_type) && !self.exclude.has(entry_type)
    }
}
