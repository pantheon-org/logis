//! The tree every contract check starts from, built in both `System` implementations.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use logis_core::system::{MemSystem, RealSystem, System};

/// The tree every check uses, under `root`:
/// `dir/` (0o755), `dir/file` (0o644, "contents"), `dir/sub/` (0o700),
/// `dir/link -> file`, `dir/dangling -> missing`, `dirlink -> dir`, `exe` (0o755, "#!").
fn real(root: &Path) -> RealSystem {
    let set =
        |p: &Path, mode| std::fs::set_permissions(p, PermissionsExt::from_mode(mode)).unwrap();
    std::fs::create_dir_all(root.join("dir/sub")).unwrap();
    set(&root.join("dir"), 0o755);
    set(&root.join("dir/sub"), 0o700);
    std::fs::write(root.join("dir/file"), "contents").unwrap();
    set(&root.join("dir/file"), 0o644);
    std::os::unix::fs::symlink("file", root.join("dir/link")).unwrap();
    std::os::unix::fs::symlink("missing", root.join("dir/dangling")).unwrap();
    std::os::unix::fs::symlink("dir", root.join("dirlink")).unwrap();
    std::fs::write(root.join("exe"), "#!").unwrap();
    set(&root.join("exe"), 0o755);
    RealSystem
}

fn mem(root: &Path) -> MemSystem {
    MemSystem::new()
        .with_dir(root, 0o755)
        .with_dir(root.join("dir"), 0o755)
        .with_dir(root.join("dir/sub"), 0o700)
        .with_file(root.join("dir/file"), "contents", 0o644)
        .with_symlink(root.join("dir/link"), "file")
        .with_symlink(root.join("dir/dangling"), "missing")
        .with_symlink(root.join("dirlink"), "dir")
        .with_file(root.join("exe"), "#!", 0o755)
}

/// Run `check` against both implementations.
pub fn both(check: impl Fn(&dyn System, &Path)) {
    let tmp = tempfile::tempdir().unwrap();
    let root: PathBuf = tmp.path().join("root");
    std::fs::create_dir(&root).unwrap();
    check(&real(&root), &root);
    let mem_root = Path::new("/mem/root");
    check(&mem(mem_root), mem_root);
}
