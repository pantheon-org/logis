//! Jinja templating for logis, built on minijinja (engine choice still to be confirmed).
//!
//! Must not use `std::process` directly: template functions that run commands go through an
//! injected trait. Must not depend on `logis-config` or the CLI (see `tests/architecture.rs`).
//!
//! Intentionally empty: code arrives test-first in roadmap phase 4.
