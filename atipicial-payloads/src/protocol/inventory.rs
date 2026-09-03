//! Re-export of the Inventory trait from atipicial-primitives.
//!
//! The canonical `Inventory` trait now lives in [`atipicial_primitives`] so that
//! both atipicial-core (implementations) and atipicial-p2p (networking) can depend on
//! it without a circular dependency.

pub use atipicial_primitives::Inventory;
pub use atipicial_primitives::InventoryType;
