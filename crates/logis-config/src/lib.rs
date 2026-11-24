//! Pkl configuration for logis.
//!
//! Plan: evaluate `logis.pkl` with `pkl eval -f json` and deserialize with serde. The schema
//! lives in `pkl/Logis.pkl`. Only the runner module may spawn `pkl`. Must not depend on
//! `logis-template` or the CLI (see `tests/architecture.rs`).
//!
//! Intentionally empty: code arrives test-first in roadmap phase 3.
