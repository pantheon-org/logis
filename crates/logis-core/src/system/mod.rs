//! The boundary between logis and the machine it manages.
//!
//! Only `real.rs` may touch the filesystem or spawn processes. Every other module receives a
//! `&dyn System`. The trait lives in `port.rs` so implementations depend on it, not on this
//! module (keeps the file graph acyclic).

mod port;
mod real;

pub use port::System;
pub use real::RealSystem;
