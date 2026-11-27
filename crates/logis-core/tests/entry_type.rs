//! Port of chezmoi's `TestIncludeMaskSet` and `TestIncludeMaskStringSlice`
//! (`internal/chezmoi/entrytypeset_test.go`), plus which source entries each type matches
//! (`ContainsSourceStateEntry`).

use std::path::PathBuf;

use logis_core::entry_type::{EntryType, EntryTypeFilter, EntryTypeSet};
use logis_core::source_name::{DirAttr, FileAttr, ScriptCondition, TargetType};
use logis_core::source_state::SourceEntry;

fn set(types: &[EntryType]) -> EntryTypeSet {
    types.iter().copied().collect()
}

#[test]
fn include_mask_set() {
    use EntryType::{Dirs, Files, Scripts, Symlinks};
    for (input, expected) in [
        ("", EntryTypeSet::none()),
        ("none", EntryTypeSet::none()),
        ("dirs,files", set(&[Dirs, Files])),
        ("all", EntryTypeSet::all()),
        ("all,noscripts", EntryTypeSet::all().without(Scripts)),
        ("noscripts", EntryTypeSet::all().without(Scripts)),
        (
            "noscripts,nosymlinks",
            EntryTypeSet::all().without(Scripts).without(Symlinks),
        ),
        ("symlinks,,", set(&[Symlinks])),
    ] {
        assert_eq!(input.parse::<EntryTypeSet>(), Ok(expected), "{input:?}");
    }
    assert_eq!(
        "devices".parse::<EntryTypeSet>(),
        Err("devices: unknown entry type".to_owned())
    );
}

#[test]
fn include_mask_string_slice() {
    use EntryType::{Dirs, Encrypted, Externals, Files, Remove, Scripts, Symlinks};
    for (bits, expected) in [
        (EntryTypeSet::all(), "all"),
        (set(&[Dirs]), "dirs"),
        (set(&[Files]), "files"),
        (set(&[Remove]), "remove"),
        (set(&[Scripts]), "scripts"),
        (set(&[Symlinks]), "symlinks"),
        (set(&[Encrypted]), "encrypted"),
        (set(&[Externals]), "externals"),
        (EntryTypeSet::none(), "none"),
        (set(&[Dirs, Files]), "dirs,files"),
    ] {
        assert_eq!(bits.to_string(), expected);
    }
}

fn dir() -> SourceEntry {
    SourceEntry::Dir {
        source_path: PathBuf::from("/s/dir"),
        source_rel: "dir".into(),
        attr: DirAttr {
            target_name: "dir".into(),
            ..DirAttr::default()
        },
    }
}

fn file(attr: &FileAttr) -> SourceEntry {
    SourceEntry::File {
        source_path: PathBuf::from("/s/f"),
        source_rel: "f".into(),
        attr: FileAttr {
            target_name: "f".into(),
            ..attr.clone()
        },
    }
}

fn of_type(target_type: TargetType) -> FileAttr {
    FileAttr {
        target_type,
        ..FileAttr::default()
    }
}

/// Which single types match each kind of source entry (`ContainsSourceStateEntry`).
#[test]
fn source_entries_match_their_types() {
    use EntryType::{Always, Dirs, Encrypted, Files, Remove, Scripts, Symlinks, Templates};
    let script = file(&FileAttr {
        condition: ScriptCondition::Always,
        ..of_type(TargetType::Script)
    });
    let once_script = file(&FileAttr {
        condition: ScriptCondition::Once,
        ..of_type(TargetType::Script)
    });
    let encrypted = file(&FileAttr {
        encrypted: true,
        ..of_type(TargetType::File)
    });
    let template_symlink = file(&FileAttr {
        template: true,
        ..of_type(TargetType::Symlink)
    });
    for (entry, matching) in [
        (dir(), vec![Dirs]),
        (file(&of_type(TargetType::File)), vec![Files]),
        (file(&of_type(TargetType::Create)), vec![Files]),
        (file(&of_type(TargetType::Modify)), vec![Files]),
        (file(&of_type(TargetType::Remove)), vec![Remove]),
        (file(&of_type(TargetType::Symlink)), vec![Symlinks]),
        (script, vec![Scripts, Always]),
        (once_script, vec![Scripts]),
        (encrypted, vec![Files, Encrypted]),
        (template_symlink, vec![Symlinks, Templates]),
    ] {
        for entry_type in EntryType::ALL {
            assert_eq!(
                set(&[entry_type]).contains(&entry),
                matching.contains(&entry_type),
                "{entry_type:?} for {entry:?}"
            );
        }
    }
}

#[test]
fn filter_includes_minus_excludes() {
    use EntryType::{Encrypted, Files};
    let plain = file(&of_type(TargetType::File));
    let encrypted = file(&FileAttr {
        encrypted: true,
        ..of_type(TargetType::File)
    });
    let filter = EntryTypeFilter {
        include: set(&[Files]),
        exclude: set(&[Encrypted]),
    };
    assert!(filter.includes(&plain));
    assert!(!filter.includes(&encrypted));
    assert!(!filter.includes(&dir()));
    assert!(EntryTypeFilter::default().includes(&dir()));
}

/// chezmoi's `EntryTypeFilter.IncludeEntryTypeBits`, which the diff system uses per write.
#[test]
fn filter_includes_types_minus_excluded_types() {
    let filter = EntryTypeFilter {
        include: "files,dirs".parse().unwrap(),
        exclude: "dirs".parse().unwrap(),
    };
    assert!(filter.includes_type(EntryType::Files));
    assert!(!filter.includes_type(EntryType::Dirs));
    assert!(!filter.includes_type(EntryType::Symlinks));
    assert!(EntryTypeFilter::default().includes_type(EntryType::Remove));
}
