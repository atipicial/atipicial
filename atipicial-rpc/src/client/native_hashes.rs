//! Static native-contract script hashes used by RPC client helpers.
//!
//! Client code only needs stable contract hashes for script construction and
//! balance probes. Use the associated `script_hash()` methods directly instead
//! of constructing native contract handles just to call `hash()`.

use atipicial_native_contracts::{AtipicialDollar, AtipicialCoin, PolicyContract};
use atipicial_primitives::UInt160;

#[must_use]
pub(super) fn atipicial_hash() -> UInt160 {
    AtipicialCoin::script_hash()
}

#[must_use]
pub(super) fn gas_hash() -> UInt160 {
    AtipicialDollar::script_hash()
}

#[must_use]
pub(super) fn policy_hash() -> UInt160 {
    PolicyContract::script_hash()
}
