//! Round-trip properties of the source-name grammar beyond chezmoi's fixed tables: for any
//! valid target name and any attributes the target type allows, parsing the source name gives
//! the attributes back, and the source name is canonical.

use logis_core::source_name::{DirAttr, FileAttr, ScriptCondition, ScriptOrder, TargetType};
use proptest::prelude::*;

/// Target names built to hit the grammar's edges: attribute-like prefixes, a leading dot,
/// and `.tmpl` / `.literal` / `.age` suffixes.
fn target_name() -> impl Strategy<Value = String> {
    let prefix = prop::sample::select(vec![
        "",
        ".",
        "..",
        "dot_",
        "literal_",
        "exact_",
        "private_",
        "readonly_",
        "run_",
        "once_",
        "onchange_",
        "before_",
        "after_",
        "create_",
        "modify_",
        "remove_",
        "symlink_",
        "empty_",
        "encrypted_",
        "executable_",
        "external_",
    ]);
    let suffix = prop::sample::select(vec!["", ".tmpl", ".literal", ".age", ".tmpl.literal"]);
    (prefix, "[a-z0-9_.-]{0,6}", suffix)
        .prop_map(|(p, mid, s)| format!("{p}{mid}{s}"))
        .prop_filter("not a valid target name", |name| {
            !name.is_empty() && name != "." && name != ".." && !name.contains('/')
        })
}

fn file_attr() -> impl Strategy<Value = FileAttr> {
    let target_type = prop::sample::select(vec![
        TargetType::Create,
        TargetType::File,
        TargetType::Modify,
        TargetType::Remove,
        TargetType::Script,
        TargetType::Symlink,
    ]);
    let condition = prop::sample::select(vec![
        ScriptCondition::Always,
        ScriptCondition::Once,
        ScriptCondition::OnChange,
    ]);
    let order = prop::sample::select(vec![
        ScriptOrder::Before,
        ScriptOrder::During,
        ScriptOrder::After,
    ]);
    (
        target_name(),
        target_type,
        condition,
        order,
        prop::array::uniform6(any::<bool>()),
    )
        .prop_map(|(target_name, target_type, condition, order, f)| {
            let flags = matches!(
                target_type,
                TargetType::Create | TargetType::File | TargetType::Modify
            );
            let script = target_type == TargetType::Script;
            FileAttr {
                target_name,
                target_type,
                condition: if script {
                    condition
                } else {
                    ScriptCondition::None
                },
                order: if script { order } else { ScriptOrder::During },
                empty: flags && target_type != TargetType::Modify && f[0],
                encrypted: flags && f[1],
                executable: flags && f[2],
                private: flags && f[3],
                read_only: flags && f[4],
                template: flags && f[5],
            }
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn file_attr_round_trips(
        attr in file_attr(),
        suffix in prop::sample::select(vec!["", ".age", ".asc"]),
    ) {
        let source_name = attr.source_name(suffix);
        let parsed = FileAttr::parse(&source_name, suffix);
        prop_assert_eq!(parsed.as_ref(), Ok(&attr), "source name {:?}", source_name);
        prop_assert_eq!(parsed.unwrap().source_name(suffix), source_name);
    }

    #[test]
    fn dir_attr_round_trips(
        target_name in target_name(),
        f in prop::array::uniform5(any::<bool>()),
    ) {
        let attr = DirAttr {
            target_name,
            exact: f[0],
            external: f[1],
            private: f[2],
            read_only: f[3],
            remove: f[4],
        };
        let source_name = attr.source_name();
        let parsed = DirAttr::parse(&source_name);
        prop_assert_eq!(parsed.as_ref(), Ok(&attr), "source name {:?}", source_name);
        prop_assert_eq!(parsed.unwrap().source_name(), source_name);
    }

    /// Parsing never panics, whatever the name.
    #[test]
    fn parse_never_panics(name in "\\PC{0,24}") {
        let _ = FileAttr::parse(&name, ".age");
        let _ = DirAttr::parse(&name);
    }
}
