use super::error::ParseError;
use super::prefix::{
    AFTER, BEFORE, CREATE, EMPTY, ENCRYPTED, EXECUTABLE, LITERAL_SUFFIX, MODIFY, ON_CHANGE, ONCE,
    PRIVATE, READ_ONLY, REMOVE, RUN, SYMLINK, TEMPLATE_SUFFIX, cut, cut_suffix, decode_name,
    encode_name, is_empty_or_parent, join_set, needs_literal_suffix,
};

/// What a file in the source state stands for in the target state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TargetType {
    /// Created if missing, never overwritten (`create_`).
    Create,
    #[default]
    File,
    /// A script that edits the existing target (`modify_`).
    Modify,
    /// The target is removed (`remove_`).
    Remove,
    /// A script that is run, not a target (`run_`).
    Script,
    Symlink,
}

/// When a script runs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScriptCondition {
    /// Not a script.
    #[default]
    None,
    Always,
    Once,
    OnChange,
}

/// Where a script runs relative to updating the targets.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScriptOrder {
    Before,
    #[default]
    During,
    After,
}

/// Attributes parsed from a source file name.
#[allow(
    clippy::struct_excessive_bools,
    reason = "each flag is an independent attribute of the source-name grammar"
)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileAttr {
    pub target_name: String,
    pub target_type: TargetType,
    pub condition: ScriptCondition,
    pub order: ScriptOrder,
    pub empty: bool,
    pub encrypted: bool,
    pub executable: bool,
    pub private: bool,
    pub read_only: bool,
    pub template: bool,
}

impl FileAttr {
    /// Parse one source file name. `encrypted_suffix` (for example `.age`) is removed from
    /// encrypted files' names.
    pub fn parse(name: &str, encrypted_suffix: &str) -> Result<Self, ParseError> {
        if name.is_empty() {
            return Err(ParseError::EmptyFileName);
        }
        let mut attr = Self::default();
        let mut rest = name;
        attr.parse_type_and_flags(&mut rest);
        let (dot, mut base) = decode_name(rest);
        if attr.encrypted {
            cut_suffix(&mut base, encrypted_suffix);
        }
        attr.template = cut_attribute_suffix(&mut base);
        if is_empty_or_parent(dot, base) || base == "." {
            return Err(ParseError::InvalidPath(name.to_owned()));
        }
        attr.target_name = format!("{dot}{base}");
        Ok(attr)
    }

    /// Read the leading type prefix and the flags that type allows.
    fn parse_type_and_flags(&mut self, rest: &mut &str) {
        if cut(rest, CREATE) {
            self.target_type = TargetType::Create;
            self.parse_flags(rest, true);
        } else if cut(rest, REMOVE) {
            self.target_type = TargetType::Remove;
        } else if cut(rest, RUN) {
            self.target_type = TargetType::Script;
            self.parse_script(rest);
        } else if cut(rest, SYMLINK) {
            self.target_type = TargetType::Symlink;
        } else if cut(rest, MODIFY) {
            self.target_type = TargetType::Modify;
            self.parse_flags(rest, false);
        } else {
            self.parse_flags(rest, true);
        }
    }

    /// Flags in their only accepted order: `encrypted_`, `private_`, `readonly_`, `empty_`
    /// (where allowed), `executable_`.
    fn parse_flags(&mut self, rest: &mut &str, allow_empty: bool) {
        self.encrypted = cut(rest, ENCRYPTED);
        self.private = cut(rest, PRIVATE);
        self.read_only = cut(rest, READ_ONLY);
        self.empty = allow_empty && cut(rest, EMPTY);
        self.executable = cut(rest, EXECUTABLE);
    }

    fn parse_script(&mut self, rest: &mut &str) {
        self.condition = if cut(rest, ONCE) {
            ScriptCondition::Once
        } else if cut(rest, ON_CHANGE) {
            ScriptCondition::OnChange
        } else {
            ScriptCondition::Always
        };
        if cut(rest, BEFORE) {
            self.order = ScriptOrder::Before;
        } else if cut(rest, AFTER) {
            self.order = ScriptOrder::After;
        }
    }

    /// The canonical source name for these attributes.
    #[must_use]
    pub fn source_name(&self, encrypted_suffix: &str) -> String {
        let mut name = self.type_and_flags();
        name.push_str(&encode_name(&self.target_name, false));
        if needs_literal_suffix(&self.target_name) {
            name.push_str(LITERAL_SUFFIX);
        }
        if self.template {
            name.push_str(TEMPLATE_SUFFIX);
        }
        if self.encrypted {
            name.push_str(encrypted_suffix);
        }
        name
    }

    fn type_and_flags(&self) -> String {
        match self.target_type {
            TargetType::Create => format!("{CREATE}{}", self.flags(true)),
            TargetType::File => self.flags(true),
            TargetType::Modify => format!("{MODIFY}{}", self.flags(false)),
            TargetType::Remove => REMOVE.to_owned(),
            TargetType::Script => {
                format!("{RUN}{}{}", self.condition_prefix(), self.order_prefix())
            }
            TargetType::Symlink => SYMLINK.to_owned(),
        }
    }

    fn flags(&self, allow_empty: bool) -> String {
        join_set(&[
            (self.encrypted, ENCRYPTED),
            (self.private, PRIVATE),
            (self.read_only, READ_ONLY),
            (allow_empty && self.empty, EMPTY),
            (self.executable, EXECUTABLE),
        ])
    }

    fn condition_prefix(&self) -> &'static str {
        match self.condition {
            ScriptCondition::Once => ONCE,
            ScriptCondition::OnChange => ON_CHANGE,
            ScriptCondition::None | ScriptCondition::Always => "",
        }
    }

    fn order_prefix(&self) -> &'static str {
        match self.order {
            ScriptOrder::Before => BEFORE,
            ScriptOrder::During => "",
            ScriptOrder::After => AFTER,
        }
    }

    /// The permission bits of the target file, before the umask.
    #[must_use]
    pub fn perm(&self) -> u32 {
        let mut perm = 0o666;
        if self.executable {
            perm |= 0o111;
        }
        if self.private {
            perm &= !0o077;
        }
        if self.read_only {
            perm &= !0o222;
        }
        perm
    }
}

/// Remove `.literal`, or `.tmpl` (then one protecting `.literal`), from a file name. Returns
/// whether the name was a template.
fn cut_attribute_suffix(base: &mut &str) -> bool {
    if cut_suffix(base, LITERAL_SUFFIX) {
        return false;
    }
    if cut_suffix(base, TEMPLATE_SUFFIX) {
        cut_suffix(base, LITERAL_SUFFIX);
        return true;
    }
    false
}
