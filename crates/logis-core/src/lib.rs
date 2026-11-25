//! Core engine for logis: source state, target state and apply.
//!
//! All filesystem and process access goes through the [`system::System`] trait so that
//! dry-run, read-only and test sandboxes behave identically to the real system.
//! `tests/architecture.rs` enforces that boundary.

pub mod source_name;
pub mod system;
