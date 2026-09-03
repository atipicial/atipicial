//! # atipicial-node
//!
//! Application-level process coordination shared by the Atipicial node binaries.
//!
//! ## Boundary
//!
//! This library owns process lifecycle mechanics for `atipicial-node`. It does not
//! define protocol behavior, storage formats, or persistence semantics.
//!
//! ## Contents
//!
//! - `process`: process-wide ownership and lifecycle primitives.

#![doc(html_root_url = "https://docs.rs/atipicial-node/0.11.1")]

mod process;

pub use process::{NodeLifecycleLock, NodeLifecycleLockError};
