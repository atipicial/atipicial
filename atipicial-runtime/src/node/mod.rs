//! # Node Capabilities
//!
//! This module defines the active provider-trait decoupling layer
//! ([`StoreProvider`] and [`TxAdmission`]) that lets upper-layer services
//! depend on narrow runtime capabilities instead of the concrete
//! `atipicial_system::Node`.
//!
//! ## Boundary
//!
//! These are dependency-inversion contracts only. Concrete node composition
//! and application lifecycle remain in `atipicial-system` and `atipicial-node`.
//!
//! ## Contents
//!
//! - `providers`: storage and transaction-admission capability traits.

mod providers;

pub use providers::{StoreProvider, TxAdmission};
