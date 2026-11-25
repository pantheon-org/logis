use super::error::ParseError;
use super::prefix::{
    EXACT, EXTERNAL, PRIVATE, READ_ONLY, REMOVE, cut, decode_name, encode_name, is_empty_or_parent,
    join_set,
};

/// Attributes parsed from a source directory name.
#[allow(
    clippy::struct_excessive_bools,
    reason = "each flag is an independent attribute of the source-name grammar"
)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DirAttr {
    pub target_name: String,
    pub exact: bool,
    pub external: bool,
    pub private: bool,
    pub read_only: bool,
    pub remove: bool,
}

impl DirAttr {
    /// Parse one source directory name. Attributes must appear in the order
    /// `remove_`, `external_`, `exact_`, `private_`, `readonly_`.
    pub fn parse(name: &str) -> Result<Self, ParseError> {
        if name.is_empty() {
            return Err(ParseError::EmptyDirName);
        }
        let mut rest = name;
        let remove = cut(&mut rest, REMOVE);
        let external = cut(&mut rest, EXTERNAL);
        let exact = cut(&mut rest, EXACT);
        let private = cut(&mut rest, PRIVATE);
        let read_only = cut(&mut rest, READ_ONLY);
        let (dot, base) = decode_name(rest);
        if is_empty_or_parent(dot, base) {
            return Err(ParseError::InvalidPath(name.to_owned()));
        }
        Ok(Self {
            target_name: format!("{dot}{base}"),
            exact,
            external,
            private,
            read_only,
            remove,
        })
    }

    /// The canonical source name for these attributes.
    #[must_use]
    pub fn source_name(&self) -> String {
        let mut name = join_set(&[
            (self.remove, REMOVE),
            (self.external, EXTERNAL),
            (self.exact, EXACT),
            (self.private, PRIVATE),
            (self.read_only, READ_ONLY),
        ]);
        name.push_str(&encode_name(&self.target_name, true));
        name
    }

    /// The permission bits of the target directory, before the umask.
    #[must_use]
    pub fn perm(&self) -> u32 {
        let mut perm = 0o777;
        if self.private {
            perm &= !0o077;
        }
        if self.read_only {
            perm &= !0o222;
        }
        perm
    }
}
