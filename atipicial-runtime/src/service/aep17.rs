//! AEP-17 token metadata reader trait.
//!
//! Defines the abstract boundary between the wallet layer (which needs
//! `symbol` / `decimals` to render transfer amounts) and the execution
//! layer (which runs a read-only contract call through `ApplicationEngine`
//! to obtain them).
//!
//! `atipicial-wallets` depends on this trait instead of `atipicial-execution` directly,
//! breaking the L4 → L3 execution-engine dependency for metadata reads.

use crate::error::ServiceError;
use atipicial_primitives::UInt160;
use std::fmt::Debug;

/// AEP-17 token metadata returned by [`Aep17MetadataReader::read_metadata`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aep17Metadata {
    /// The token's ticker symbol (e.g. `"ATD"`, `"ATC"`).
    pub symbol: String,
    /// The number of decimal places used by the token.
    pub decimals: u8,
}

/// Read-only AEP-17 token metadata provider.
///
/// Implemented by the execution layer; consumed by the wallet layer.
/// The concrete implementation runs a read-only `ApplicationEngine` script
/// that calls `symbol` and `decimals` on the target contract, matching
/// C# `Atipicial.Wallets.AssetDescriptor` semantics.
///
/// A single `read_metadata` call returns both fields so the implementor
/// can batch them into one VM execution (as the C# reference does).
pub trait Aep17MetadataReader: Send + Sync + Debug + 'static {
    /// Returns the symbol and decimals of the AEP-17 token at the given
    /// contract hash.
    ///
    /// Returns [`ServiceError::InvalidInput`] when the contract execution
    /// does not `HALT` or when the reported values are malformed.
    fn read_metadata(&self, contract_hash: UInt160) -> Result<Aep17Metadata, ServiceError>;
}
