//! Verifiable trait for blockchain objects.
//!
//! This is the simplified base trait that lives in atipicial-primitives (Layer 0).
//! Atipicial-core extends it with additional methods that depend on DataCache,
//! ProtocolSettings, and the smart contract execution engine.

use crate::UInt256;
use crate::error::PrimitiveResult;

/// Base trait for verifiable blockchain objects.
///
/// Provides hash computation and witness access without depending on
/// higher-layer types (DataCache, ProtocolSettings, ApplicationEngine).
///
/// # Implementors
///
/// - `Block` (atipicial-core)
/// - `Transaction` (atipicial-core)
/// - `Header` / `BlockHeader` (atipicial-core)
/// - `ExtensiblePayload` (atipicial-core)
pub trait Verifiable: Send + Sync {
    /// Verifies the cryptographic validity of the object (state-independent checks only).
    fn verify(&self) -> bool;

    /// Computes the hash of the object.
    fn hash(&self) -> PrimitiveResult<UInt256>;

    /// Gets the serialized data used for hash computation (unsigned, no witnesses).
    fn hash_data(&self) -> Vec<u8>;
}
