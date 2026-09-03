//! # atipicial-native-contracts
//!
//! Atipicial N3 native contract implementations and storage codecs.
//!
//! ## Boundary
//!
//! This execution-domain crate owns native contract logic and storage codecs
//! and must not own node startup, RPC transport, or P2P sync.
//!
//! ## Contents
//!
//! - `registry`: Native contract registry and dispatch helpers.
//! - `support`: Shared support helpers that keep domain modules focused.
//! - `text`: Text segmentation and compatibility helpers for native contracts.
//! - `contract_management`: Native ContractManagement state, storage, and
//!   lifecycle operations.
//! - `crypto_lib`: Native CryptoLib interop surface and verification helpers.
//! - `atipicial_dollar`: Native ATD token state, accounting, and transfer behavior.
//! - `ledger_contract`: Native Ledger contract storage and query behavior.
//! - `atipicial_coin`: Native ATC token governance, voting, and committee behavior.
//! - `notary`: Native Notary contract state and request verification behavior.
//! - `oracle_contract`: Native Oracle contract request, response, and fee
//!   behavior.
//! - `policy_contract`: Native Policy contract fee, account, and storage policy
//!   behavior.
//! - `role_management`: Native RoleManagement state and designated-node
//!   behavior.
//! - `std_lib`: Native StdLib string, memory, and serialization helpers.
//! - `test_support`: crate-local test support fixtures.
//! - `treasury`: Native treasury accounting and fund recovery behavior.
//! - `tests`: Module-local tests and regression coverage.

pub use atipicial_execution::{
    HardforkActivable, NativeContract, NativeContractsCache, NativeContractsCacheEntry,
    NativeEvent, NativeMethod, NativeRegistry, is_active_for,
};

#[macro_use]
pub(crate) mod support;

/// Native-contract catalog, hashes, provider, and role definitions.
pub mod registry;
mod text;

#[path = "atipicial_token/mod.rs"]
pub mod atipicial_coin;
pub mod atipicial_dollar;
pub mod contract_management;
pub mod crypto_lib;
pub mod ledger_contract;
pub mod notary;
pub mod oracle_contract;
pub mod policy_contract;
pub mod role_management;
pub mod std_lib;
#[cfg(test)]
#[path = "tests/test_support.rs"]
pub(crate) mod test_support;
pub mod treasury;

pub use registry::{catalog, hashes, native_contract, provider, role, standard};
pub(crate) use support::{args, committee, keys};

pub use atipicial_coin::AtipicialCoin;
pub use atipicial_dollar::AtipicialDollar;
pub use catalog::{
    STANDARD_NATIVE_CONTRACT_COUNT, StandardNativeContractHashes, StandardNativeContractSpec,
    StandardNativeContractSpecs, is_standard_native_contract_hash, standard_native_contract_hashes,
    standard_native_contract_spec_by_hash, standard_native_contract_spec_by_id,
    standard_native_contract_spec_by_name, standard_native_contract_specs,
    standard_native_contracts,
};
pub use contract_management::ContractManagement;
pub use crypto_lib::CryptoLib;
pub use ledger_contract::LedgerContract;
pub use notary::Notary;
pub use oracle_contract::{OracleContract, OracleRequest};
pub use policy_contract::PolicyContract;
pub use provider::StandardNativeProvider;
pub use role::Role;
pub use role_management::RoleManagement;
pub use standard::StandardNativeContract;
pub use std_lib::StdLib;
pub use treasury::Treasury;

#[cfg(test)]
pub(crate) use support::token::aep::{
    AEP11_PAYMENT_METHOD, AEP17_PREFIX_ACCOUNT, AEP17_PREFIX_TOTAL_SUPPLY,
};
pub(crate) use support::token::aep::{
    AEP17_PAYMENT_METHOD, AEP17_STANDARD, AEP17_TRANSFER_EVENT, AEP26_STANDARD, AEP27_STANDARD,
    AEP30_STANDARD, AccountState, aep11_payment_method, aep17_account_key, aep17_balance_of_method,
    aep17_decimals_method, aep17_payment_callback_args, aep17_payment_data_item,
    aep17_payment_method, aep17_symbol_method, aep17_total_supply_key, aep17_total_supply_method,
    aep17_transfer_method, aep17_transfer_notification_state, deserialize_account_state,
    fungible_token_transfer_event, native_supported_standards, read_aep17_balance,
    read_aep17_total_supply, serialize_account_state,
};
pub(crate) use support::token::storage_encoding::bigint_to_storage_bytes;

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
