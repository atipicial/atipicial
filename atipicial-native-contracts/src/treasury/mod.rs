//! # atipicial-native-contracts::treasury
//!
//! Native treasury accounting and fund recovery behavior.
//!
//! ## Boundary
//!
//! This module belongs to `atipicial-native-contracts`. This execution-domain crate
//! owns native contract logic and storage codecs and must not own node startup,
//! RPC transport, or P2P sync.
//!
//! ## Contents
//!
//! - `invoke`: native method handlers for payment callbacks and verification.
//! - `metadata`: Native contract metadata and descriptor helpers.
//! - `tests`: Module-local tests and regression coverage.
//! - `verify_witness_tests`: witness verification coverage.

use atipicial_config::{Hardfork, ProtocolSettings};
use atipicial_execution::{NativeContract, NativeMethod};

use crate::hashes::TREASURY_HASH;

mod invoke;
mod metadata;

native_contract_handle!(
    /// The Treasury native contract.
    pub struct Treasury {
        id: -11,
        contract_name: "Treasury",
        hash: TREASURY_HASH,
    }
);

impl<P> NativeContract<P> for Treasury
where
    P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
{
    native_contract_identity!(Treasury);

    // C# `Treasury.Activations => [Hardfork.HF_Faun]` (Treasury.cs:29): the
    // contract does not exist before HF_Faun. Without this override Treasury
    // would be genesis-active in atipicial-rs, diverging native deployment and
    // manifest state below the configured Faun height. If a custom/private
    // config omits Faun, C# `IsActive` treats ActiveIn as genesis-active, while
    // `IsInitializeBlock` skips the missing hardfork initialization block.
    fn active_in(&self) -> Option<Hardfork> {
        Some(Hardfork::HfFaun)
    }

    fn activations(&self) -> &'static [Hardfork] {
        &[Hardfork::HfFaun]
    }

    /// C# `Treasury.OnManifestCompose` (Treasury.cs:31-34): unconditional —
    /// the contract only exists from HF_Faun onwards.
    fn supported_standards(&self, _settings: &ProtocolSettings, _block_height: u32) -> Vec<String> {
        crate::native_supported_standards(&[
            crate::AEP26_STANDARD,
            crate::AEP27_STANDARD,
            crate::AEP30_STANDARD,
        ])
    }

    fn methods(&self) -> &[NativeMethod] {
        &metadata::TREASURY_METHODS
    }

    fn supports_empty_block_fast_forward(&self) -> bool {
        true
    }

    native_contract_dispatch!(metadata::treasury_method_bindings);
}

#[cfg(test)]
use atipicial_execution::ApplicationEngine;

#[cfg(test)]
#[path = "../tests/treasury/mod.rs"]
mod tests;

/// End-to-end verification of `verify` through the VM: a script
/// `System.Contract.Call`s Treasury and the boolean result reflects whether
/// the committee multisig address witnesses the transaction (C#
/// `Treasury.Verify` = `CheckCommittee(engine)`).
#[cfg(test)]
#[path = "../tests/treasury/verify_witness_tests.rs"]
mod verify_witness_tests;
