//! Attribute prefixes and suffixes, and when a target name needs `literal_` / `.literal` to
//! stop it being read as an attribute.

pub const AFTER: &str = "after_";
pub const BEFORE: &str = "before_";
pub const CREATE: &str = "create_";
pub const DOT: &str = "dot_";
pub const EMPTY: &str = "empty_";
pub const ENCRYPTED: &str = "encrypted_";
pub const EXACT: &str = "exact_";
pub const EXECUTABLE: &str = "executable_";
pub const EXTERNAL: &str = "external_";
pub const LITERAL: &str = "literal_";
pub const MODIFY: &str = "modify_";
pub const ONCE: &str = "once_";
pub const ON_CHANGE: &str = "onchange_";
pub const PRIVATE: &str = "private_";
pub const READ_ONLY: &str = "readonly_";
pub const REMOVE: &str = "remove_";
pub const RUN: &str = "run_";
pub const SYMLINK: &str = "symlink_";
pub const LITERAL_SUFFIX: &str = ".literal";
pub const TEMPLATE_SUFFIX: &str = ".tmpl";

/// Prefixes that a directory target name must be protected from with `literal_`.
///
/// ADR-007: chezmoi's list lacks `remove_` and `external_`, so it writes a directory named
/// `remove_foo` as `remove_foo`, which reads back as "remove `foo`". logis protects them.
const DIR_ATTRIBUTE_PREFIXES: [&str; 7] =
    [DOT, EXACT, EXTERNAL, LITERAL, PRIVATE, READ_ONLY, REMOVE];

/// Prefixes that a file target name must be protected from with `literal_`.
///
/// ADR-007: chezmoi's list lacks `onchange_` (it has `once_`), so the script `onchange_x` reads
/// back as the onchange script `x`. logis protects it.
const FILE_ATTRIBUTE_PREFIXES: [&str; 16] = [
    AFTER, BEFORE, CREATE, DOT, EMPTY, ENCRYPTED, EXECUTABLE, LITERAL, MODIFY, ONCE, ON_CHANGE,
    PRIVATE, READ_ONLY, REMOVE, RUN, SYMLINK,
];

/// The prefixes whose flag is set, in order.
pub fn join_set(flags: &[(bool, &str)]) -> String {
    flags
        .iter()
        .filter(|(on, _)| *on)
        .map(|(_, prefix)| *prefix)
        .collect()
}

/// Remove `prefix` from the front of `name`, returning whether it was there.
pub fn cut(name: &mut &str, prefix: &str) -> bool {
    match name.strip_prefix(prefix) {
        Some(rest) => {
            *name = rest;
            true
        }
        None => false,
    }
}

/// Remove `suffix` from the end of `name`, returning whether it was there.
pub fn cut_suffix(name: &mut &str, suffix: &str) -> bool {
    match name.strip_suffix(suffix) {
        Some(rest) => {
            *name = rest;
            true
        }
        None => false,
    }
}

/// Split the name part of a source name into the target's leading `.` (from `dot_`) and the
/// rest, removing a protecting `literal_`.
pub fn decode_name(name: &str) -> (&'static str, &str) {
    if let Some(rest) = name.strip_prefix(DOT) {
        (".", rest)
    } else {
        ("", name.strip_prefix(LITERAL).unwrap_or(name))
    }
}

/// The name part of a source name for `target_name`: `.x` becomes `dot_x`, and a name that
/// would be read as an attribute gets `literal_`.
pub fn encode_name(target_name: &str, is_dir: bool) -> String {
    if let Some(rest) = target_name.strip_prefix('.') {
        return format!("{DOT}{rest}");
    }
    let prefixes: &[&str] = if is_dir {
        &DIR_ATTRIBUTE_PREFIXES
    } else {
        &FILE_ATTRIBUTE_PREFIXES
    };
    if prefixes.iter().any(|p| target_name.starts_with(p)) {
        format!("{LITERAL}{target_name}")
    } else {
        target_name.to_owned()
    }
}

/// Whether a file target name ends like an attribute suffix and needs `.literal`.
pub fn needs_literal_suffix(target_name: &str) -> bool {
    target_name.ends_with(LITERAL_SUFFIX) || target_name.ends_with(TEMPLATE_SUFFIX)
}

/// Whether a decoded target name (`dot` is `.` or empty, `name` the rest) is empty or `..`.
pub fn is_empty_or_parent(dot: &str, name: &str) -> bool {
    name.is_empty() || (dot == "." && name == ".") || (dot.is_empty() && name == "..")
}
