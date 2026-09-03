//! # atipicial-native-contracts::atipicial_dollar
//!
//! Native ATD token state, accounting, and transfer behavior.
//!
//! ## Boundary
//!
//! This module belongs to `atipicial-native-contracts`. This execution-domain crate
//! owns native contract logic and storage codecs and must not own node startup,
//! RPC transport, or P2P sync.
//!
//! ## Contents
//!
//! - `initialize`: genesis ATD distribution seeding.
//! - `invoke`: AEP-17 native method handlers.
//! - `metadata`: Native contract metadata and descriptor helpers.
//! - `persist`: block-persist fee burn and primary reward accounting.
//! - `storage`: ATD account and total-supply storage helpers.
//! - `transfers`: ATD transfer, mint, and burn helpers.
//! - `tests`: Module-local tests and regression coverage.

use atipicial_config::ProtocolSettings;
use atipicial_error::CoreResult;
use atipicial_execution::{ApplicationEngine, NativeContract, NativeEvent, NativeMethod};

use crate::hashes::GAS_TOKEN_HASH;

mod initialize;
mod invoke;
mod metadata;
mod persist;
mod storage;
mod transfers;

native_contract_handle!(
    /// The AtipicialDollar native contract.
    pub struct AtipicialDollar {
        id: -6,
        contract_name: "AtipicialDollar",
        hash: GAS_TOKEN_HASH,
    }
);

impl AtipicialDollar {
    /// AEP-17 symbol (C# `AtipicialDollar.Symbol => "ATD"`).
    pub const SYMBOL: &'static str = "ATD";
    /// AEP-17 decimals (C# `AtipicialDollar.Decimals => 8`).
    pub const DECIMALS: u8 = 8;
}

impl<P> NativeContract<P> for AtipicialDollar
where
    P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
{
    native_contract_identity!(AtipicialDollar);

    fn methods(&self) -> &[NativeMethod] {
        &metadata::GAS_TOKEN_METHODS
    }

    fn supports_empty_block_fast_forward(&self) -> bool {
        true
    }

    /// C# `FungibleToken.OnManifestCompose` (FungibleToken.cs:68-71): every
    /// fungible token declares AEP-17 unconditionally.
    fn supported_standards(&self, _settings: &ProtocolSettings, _block_height: u32) -> Vec<String> {
        crate::native_supported_standards(&[crate::AEP17_STANDARD])
    }

    fn event_descriptors(&self) -> &[NativeEvent] {
        &metadata::GAS_TOKEN_EVENTS
    }

    native_contract_dispatch!(metadata::atipicial_dollar_method_bindings);

    fn initialize<D, B>(&self, engine: &mut ApplicationEngine<P, D, B>) -> CoreResult<()>
    where
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    {
        self.initialize_native(engine)
    }

    fn on_persist<D, B>(&self, engine: &mut ApplicationEngine<P, D, B>) -> CoreResult<()>
    where
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    {
        self.on_persist_native(engine)
    }
}

#[cfg(test)]
use transfers::GasTransferOutcome;

#[cfg(test)]
#[path = "../tests/atipicial_dollar/mod.rs"]
mod tests;
