//! ATC account-state codecs and balance reads.

use super::*;

impl AtipicialCoin {
    /// Decodes a stored `AtipicialAccountState` struct into its fields.
    pub(in crate::atipicial_coin) fn decode_atipicial_account_state(
        value: &[u8],
    ) -> CoreResult<AtipicialAccountStateView> {
        let decoded = crate::support::codec::decode_stack_item(value, "atipicial account state")?;
        AtipicialAccountStateView::from_stack_item(&decoded)
    }

    /// Encodes a `AtipicialAccountState` (`Struct[Balance, BalanceHeight, VoteTo,
    /// LastGasPerVote]`) — the write counterpart of [`decode_atipicial_account_state`].
    pub(in crate::atipicial_coin) fn encode_atipicial_account_state(
        state: &AtipicialAccountStateView,
    ) -> CoreResult<Vec<u8>> {
        crate::support::codec::encode_storage_struct(state, "atipicial account state")
    }

    /// C# `GetAccountState`: the stored `AtipicialAccountState` struct bytes under
    /// `Prefix_Account ++ account`, or `None` when the account has no entry. The
    /// stored value is already the BinarySerializer-encoded struct (balance,
    /// balanceHeight, voteTo, lastGasPerVote), which is exactly the Array/Struct
    /// return shape, so it is returned as-is (the same pattern as
    /// `getDesignatedByRole` / `getContract`).
    pub(in crate::atipicial_coin) fn read_account_state<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        account: &UInt160,
    ) -> Option<Vec<u8>> {
        let key = Self::account_key(account);
        snapshot
            .get(&key)
            .map(|item| item.value_bytes().into_owned())
    }

    /// Reads the ATC balance from the ATC-specific account state.
    pub(crate) fn balance_of<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        account: &UInt160,
    ) -> CoreResult<BigInt> {
        let Some(bytes) = self.read_account_state(snapshot, account) else {
            return Ok(BigInt::from(0));
        };
        Ok(Self::decode_atipicial_account_state(&bytes)?.balance)
    }
}
