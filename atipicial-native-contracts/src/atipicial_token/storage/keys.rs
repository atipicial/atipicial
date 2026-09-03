//! Storage key constructors for the ATC native contract.
//!
//! This module is the single place for ATC storage-key layout helpers. It does
//! not serialize values or execute contract logic; it only maps domain inputs to
//! the exact C#-compatible storage prefixes and suffixes.

use super::*;

impl AtipicialCoin {
    pub(in crate::atipicial_coin) fn register_price_key() -> StorageKey {
        crate::keys::prefixed_key(Self::ID, PREFIX_REGISTER_PRICE, &[])
    }

    /// The `Prefix_GasPerBlock` prefix key used for backward gas-record scans.
    pub(in crate::atipicial_coin) fn gas_per_block_prefix_key() -> StorageKey {
        crate::keys::prefixed_key(Self::ID, PREFIX_GAS_PER_BLOCK, &[])
    }

    /// The `Prefix_GasPerBlock` storage key for a record index.
    pub(in crate::atipicial_coin) fn gas_per_block_key(index: u32) -> StorageKey {
        crate::keys::prefixed_u32_be_key(Self::ID, PREFIX_GAS_PER_BLOCK, index)
    }

    /// The `Prefix_VotersCount` storage key (a single key, no suffix).
    pub(crate) fn voters_count_key() -> StorageKey {
        crate::keys::prefixed_key(Self::ID, PREFIX_VOTERS_COUNT, &[])
    }

    /// The `Prefix_Candidate` storage key for `pubkey` (`prefix ++ 33-byte pubkey`).
    pub(crate) fn candidate_key(pubkey: &ECPoint) -> StorageKey {
        crate::keys::prefixed_key(Self::ID, PREFIX_CANDIDATE, &pubkey.to_bytes())
    }

    /// The `Prefix_Candidate` prefix key used for candidate scans.
    pub(in crate::atipicial_coin) fn candidate_prefix_key() -> StorageKey {
        crate::keys::prefixed_key(Self::ID, PREFIX_CANDIDATE, &[])
    }

    /// The `Prefix_Committee` storage key (a single key, no suffix).
    pub(crate) fn committee_key() -> StorageKey {
        crate::keys::prefixed_key(Self::ID, PREFIX_COMMITTEE, &[])
    }

    /// The `Prefix_VoterRewardPerCommittee` storage key for `pubkey`.
    pub(crate) fn voter_reward_per_committee_key(pubkey: &ECPoint) -> StorageKey {
        crate::keys::prefixed_key(
            Self::ID,
            PREFIX_VOTER_REWARD_PER_COMMITTEE,
            &pubkey.to_bytes(),
        )
    }

    /// The `Prefix_Account` storage key for `account` (AEP-17 account prefix).
    pub(crate) fn account_key(account: &UInt160) -> StorageKey {
        crate::aep17_account_key(Self::ID, account)
    }

    /// The AEP-17 total-supply storage key for ATC (`Prefix_TotalSupply`).
    pub(crate) fn total_supply_key() -> StorageKey {
        crate::aep17_total_supply_key(Self::ID)
    }
}
