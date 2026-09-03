//! Network event types.
//!
//! The canonical [`NetworkEvent`] enum is defined in
//! [`atipicial_runtime::outcome::NetworkEvent`] as part of the Stage A
//! service trait contract. This module re-exports it and provides a
//! local type alias so the rest of `atipicial-network` can use the
//! unqualified name.

pub use atipicial_runtime::NetworkEvent as NetworkEventKind;

/// Local alias for [`atipicial_runtime::NetworkEvent`].
pub type NetworkEvent = NetworkEventKind;
