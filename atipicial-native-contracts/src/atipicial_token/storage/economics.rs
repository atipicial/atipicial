//! ATC economic storage records and unclaimed-GAS calculation.

use super::*;
use atipicial_error::CoreError;
use num_traits::ToPrimitive;

impl AtipicialCoin {
    /// C# `GetRegisterPrice` = `(long)(BigInteger)snapshot[_registerPrice]`.
    pub(in crate::atipicial_coin) fn register_price<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
    ) -> CoreResult<i64> {
        let key = Self::register_price_key();
        let Some(item) = snapshot.get(&key) else {
            return Err(CoreError::invalid_operation(
                "AtipicialCoin RegisterPrice storage is missing",
            ));
        };
        BigInt::from_signed_bytes_le(&item.value_bytes())
            .to_i64()
            .ok_or_else(|| {
                CoreError::invalid_operation("AtipicialCoin RegisterPrice is out of range")
            })
    }

    /// C# `SetRegisterPrice` storage effect: overwrite `Prefix_RegisterPrice` as a
    /// `BigInteger` (`GetAndChange(_registerPrice).Set(registerPrice)`).
    pub(in crate::atipicial_coin) fn put_register_price<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        price: i64,
    ) -> CoreResult<()> {
        let key = Self::register_price_key();
        if snapshot.get(&key).is_none() {
            return Err(CoreError::invalid_operation(
                "AtipicialCoin RegisterPrice storage is missing",
            ));
        }
        snapshot.update(
            key,
            StorageItem::from_bytes(crate::bigint_to_storage_bytes(&BigInt::from(price))),
        );
        Ok(())
    }

    /// C# `SetAtipicialDollarPerBlock` storage effect: write a `Prefix_AtipicialDollarPerBlock` record at
    /// `index` (a big-endian `uint` key suffix), overwriting any record already at
    /// that index (`GetAndChange(key, factory).Set(gasPerBlock)`). `update` upserts
    /// (a brand-new index key is tracked as Changed), which commits to the same
    /// stored key/value as the C# Added path — only the resulting store contents
    /// feed the state root.
    pub(in crate::atipicial_coin) fn put_gas_per_block<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        index: u32,
        gas_per_block: &BigInt,
    ) {
        let key = Self::gas_per_block_key(index);
        snapshot.update(
            key,
            StorageItem::from_bytes(crate::bigint_to_storage_bytes(gas_per_block)),
        );
    }

    /// Returns the GAS-per-block effective at `index`: the most recent
    /// `Prefix_AtipicialDollarPerBlock` record whose record index is ≤ `index` (C#
    /// `GetSortedGasRecords(...).First().AtipicialDollarPerBlock`), defaulting to 5 GAS.
    pub(in crate::atipicial_coin) fn gas_per_block_at<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        index: u32,
    ) -> BigInt {
        let prefix = Self::gas_per_block_prefix_key();
        for (key, item) in snapshot.find(Some(&prefix), SeekDirection::Backward) {
            let key_bytes = key.key();
            if key_bytes.len() >= 5 {
                let record_index =
                    u32::from_be_bytes([key_bytes[1], key_bytes[2], key_bytes[3], key_bytes[4]]);
                if record_index <= index {
                    return BigInt::from_signed_bytes_le(&item.value_bytes());
                }
            }
        }
        BigInt::from(DEFAULT_GAS_PER_BLOCK)
    }

    /// Reads the total voted ATC (`Prefix_VotersCount`), defaulting to zero.
    pub(in crate::atipicial_coin) fn read_voters_count<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
    ) -> BigInt {
        snapshot
            .get(&Self::voters_count_key())
            .map(|item| BigInt::from_signed_bytes_le(&item.value_bytes()))
            .unwrap_or_else(|| BigInt::from(0))
    }

    /// Writes the total voted ATC (`Prefix_VotersCount`).
    pub(in crate::atipicial_coin) fn write_voters_count<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        value: &BigInt,
    ) {
        snapshot.update(
            Self::voters_count_key(),
            StorageItem::from_bytes(crate::bigint_to_storage_bytes(value)),
        );
    }

    /// C# `GetSortedGasRecords(snapshot, end)`: the `Prefix_AtipicialDollarPerBlock` records with
    /// index ≤ `end`, descending by index.
    pub(in crate::atipicial_coin) fn sorted_gas_records<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        end: u32,
    ) -> Vec<(u32, BigInt)> {
        let prefix = Self::gas_per_block_prefix_key();
        let mut out = Vec::new();
        for (key, item) in snapshot.find(Some(&prefix), SeekDirection::Backward) {
            let key_bytes = key.key();
            if key_bytes.len() >= 5 {
                let index =
                    u32::from_be_bytes([key_bytes[1], key_bytes[2], key_bytes[3], key_bytes[4]]);
                if index <= end {
                    out.push((index, BigInt::from_signed_bytes_le(&item.value_bytes())));
                }
            }
        }
        out
    }

    /// Reads the accumulated GAS-per-vote for `pubkey` (`Prefix_VoterRewardPerCommittee`).
    pub(in crate::atipicial_coin) fn voter_reward_per_committee<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        pubkey: &ECPoint,
    ) -> BigInt {
        let key = Self::voter_reward_per_committee_key(pubkey);
        snapshot
            .get(&key)
            .map(|item| BigInt::from_signed_bytes_le(&item.value_bytes()))
            .unwrap_or_else(|| BigInt::from(0))
    }

    /// C# `AtipicialCoin.CalculateBonus`: the unclaimed GAS for an account between
    /// `BalanceHeight` and `end` — the ATC-holder reward (`balance * Σ gasPerBlock *
    /// 10 / 100 / TotalAmount`) plus the vote reward (`balance * (latestGasPerVote -
    /// lastGasPerVote) / VoteFactor`).
    pub(in crate::atipicial_coin) fn calculate_bonus<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        state: &AtipicialAccountStateView,
        end: u32,
    ) -> CoreResult<BigInt> {
        if state.balance == BigInt::from(0) {
            return Ok(BigInt::from(0));
        }
        if state.balance < BigInt::from(0) {
            return Err(CoreError::invalid_operation(
                "AtipicialCoin account balance cannot be negative",
            ));
        }
        if state.balance_height >= end {
            return Ok(BigInt::from(0));
        }

        // ATC-holder reward over [BalanceHeight, end), folding in each ATD-per-block
        // change point (C# CalculateReward).
        let start = state.balance_height;
        let mut sum_gas_per_block = BigInt::from(0);
        let mut window_end = end;
        for (index, gas_per_block) in self.sorted_gas_records(snapshot, end.saturating_sub(1)) {
            if index > start {
                sum_gas_per_block += &gas_per_block * (window_end - index);
                window_end = index;
            } else {
                sum_gas_per_block += &gas_per_block * (window_end - start);
                break;
            }
        }
        let atipicial_holder_reward =
            &state.balance * &sum_gas_per_block * ATC_HOLDER_REWARD_RATIO / 100 / ATC_TOTAL_AMOUNT;

        // Vote reward (only when the account currently votes).
        let vote_reward = match &state.vote_to {
            Some(vote) => {
                let latest = self.voter_reward_per_committee(snapshot, vote);
                &state.balance * (latest - &state.last_gas_per_vote) / VOTE_FACTOR
            }
            None => BigInt::from(0),
        };

        Ok(atipicial_holder_reward + vote_reward)
    }
}
