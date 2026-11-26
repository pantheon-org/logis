//! Reading the process umask, as chezmoi does at start-up (`internal/chezmoi/chezmoi_unix.go`).
//!
//! The umask is process-wide, so this is its own test binary with a single test: no other test
//! can create files while it changes the umask.

#![cfg(unix)]

use logis_core::system::process_umask;
use rustix::fs::Mode;
use rustix::process::umask;

#[test]
fn reads_the_umask_and_leaves_it_unchanged() {
    let original = umask(Mode::from_raw_mode(0o027));
    for _ in 0..2 {
        assert_eq!(process_umask(), 0o027);
    }
    umask(Mode::from_raw_mode(0o077));
    assert_eq!(process_umask(), 0o077);
    umask(original);
}
