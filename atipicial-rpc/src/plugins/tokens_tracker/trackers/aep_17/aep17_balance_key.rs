//! AEP-17 balance key.
//!
//! Storage key for AEP-17 token balances.

use atipicial_io::impl_ord_by_fields;
use atipicial_io::impl_serializable;
use atipicial_primitives::UInt160;
use serde::{Deserialize, Serialize};

/// Key for AEP-17 balance records.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Aep17BalanceKey {
    /// User's script hash.
    pub user_script_hash: UInt160,
    /// Token contract's script hash.
    pub asset_script_hash: UInt160,
}

impl Aep17BalanceKey {
    /// Creates a new balance key.
    pub fn new(user_script_hash: UInt160, asset_script_hash: UInt160) -> Self {
        Self {
            user_script_hash,
            asset_script_hash,
        }
    }
}

impl_ord_by_fields!(Aep17BalanceKey, user_script_hash, asset_script_hash);

impl_serializable! {
    struct Aep17BalanceKey {
        user_script_hash: UInt160,
        asset_script_hash: UInt160,
    }
}
