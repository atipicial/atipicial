//! ContractManagement genesis initialization.
//!
//! Seeds the C# genesis settings for contract deployment while keeping the root
//! module focused on identity, metadata, hooks, provider seams, and dispatch.

use super::{ContractManagement, DEFAULT_MINIMUM_DEPLOYMENT_FEE, DEFAULT_NEXT_AVAILABLE_ID};
use atipicial_error::CoreResult;
use atipicial_execution::ApplicationEngine;
use atipicial_storage::StorageItem;
use num_bigint::BigInt;

impl ContractManagement {
    /// C# `ContractManagement.InitializeAsync(engine, hardfork)` for `hardfork
    /// == ActiveIn` (ContractManagement.cs:53-61; the contract is
    /// genesis-active, so this runs while persisting block 0): seed
    /// `Prefix_MinimumDeploymentFee` (10 GAS) and `Prefix_NextAvailableId` (1).
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
            Self::minimum_deployment_fee_key(),
            StorageItem::from_bytes(crate::bigint_to_storage_bytes(&BigInt::from(
                DEFAULT_MINIMUM_DEPLOYMENT_FEE,
            ))),
        );
        snapshot.add(
            Self::next_available_id_key(),
            StorageItem::from_bytes(crate::bigint_to_storage_bytes(&BigInt::from(
                DEFAULT_NEXT_AVAILABLE_ID,
            ))),
        );
        Ok(())
    }
}
