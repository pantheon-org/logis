//! Paths printed as a tree, as chezmoi's `--tree` flag does (`internal/cmd/pathlisttree.go`):
//! one name per line, sorted, indented two spaces per level, parents printed once.

use std::collections::BTreeMap;

#[derive(Default)]
struct Node<'a> {
    children: BTreeMap<&'a str, Node<'a>>,
}

/// The `/`-separated `paths` as an indented tree, each line ending in a newline.
#[must_use]
pub fn tree<S: AsRef<str>>(paths: &[S]) -> String {
    let mut root = Node::default();
    for path in paths {
        let mut node = &mut root;
        for name in path.as_ref().split('/') {
            node = node.children.entry(name).or_default();
        }
    }
    let mut out = String::new();
    write(&root, "", &mut out);
    out
}

fn write(node: &Node<'_>, indent: &str, out: &mut String) {
    for (name, child) in &node.children {
        out.push_str(indent);
        out.push_str(name);
        out.push('\n');
        write(child, &format!("{indent}  "), out);
    }
}
