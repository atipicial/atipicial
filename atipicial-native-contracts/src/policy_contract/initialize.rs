//! PolicyContract genesis initialization.
//!
//! Seeds the C# genesis policy settings while keeping the root module focused
//! on identity, metadata, provider seams, and dispatch.

use super::{DEFAULT_EXEC_FEE_FACTOR, DEFAULT_FEE_PER_BYTE, DEFAULT_STORAGE_PRICE, PolicyContract};
use atipicial_error::CoreResult;
use atipicial_execution::ApplicationEngine;
use atipicial_storage::StorageItem;
use num_bigint::BigInt;

impl PolicyContract {
    /// C# `PolicyContract.InitializeAsync(engine, hardfork)` for `hardfork ==
    /// ActiveIn` (PolicyContract.cs:137-143; Policy is genesis-active, so this
    /// runs while persisting block 0): seed `Prefix_FeePerByte` (1000),
    /// `Prefix_ExecFeeFactor` (30), and `Prefix_StoragePrice` (100000). The
    /// HF_Echidna / HF_Faun re-initialization branches live in
    /// `initialize_for_hardfork`, triggered by `ContractManagement`'s
    /// `on_persist` at those hardfork blocks.
    pub(super) fn initialize_native<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
    ) -> CoreResult<()> {
        let snapshot = engine.snapshot_cache();
        snapshot.add(
            Self::fee_per_byte_key(),
            StorageItem::from_bytes(crate::bigint_to_storage_bytes(&BigInt::from(
                DEFAULT_FEE_PER_BYTE,
            ))),
        );
        snapshot.add(
            Self::exec_fee_factor_key(),
            StorageItem::from_bytes(crate::bigint_to_storage_bytes(&BigInt::from(
                DEFAULT_EXEC_FEE_FACTOR,
            ))),
        );
        snapshot.add(
            Self::storage_price_key(),
            StorageItem::from_bytes(crate::bigint_to_storage_bytes(&BigInt::from(
                DEFAULT_STORAGE_PRICE,
            ))),
        );
        Ok(())
    }
}
