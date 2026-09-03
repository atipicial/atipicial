//! # atipicial-config::network
//!
//! Network selection, genesis data, and chain identity configuration.
//!
//! ## Boundary
//!
//! This module belongs to `atipicial-config`. This configuration crate owns typed
//! settings and must not open storage, start services, or run protocol
//! workflows.
//!
//! ## Contents
//!
//! - `chain_spec`: validated immutable chain identity and protocol rules.
//! - `genesis`: genesis block and committee configuration.
//! - `network_type`: Atipicial network identifiers.

pub mod chain_spec;
pub mod genesis;
pub mod network_type;
pub(crate) mod public_key;
