//! ATD transfer, mint, and burn helpers.
//!
//! Keeps AEP-17 accounting and notification/callback behavior separate from the
//! AtipicialDollar root and block-persist hooks.

use super::AtipicialDollar;
use atipicial_error::{CoreError, CoreResult};
use atipicial_execution::ApplicationEngine;
use atipicial_primitives::UInt160;
use atipicial_storage::StorageItem;
use atipicial_vm::StackItem;
use num_bigint::BigInt;
use num_traits::Zero;

/// The balance outcome of a ATD transfer (no governance - `OnBalanceChanging`
/// is a no-op for GAS).
#[derive(Debug, PartialEq, Eq)]
pub(super) enum GasTransferOutcome {
    /// `amount > 0` but the from-account is absent or under-funded: return
    /// `false`, no state change.
    InsufficientBalance,
    /// No balance movement (`amount == 0`, or `from == to`): succeed and still
    /// emit, but write no balances.
    NoMovement,
    /// Deduct `amount` from `from` (delete its entry when `delete_from`) and
    /// credit `to` with `to_new`.
    Move {
        from_new: BigInt,
        delete_from: bool,
        to_new: BigInt,
    },
}

impl AtipicialDollar {
    /// Pure GAS-transfer balance arithmetic, mirroring C#
    /// `FungibleToken.Transfer` with a no-op `OnBalanceChanging`. `amount` is
    /// assumed non-negative (the caller faults on negative).
    /// `from_balance`/`to_balance` are the current balances (`None` = no
    /// account entry).
    pub(in crate::atipicial_dollar) fn compute_gas_transfer(
        from_balance: Option<BigInt>,
        to_balance: Option<BigInt>,
        same_account: bool,
        amount: &BigInt,
    ) -> GasTransferOutcome {
        if amount.is_zero() {
            // C#: amount == 0 -> OnBalanceChanging no-op; no balance movement.
            return GasTransferOutcome::NoMovement;
        }
        let Some(from_bal) = from_balance else {
            return GasTransferOutcome::InsufficientBalance; // storageFrom is null
        };
        if &from_bal < amount {
            return GasTransferOutcome::InsufficientBalance;
        }
        if same_account {
            // C#: from == to -> OnBalanceChanging(0); no net movement.
            return GasTransferOutcome::NoMovement;
        }
        let from_new = &from_bal - amount;
        let delete_from = from_new.is_zero(); // C#: if stateFrom.Balance == amount -> Delete.
        let to_new = to_balance.unwrap_or_else(BigInt::zero) + amount;
        GasTransferOutcome::Move {
            from_new,
            delete_from,
            to_new,
        }
    }

