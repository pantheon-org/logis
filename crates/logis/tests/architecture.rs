//! Architecture rules (`ArchUnitRust`). These are tests: write the rule first, then the code.
//!
//! Empty scopes are allowed only while a crate has no code yet. Remove `allow_empty()` from a
//! rule as soon as its scope has files, so a stale selector can never pass silently.

use archunit::{CheckOptions, assert_passes, project_files, project_layers};

fn allow_empty() -> CheckOptions {
    CheckOptions::new().with_allow_empty_tests(true)
}

#[test]
fn crates_follow_the_declared_layers() {
    let rule = project_layers()
        .layer("cli")
        .defined_by("crates/logis/src/**")
        .layer("core")
        .defined_by("crates/logis-core/src/**")
        .layer("config")
        .defined_by("crates/logis-config/src/**")
        .layer("template")
        .defined_by("crates/logis-template/src/**")
        .where_layer("core")
        .may_only_depend_on_layers(&[])
        .where_layer("config")
        .may_only_depend_on_layers(&["core"])
        .where_layer("template")
        .may_only_depend_on_layers(&["core"]);

    assert_passes!(rule, allow_empty());
}

#[test]
fn no_dependency_cycles() {
    let rule = project_files()
        .in_path("crates/**/src/**")
        .should()
        .have_no_cycles();

    assert_passes!(rule);
}

#[test]
fn only_the_real_system_touches_the_machine() {
    let rule = project_files()
        .in_path("crates/logis-core/src/**")
        .should()
        .adhere_to(
            |file: &archunit::FileInfo| {
                const FORBIDDEN: [&str; 4] = [
                    "std::fs::",
                    "std::process::",
                    "use std::fs",
                    "use std::process",
                ];
                file.path.ends_with("system/real.rs")
                    || !FORBIDDEN.iter().any(|needle| file.content.contains(needle))
            },
            "use std::fs and std::process only in system/real.rs",
        );

    assert_passes!(rule);
}

#[test]
fn files_stay_small() {
    let rule = project_files()
        .in_path("crates/**/src/**")
        .should()
        .adhere_to(
            |file: &archunit::FileInfo| file.non_blank_line_count <= 400,
            "contain at most 400 non-blank lines",
        );

    assert_passes!(rule);
}
