//! # atipicial-native-contracts::atipicial_coin
//!
//! Native ATC token governance, voting, and committee behavior.
//!
//! ## Boundary
//!
//! This module belongs to `atipicial-native-contracts`. This execution-domain crate
//! owns native contract logic and storage codecs and must not own node startup,
//! RPC transport, or P2P sync.
//!
//! ## Contents
//!
//! - `constants`: protocol storage prefixes, defaults, reward ratios, and
//!   event names.
//! - `fast_forward`: state-equivalent empty-block reward batching.
//! - `initialize`: genesis storage seeding.
//! - `invoke`: native ATC invocation helpers.
//! - `metadata`: Native contract metadata and descriptor helpers.
//! - `persist`: block-persist committee and reward hooks.
//! - `providers`: validator and next-consensus read providers.
//! - `storage`: Storage contexts, key builders, and storage item helpers for
//!   execution.
//! - `transfers`: wallet transfer RPC handlers.
//! - `tests`: Module-local tests and regression coverage.

use atipicial_config::{Hardfork, ProtocolSettings};
use atipicial_crypto::ECPoint;
use atipicial_error::CoreResult;
use atipicial_execution::{ApplicationEngine, NativeContract, NativeEvent, NativeMethod};
use atipicial_primitives::UInt160;
use atipicial_storage::persistence::{DataCache, SeekDirection};
use atipicial_storage::{StorageItem, StorageKey};
use atipicial_vm::StackItem;
use num_bigint::BigInt;

use crate::hashes::ATC_TOKEN_HASH;

mod constants;
mod fast_forward;
mod initialize;
mod invoke;
mod metadata;
mod persist;
mod providers;
mod storage;
mod transfers;

pub(in crate::atipicial_coin) use constants::*;
pub(crate) use constants::{
    ATC_CANDIDATE_STATE_CHANGED_EVENT, ATC_COMMITTEE_CHANGED_EVENT, ATC_VOTE_EVENT,
};
pub(crate) use storage::CachedCommittee;
// Rationale: this storage type is imported for native-contract parity paths
// that are conditionally compiled or reached by integration-only flows.
use storage::AtipicialAccountStateView;
#[allow(unused_imports)]
use storage::CandidateState;
pub(crate) use storage::candidate_signature_account;

native_contract_handle!(
    /// The AtipicialCoin native contract.
    pub struct AtipicialCoin {
        id: -5,
        contract_name: "AtipicialCoin",
        hash: ATC_TOKEN_HASH,
    }
);

impl AtipicialCoin {
    /// AEP-17 symbol (C# `AtipicialCoin.Symbol => "ATC"`).
    pub const SYMBOL: &'static str = "ATC";
    /// AEP-17 decimals (C# `AtipicialCoin.Decimals => 0`).
    pub const DECIMALS: u8 = 0;
}

impl<P> NativeContract<P> for AtipicialCoin
where
    P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
{
    native_contract_identity!(AtipicialCoin);

    fn methods(&self) -> &[NativeMethod] {
        &metadata::ATC_TOKEN_METHODS
    }

    fn supports_empty_block_fast_forward(&self) -> bool {
        true
    }

    /// C# `AtipicialCoin.OnManifestCompose` (AtipicialCoin.cs:112-122): ATC declares
    /// AEP-27 in addition to AEP-17 once HF_Echidna is enabled at the height.
    fn supported_standards(&self, settings: &ProtocolSettings, block_height: u32) -> Vec<String> {
        if settings.is_hardfork_enabled(Hardfork::HfEchidna, block_height) {
            crate::native_supported_standards(&[crate::AEP17_STANDARD, crate::AEP27_STANDARD])
        } else {
            crate::native_supported_standards(&[crate::AEP17_STANDARD])
        }
    }

    fn event_descriptors(&self) -> &[NativeEvent] {
        &metadata::ATC_TOKEN_EVENTS
    }

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

    fn post_persist<D, B>(&self, engine: &mut ApplicationEngine<P, D, B>) -> CoreResult<()>
    where
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    {
        self.post_persist_native(engine)
    }

    native_contract_dispatch!(metadata::atipicial_coin_method_bindings);

    /// C# `ATC.GetCommitteeAddress`, exposed through the native-contract seam so
    /// the engine's `check_committee_witness` can verify committee-gated writers
    /// without depending on `atipicial-native-contracts`.
    fn committee_address<B>(&self, snapshot: &DataCache<B>) -> CoreResult<Option<UInt160>>
    where
        B: atipicial_storage::CacheRead,
    {
        Ok(Some(self.compute_committee_address(snapshot)?))
    }
}

#[cfg(test)]
#[path = "../tests/atipicial_token/mod.rs"]
mod tests;
