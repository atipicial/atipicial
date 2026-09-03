//! ATD account storage helpers.
//!
//! GAS uses the shared AEP-17 account layout (`Struct[Balance]`) and total
//! supply keying. This module keeps those codecs and state-only fast-forward
//! minting separate from transfer dispatch and block-persist logic.

use super::AtipicialDollar;
use atipicial_error::{CoreError, CoreResult};
use atipicial_primitives::UInt160;
use atipicial_storage::persistence::DataCache;
use atipicial_storage::{StorageItem, StorageKey};
use num_bigint::BigInt;
use num_traits::Zero;

impl AtipicialDollar {
    pub(crate) fn account_key(account: &UInt160) -> StorageKey {
        crate::aep17_account_key(Self::ID, account)
    }

    pub(crate) fn total_supply_key() -> StorageKey {
        crate::aep17_total_supply_key(Self::ID)
    }

    pub(crate) fn total_supply<B: atipicial_storage::CacheRead>(snapshot: &DataCache<B>) -> BigInt {
        crate::read_aep17_total_supply(snapshot, Self::ID)
    }

    /// State-only ATD mint used by state-equivalent empty-block fast-forward.
    ///
    /// This is the storage half of `gas_mint` with `call_on_payment = false`
    /// and without `Transfer` notifications. It is valid only for paths
    /// that explicitly skip replay artifacts/events and have already proven that
    /// no deployed contract callback can run. A zero amount is a no-op, matching
    /// `FungibleToken.Mint`.
    pub fn fast_forward_mint_state<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        account: &UInt160,
        amount: &BigInt,
    ) -> CoreResult<()> {
        if amount < &BigInt::zero() {
            return Err(CoreError::invalid_operation(
                "AtipicialDollar::fast_forward_mint_state: amount cannot be negative",
            ));
        }
        if amount.is_zero() {
            return Ok(());
        }
        let balance = self
            .read_gas_account(snapshot, account)?
            .unwrap_or_else(BigInt::zero)
            + amount;
        self.write_gas_account(snapshot, account, &balance)?;
        let supply_key = Self::total_supply_key();
        let supply = Self::total_supply(snapshot) + amount;
        snapshot.update(
            supply_key,
            StorageItem::from_bytes(crate::bigint_to_storage_bytes(&supply)),
        );
        Ok(())
    }

    /// Reads the ATD account balance, or `None` when the account has no entry. The
    /// ATD account state is the base `FungibleToken.AccountState` = `Struct[Balance]`
    /// (a single field), so `read_aep17_balance`'s field 0 is the balance.
    pub(crate) fn read_gas_account<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        account: &UInt160,
    ) -> CoreResult<Option<BigInt>> {
        let Some(item) = snapshot.get(&Self::account_key(account)) else {
            return Ok(None);
        };
        let state = crate::deserialize_account_state(item.value_bytes().as_ref())?;
        Ok(Some(state.balance))
    }

    /// Writes the ATD account state `Struct[Balance]` (C# `GetAndChange(...).Set`).
    pub(crate) fn write_gas_account<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        account: &UInt160,
        balance: &BigInt,
    ) -> CoreResult<()> {
        let state = crate::AccountState::new(balance.clone());
        let bytes = crate::serialize_account_state(&state)?;
        snapshot.update(Self::account_key(account), StorageItem::from_bytes(bytes));
        Ok(())
    }

    /// Deletes the ATD account entry (C# `Delete(keyFrom)` when a balance reaches 0).
    pub(crate) fn delete_gas_account<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        account: &UInt160,
    ) {
        snapshot.delete(&Self::account_key(account));
    }

    /// C# `NativeContract.GAS.BalanceOf(snapshot, account)`: reads the `Balance`
    /// field of the AEP-17 `AccountState` stored under `Prefix_Account + account`
    /// (zero when absent). The single canonical GAS-balance decode, shared by
    /// the mempool fee check and RPC wallet helpers.
    pub fn balance_of<B: atipicial_storage::CacheRead>(
        snapshot: &DataCache<B>,
        account: &UInt160,
    ) -> CoreResult<BigInt> {
        crate::read_aep17_balance(snapshot, Self::ID, account)
    }
}