    /// C# `FungibleToken.PostTransferAsync` for the transfer case (both
    /// `from`/`to` non-null): emit the `Transfer` event, then - only when `to`
    /// is a deployed contract - queue its `onAEP17Payment(from, amount, data)`
    /// callback (run after this native call returns, faithful to
    /// `CallFromNativeContractAsync`).
    fn post_transfer<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        from: &UInt160,
        to: &UInt160,
        amount: &BigInt,
        data: &[u8],
    ) -> CoreResult<()> {
        let gas_hash = AtipicialDollar::script_hash();
        engine
            .send_notification(
                gas_hash,
                crate::AEP17_TRANSFER_EVENT.to_owned(),
                crate::aep17_transfer_notification_state(Some(from), Some(to), amount),
            )
            .map_err(|e| {
                CoreError::invalid_operation(format!("AtipicialDollar::transfer notify: {e}"))
            })?;

        let is_contract = crate::ContractManagement::is_contract(&engine.snapshot_cache(), to);
        if !is_contract {
            return Ok(());
        }
        // The transfer `data` param is `Any`, so it arrives BinarySerialized;
        // round-trip it back to the StackItem passed to onAEP17Payment.
        let data_item = crate::aep17_payment_data_item(data, "AtipicialDollar::transfer data")?;
        engine.queue_contract_call_from_native(
            gas_hash,
            *to,
            crate::AEP17_PAYMENT_METHOD,
            crate::aep17_payment_callback_args(Some(from), amount, data_item),
        );
        Ok(())
    }

    /// C# `AtipicialDollar.Mint` (`FungibleToken.Mint`): credits `amount` GAS to
    /// `account`, raises the total supply, and emits
    /// `Transfer(null, account, amount)` - queuing the recipient's
    /// `onAEP17Payment` when `call_on_payment` and the recipient is a deployed
    /// contract. A zero amount is a no-op; a negative amount faults.
    /// `pub(crate)` so AtipicialCoin's reward distribution can mint GAS.
    pub(crate) fn gas_mint<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        account: &UInt160,
        amount: &BigInt,
        call_on_payment: bool,
    ) -> CoreResult<()> {
        if amount < &BigInt::zero() {
            return Err(CoreError::invalid_operation(
                "AtipicialDollar::mint: amount cannot be negative",
            ));
        }
        if amount.is_zero() {
            return Ok(());
        }
        let snapshot = engine.snapshot_cache();
        let balance = self
            .read_gas_account(&snapshot, account)?
            .unwrap_or_else(BigInt::zero)
            + amount;
        self.write_gas_account(&snapshot, account, &balance)?;
        let supply_key = Self::total_supply_key();
        let supply = Self::total_supply(&snapshot) + amount;
        snapshot.update(
            supply_key,
            StorageItem::from_bytes(crate::bigint_to_storage_bytes(&supply)),
        );
        self.post_mint(engine, account, amount, call_on_payment)
    }

    /// C# `PostTransferAsync(null, account, amount, ...)` for the mint case:
    /// emit `Transfer(null, account, amount)`, then queue
    /// `onAEP17Payment(null, amount, null)` when `call_on_payment` and the
    /// recipient is a deployed contract.
    fn post_mint<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        account: &UInt160,
        amount: &BigInt,
        call_on_payment: bool,
    ) -> CoreResult<()> {
        let gas_hash = AtipicialDollar::script_hash();
        engine
            .send_notification(
                gas_hash,
                crate::AEP17_TRANSFER_EVENT.to_owned(),
                crate::aep17_transfer_notification_state(None, Some(account), amount),
            )
            .map_err(|e| {
                CoreError::invalid_operation(format!("AtipicialDollar::mint notify: {e}"))
            })?;
        if !call_on_payment {
            return Ok(());
        }
        if !crate::ContractManagement::is_contract(&engine.snapshot_cache(), account) {
            return Ok(());
        }
        engine.queue_contract_call_from_native(
            gas_hash,
            *account,
            crate::AEP17_PAYMENT_METHOD,
            crate::aep17_payment_callback_args(None, amount, StackItem::null()),
        );
        Ok(())
    }

    /// C# `AtipicialDollar.Burn` (`FungibleToken.Burn`): debits `amount` GAS from
    /// `account` (deleting the entry when the balance reaches zero, like C#'s
    /// `Delete` on `Balance == amount`), lowers the total supply, and emits
    /// `Transfer(account, null, amount)` - no `onAEP17Payment` callback (C#
    /// burns with `callOnPayment: false`). A zero amount is a no-op; a negative
    /// amount or an insufficient balance faults (C# throws on `Balance <
    /// amount`, and a missing account entry NREs on the null-forgiving
    /// `GetAndChange`). `pub(crate)` so AtipicialCoin's Echidna `onAEP17Payment` can
    /// burn the register-price GAS it receives.
    pub(crate) fn gas_burn<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        account: &UInt160,
        amount: &BigInt,
    ) -> CoreResult<()> {
        if amount < &BigInt::zero() {
            return Err(CoreError::invalid_operation(
                "AtipicialDollar::burn: amount cannot be negative",
            ));
        }
        if amount.is_zero() {
            return Ok(());
        }
        let snapshot = engine.snapshot_cache();
        let balance = self
            .read_gas_account(&snapshot, account)?
            .unwrap_or_else(BigInt::zero);
        if &balance < amount {
            return Err(CoreError::invalid_operation(format!(
                "AtipicialDollar::burn: insufficient balance {balance} to burn {amount}"
            )));
        }
        if &balance == amount {
            self.delete_gas_account(&snapshot, account);
        } else {
            self.write_gas_account(&snapshot, account, &(&balance - amount))?;
        }
        let supply_key = Self::total_supply_key();
        let supply = Self::total_supply(&snapshot) - amount;
        snapshot.update(
            supply_key,
            StorageItem::from_bytes(crate::bigint_to_storage_bytes(&supply)),
        );
        engine
            .send_notification(
                AtipicialDollar::script_hash(),
                crate::AEP17_TRANSFER_EVENT.to_owned(),
                crate::aep17_transfer_notification_state(Some(account), None, amount),
            )
            .map_err(|e| CoreError::invalid_operation(format!("AtipicialDollar::burn notify: {e}")))
    }

    /// Core AEP-17 transfer (C# `FungibleToken.Transfer`), shared by the
    /// `transfer` ABI method and by native-to-native callers (e.g.
    /// `Notary.withdraw`).
    ///
    /// `caller` is the script hash treated as the immediate caller for the
    /// witness bypass - C# `from.Equals(CallingScriptHash)`. The ABI method
    /// passes the engine's calling script hash; a native contract transferring
    /// its own balance (where C#'s nested call would set `CallingScriptHash` to
    /// that contract) passes its own hash, so the witness check passes. Returns
    /// `Ok(true)`/`Ok(false)` per AEP-17, or `Err` for a negative `amount`.
    pub(crate) fn transfer_core<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        engine: &mut ApplicationEngine<P, D, B>,
        caller: UInt160,
        from: &UInt160,
        to: &UInt160,
        amount: &BigInt,
        data: &[u8],
    ) -> CoreResult<bool> {
        // C#: amount.Sign < 0 -> ArgumentOutOfRangeException (fault).
        if amount < &BigInt::zero() {
            return Err(CoreError::invalid_operation(
                "AtipicialDollar::transfer: amount cannot be negative",
            ));
        }
        // C#: !from.Equals(CallingScriptHash) && !CheckWitnessInternal(from) -> false.
        let witnessed = from == &caller
            || engine.check_witness(from).map_err(|e| {
                CoreError::invalid_operation(format!("AtipicialDollar::transfer witness: {e}"))
            })?;
        if !witnessed {
            return Ok(false);
        }

        let snapshot = engine.snapshot_cache();
        let from_balance = AtipicialDollar::new().read_gas_account(&snapshot, from)?;
        let to_balance = AtipicialDollar::new().read_gas_account(&snapshot, to)?;
        match Self::compute_gas_transfer(from_balance, to_balance, from == to, amount) {
            GasTransferOutcome::InsufficientBalance => return Ok(false),
            GasTransferOutcome::NoMovement => {}
            GasTransferOutcome::Move {
                from_new,
                delete_from,
                to_new,
            } => {
                if delete_from {
                    AtipicialDollar::new().delete_gas_account(&snapshot, from);
                } else {
                    AtipicialDollar::new().write_gas_account(&snapshot, from, &from_new)?;
                }
                AtipicialDollar::new().write_gas_account(&snapshot, to, &to_new)?;
            }
        }

        // PostTransfer: emit Transfer + (if `to` is a contract) onAEP17Payment.
        AtipicialDollar::new().post_transfer(engine, from, to, amount, data)?;
        Ok(true)
    }
}
