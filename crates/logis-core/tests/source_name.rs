//! Port of chezmoi's `internal/chezmoi/attr_test.go`. Each test keeps the Go test's name in
//! snake case; tables and generated combinations are the same.

use logis_core::source_name::{
    DirAttr, FileAttr, ParseError, ScriptCondition, ScriptOrder, TargetType,
};

/// Every combination of `n` booleans (the Go tests use `combinator.Generate` for this).
fn bool_combinations(n: u32) -> impl Iterator<Item = Vec<bool>> {
    (0..1u32 << n).map(move |bits| (0..n).map(|i| bits & (1 << i) != 0).collect())
}

#[test]
fn dir_attr() {
    let target_names = [
        ".dir",
        "dir.tmpl",
        "dir",
        "exact_dir",
        "empty_dir",
        "encrypted_dir",
        "executable_dir",
        "once_dir",
        "run_dir",
        "run_once_dir",
        "symlink_dir",
    ];
    let mut checked = 0;
    for target_name in target_names {
        for b in bool_combinations(5) {
            let dir_attr = DirAttr {
                target_name: target_name.to_owned(),
                exact: b[0],
                external: b[1],
                private: b[2],
                read_only: b[3],
                remove: b[4],
            };
            let source_name = dir_attr.source_name();
            let parsed =
                DirAttr::parse(&source_name).unwrap_or_else(|e| panic!("{source_name}: {e}"));
            assert_eq!(parsed, dir_attr, "{source_name}");
            assert_eq!(parsed.source_name(), source_name);
            checked += 1;
        }
    }
    assert_eq!(checked, 11 * 32);
}

#[test]
fn dir_attr_literal() {
    for (source_name, dir_attr) in [
        (
            "exact_dir",
            DirAttr {
                target_name: "dir".into(),
                exact: true,
                ..DirAttr::default()
            },
        ),
        (
            "literal_exact_dir",
            DirAttr {
                target_name: "exact_dir".into(),
                ..DirAttr::default()
            },
        ),
        (
            "literal_literal_dir",
            DirAttr {
                target_name: "literal_dir".into(),
                ..DirAttr::default()
            },
        ),
    ] {
        assert_eq!(dir_attr.source_name(), source_name);
        assert_eq!(DirAttr::parse(source_name), Ok(dir_attr), "{source_name}");
    }
}

const FILE_TARGET_NAMES: [&str; 13] = [
    ".name",
    "create_name",
    "dot_name",
    "exact_name",
    "literal_name",
    "literal_name",
    "modify_name",
    "name.literal",
    "name",
    "remove_",
    "run_name",
    "symlink_name",
    "template.tmpl",
];

/// The `FileAttr`s the Go test generates, type by type.
fn generated_file_attrs() -> Vec<FileAttr> {
    let mut attrs = Vec::new();
    let base = |target_type, target_name: &str| FileAttr {
        target_name: target_name.to_owned(),
        target_type,
        ..FileAttr::default()
    };
    for target_type in [TargetType::Create, TargetType::File] {
        for name in FILE_TARGET_NAMES {
            for b in bool_combinations(6) {
                attrs.push(FileAttr {
                    empty: b[0],
                    encrypted: b[1],
                    executable: b[2],
                    private: b[3],
                    read_only: b[4],
                    template: b[5],
                    ..base(target_type, name)
                });
            }
        }
    }
    for name in FILE_TARGET_NAMES {
        for b in bool_combinations(5) {
            attrs.push(FileAttr {
                encrypted: b[0],
                executable: b[1],
                private: b[2],
                read_only: b[3],
                template: b[4],
                ..base(TargetType::Modify, name)
            });
        }
    }
    for name in FILE_TARGET_NAMES {
        attrs.push(base(TargetType::Remove, name));
        attrs.push(base(TargetType::Symlink, name));
    }
    for condition in [
        ScriptCondition::Always,
        ScriptCondition::Once,
        ScriptCondition::OnChange,
    ] {
        for name in FILE_TARGET_NAMES {
            for order in [ScriptOrder::Before, ScriptOrder::During, ScriptOrder::After] {
                attrs.push(FileAttr {
                    condition,
                    order,
                    ..base(TargetType::Script, name)
                });
            }
        }
    }
    attrs
}

#[test]
fn file_attr() {
    let attrs = generated_file_attrs();
    assert_eq!(attrs.len(), 2 * 13 * 64 + 13 * 32 + 2 * 13 + 3 * 13 * 3);
    for file_attr in attrs {
        let source_name = file_attr.source_name("");
        let parsed =
            FileAttr::parse(&source_name, "").unwrap_or_else(|e| panic!("{source_name}: {e}"));
        assert_eq!(parsed, file_attr, "{source_name}");
        assert_eq!(parsed.source_name(""), source_name);
    }
}

#[test]
fn file_attr_encrypted_suffix() {
    for (source_name, expected_target_name) in [
        ("encrypted_file", "file"),
        ("encrypted_file.asc", "file"),
        ("file.asc", "file.asc"),
    ] {
        let parsed = FileAttr::parse(source_name, ".asc").expect(source_name);
        assert_eq!(parsed.target_name, expected_target_name, "{source_name}");
    }
}

