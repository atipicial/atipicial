//! # atipicial-rpc::plugins::tokens_tracker::trackers::aep_11
//!
//! AEP-11 token tracking helpers.
//!
//! ## Boundary
//!
//! This module belongs to `atipicial-rpc`. This API crate owns JSON-RPC surfaces and
//! transport adapters and must not implement consensus, VM semantics, or
//! storage engines.
//!
//! ## Contents
//!
//! - `aep11_balance_key`: AEP-11 balance key records.
//! - `aep11_tracker`: AEP-11 tracker implementation.
//! - `aep11_transfer_key`: AEP-11 transfer key records.
//! - `tests`: Module-local tests and regression coverage.

pub mod aep11_balance_key;
pub mod aep11_tracker;
pub mod aep11_transfer_key;

use num_bigint::BigInt;

pub use aep11_balance_key::Aep11BalanceKey;
pub use aep11_tracker::Aep11Tracker;
pub use aep11_transfer_key::Aep11TransferKey;

fn token_id_integer(token: &[u8]) -> BigInt {
    if token.is_empty() {
        BigInt::from(0)
    } else {
        BigInt::from_signed_bytes_le(token)
    }
}

#[cfg(test)]
#[path = "../../../../tests/plugins/tokens_tracker/trackers/aep_11.rs"]
mod tests;
