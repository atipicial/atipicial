//! AtipicialCoin genesis initialization.
//!
//! Seeds the exact C# genesis storage records for ATC while keeping the native
//! contract root focused on identity, metadata, hooks, and dispatch delegation.

use super::{ATC_TOTAL_AMOUNT, AtipicialCoin, DEFAULT_GAS_PER_BLOCK, DEFAULT_REGISTER_PRICE};
use atipicial_crypto::ECPoint;
use atipicial_error::CoreResult;
use atipicial_execution::ApplicationEngine;
use atipicial_storage::StorageItem;
use num_bigint::BigInt;

impl AtipicialCoin {
    /// C# `AtipicialCoin.InitializeAsync(engine, hardfork)` for `hardfork == ActiveIn`
    /// (ATC is genesis-active, so this runs while persisting block 0): seed the
    /// committee cache with the standby committee (zero votes each), an empty
    /// voters count, the genesis 5-GAS ATD-per-block record at index 0, the
    /// 1000-GAS register price, and mint `TotalAmount` ATC to the BFT address of
    /// the standby validators.
    pub(super) fn initialize_native<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
    ) -> CoreResult<()> {
        let standby_committee = engine.protocol_settings().standby_committee.clone();
        let standby_validators = engine.protocol_settings().standby_validators();
        let snapshot = engine.snapshot_cache();
        let members: Vec<(ECPoint, BigInt)> = standby_committee
            .into_iter()
            .map(|point| (point, BigInt::from(0)))
            .collect();
        snapshot.add(
            Self::committee_key(),
            StorageItem::from_bytes(Self::encode_committee(&members)?),
        );
        // C# `new StorageItem(Array.Empty<byte>())` — BigInteger zero is stored
        // as empty bytes.
        snapshot.add(
            Self::voters_count_key(),
            StorageItem::from_bytes(Vec::new()),
        );
        snapshot.add(
            Self::gas_per_block_key(0),
            StorageItem::from_bytes(crate::bigint_to_storage_bytes(&BigInt::from(
                DEFAULT_GAS_PER_BLOCK,
            ))),
        );
        snapshot.add(
            Self::register_price_key(),
            StorageItem::from_bytes(crate::bigint_to_storage_bytes(&BigInt::from(
                DEFAULT_REGISTER_PRICE,
            ))),
        );
        let bft = Self::bft_address(&standby_validators)?;
        self.atipicial_mint(engine, &bft, &BigInt::from(ATC_TOTAL_AMOUNT), false)
    }
}