#[test]
fn file_attr_literal() {
    let file = |target_name: &str| FileAttr {
        target_name: target_name.to_owned(),
        target_type: TargetType::File,
        ..FileAttr::default()
    };
    let script = |target_name: &str, condition| FileAttr {
        target_name: target_name.to_owned(),
        target_type: TargetType::Script,
        condition,
        ..FileAttr::default()
    };
    let template = |target_name: &str| FileAttr {
        template: true,
        ..file(target_name)
    };
    // (source name, expected attributes, canonical: source_name() gives the name back)
    for (source_name, file_attr, canonical) in [
        ("dot_file", file(".file"), true),
        ("literal_dot_file", file("dot_file"), true),
        ("literal_literal_file", file("literal_file"), true),
        (
            "run_once_script",
            script("script", ScriptCondition::Once),
            true,
        ),
        (
            "run_literal_once_script",
            script("once_script", ScriptCondition::Always),
            true,
        ),
        ("file.literal", file("file"), false),
        ("file.literal.literal", file("file.literal"), true),
        ("file.tmpl", template("file"), true),
        ("file.tmpl.literal", file("file.tmpl"), true),
        ("file.tmpl.literal.tmpl", template("file.tmpl"), true),
    ] {
        assert_eq!(
            FileAttr::parse(source_name, ""),
            Ok(file_attr.clone()),
            "{source_name}"
        );
        if canonical {
            assert_eq!(file_attr.source_name(""), source_name);
        }
    }
}

#[test]
fn file_attr_perm() {
    for (executable, private, read_only, expected) in [
        (false, false, false, 0o666),
        (true, false, false, 0o777),
        (false, true, false, 0o600),
        (true, true, false, 0o700),
        (false, false, true, 0o444),
        (true, false, true, 0o555),
        (false, true, true, 0o400),
        (true, true, true, 0o500),
    ] {
        let attr = FileAttr {
            executable,
            private,
            read_only,
            ..FileAttr::default()
        };
        assert_eq!(attr.perm(), expected, "{attr:?}");
    }
}

/// Not in the Go tests: `DirAttr.perm` is only exercised indirectly there.
#[test]
fn dir_attr_perm() {
    for (private, read_only, expected) in [
        (false, false, 0o777),
        (true, false, 0o700),
        (false, true, 0o555),
        (true, true, 0o500),
    ] {
        let attr = DirAttr {
            private,
            read_only,
            ..DirAttr::default()
        };
        assert_eq!(attr.perm(), expected, "{attr:?}");
    }
}

#[test]
fn invalid_dir_attr() {
    for name in [
        "dot_.",
        "dot_",
        "external_..",
        "external_dot_.",
        "literal_..",
        "literal_",
        "private_..",
        "private_dot_.",
        "remove_..",
        "remove_dot_.",
    ] {
        assert_eq!(
            DirAttr::parse(name),
            Err(ParseError::InvalidPath(name.to_owned()))
        );
    }
}

#[test]
fn invalid_file_attr() {
    for name in [
        "...literal",
        "...tmpl",
        "..literal",
        ".tmpl",
        "dot_.",
        "dot_",
        "encrypted_...age",
        "encrypted_.age",
        "executable_..",
        "executable_dot_.",
        "literal_..",
        "literal_.",
        "literal_",
        "private_..",
        "private_dot_.",
        "run_..",
        "run_after_..",
        "run_after_dot_.",
        "run_before_..",
        "run_before_dot_.",
        "run_dot_.",
        "run_once_..",
        "run_once_dot_.",
    ] {
        assert_eq!(
            FileAttr::parse(name, ".age"),
            Err(ParseError::InvalidPath(name.to_owned()))
        );
    }
}

/// Not in the Go tests: the empty-name errors and their messages, which chezmoi prints.
#[test]
fn empty_names_and_error_messages() {
    assert_eq!(DirAttr::parse(""), Err(ParseError::EmptyDirName));
    assert_eq!(FileAttr::parse("", ""), Err(ParseError::EmptyFileName));
    assert_eq!(ParseError::EmptyDirName.to_string(), "empty directory name");
    assert_eq!(ParseError::EmptyFileName.to_string(), "empty filename");
    assert_eq!(
        ParseError::InvalidPath("dot_".into()).to_string(),
        "dot_: invalid path"
    );
}

/// Divergence from chezmoi (ADR-007), not in the Go tests: directory targets starting
/// `remove_` or `external_` are protected with `literal_`. chezmoi writes `remove_foo`, which
/// then reads back as "remove `foo`". Existing source names still parse as chezmoi does.
#[test]
fn attribute_like_dir_names_are_protected() {
    for (target_name, source_name) in [
        ("remove_foo", "literal_remove_foo"),
        ("external_bar", "literal_external_bar"),
    ] {
        let attr = DirAttr {
            target_name: target_name.into(),
            ..DirAttr::default()
        };
        assert_eq!(attr.source_name(), source_name);
        assert_eq!(DirAttr::parse(source_name), Ok(attr));
    }
    let parsed = DirAttr::parse("remove_foo").unwrap();
    assert_eq!((parsed.target_name.as_str(), parsed.remove), ("foo", true));
}

/// Divergence from chezmoi (ADR-007), not in the Go tests: targets starting `onchange_` are
/// protected with `literal_`, as `once_` already is. chezmoi writes the script `onchange_x` as
/// `run_onchange_x`, which reads back as the onchange script `x`.
#[test]
fn onchange_names_are_protected() {
    let script = FileAttr {
        target_name: "onchange_x".into(),
        target_type: TargetType::Script,
        condition: ScriptCondition::Always,
        ..FileAttr::default()
    };
    assert_eq!(script.source_name(""), "run_literal_onchange_x");
    assert_eq!(FileAttr::parse("run_literal_onchange_x", ""), Ok(script));
    let parsed = FileAttr::parse("run_onchange_x", "").unwrap();
    assert_eq!(
        (parsed.target_name.as_str(), parsed.condition),
        ("x", ScriptCondition::OnChange)
    );
}
