//! The boundary between logis and the machine it manages.
//!
//! Only `real.rs` may touch the filesystem, the process umask or spawn processes. Every other module receives a
//! `&dyn System`. The trait lives in `port.rs` so implementations depend on it, not on this
//! module (keeps the file graph acyclic). `MemSystem` is an in-memory implementation for tests.

mod mem;
mod port;
mod real;

pub use mem::MemSystem;
pub use port::{FileType, Metadata, System};
pub use real::{RealSystem, process_umask};
