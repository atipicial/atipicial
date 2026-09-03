//! AtipicialCoin native-method handlers.
//!
//! Keeps governance, voting, transfer, and candidate-registration method bodies
//! out of the contract root while preserving C#-compatible validation order,
//! fee accounting, storage writes, notifications, and payment-callback
//! semantics. Dispatch is declared by the metadata binding table and
//! `native_contract_dispatch!`.

use atipicial_config::Hardfork;
use atipicial_crypto::ECPoint;
use atipicial_error::{CoreError, CoreResult};
use atipicial_execution::ApplicationEngine;
use atipicial_primitives::{FindOptions, UInt160};
use atipicial_vm::{Contract, StackItem};
use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::LedgerContract;

use super::{ATC_CANDIDATE_STATE_CHANGED_EVENT, ATC_TOTAL_AMOUNT, AtipicialCoin};

impl AtipicialCoin {
    pub(super) fn invoke_symbol<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        _engine: &mut ApplicationEngine<P, D, B>,
        _args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        Ok(Self::SYMBOL.as_bytes().to_vec())
    }

    pub(super) fn invoke_decimals<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        _engine: &mut ApplicationEngine<P, D, B>,
        _args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        Ok(BigInt::from(Self::DECIMALS).to_signed_bytes_le())
    }

    pub(super) fn invoke_total_supply<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        _engine: &mut ApplicationEngine<P, D, B>,
        _args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        // C# `AtipicialCoin.TotalSupply` overrides the fungible-token storage
        // reader and returns the immutable protocol amount.
        Ok(BigInt::from(ATC_TOTAL_AMOUNT).to_signed_bytes_le())
    }

    pub(super) fn invoke_balance_of<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        let account = crate::args::raw_account(args, "AtipicialCoin::balanceOf")?;
        let snapshot = engine.snapshot_cache();
        Ok(self.balance_of(&snapshot, &account)?.to_signed_bytes_le())
    }

    pub(super) fn invoke_transfer<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        // C# FungibleToken.Transfer(from, to, amount, data) with ATC's
        // governance OnBalanceChanging side-effects.
        let from = crate::args::raw_hash160(args, 0, "AtipicialCoin::transfer")?;
        let to = crate::args::raw_hash160(args, 1, "AtipicialCoin::transfer")?;
        let amount =
            crate::args::raw_required_integer_arg(args, 2, "AtipicialCoin::transfer", "an amount")?;
        let data = args.get(3).map(Vec::as_slice).unwrap_or(&[]);
        let caller = engine
            .get_calling_script_hash()
            .unwrap_or_else(UInt160::zero);
        Ok(vec![u8::from(self.atipicial_transfer_core(
            engine, caller, &from, &to, &amount, data,
        )?)])
    }

    pub(super) fn invoke_get_gas_per_block<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        _args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        let snapshot = engine.snapshot_cache();
        let index = LedgerContract::new()
            .current_index(&snapshot)?
            .saturating_add(1);
        Ok(self.gas_per_block_at(&snapshot, index).to_signed_bytes_le())
    }

    pub(super) fn invoke_get_register_price<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        _args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        let snapshot = engine.snapshot_cache();
        Ok(BigInt::from(self.register_price(&snapshot)?).to_signed_bytes_le())
    }

    pub(super) fn invoke_set_register_price<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        // C#: validate registerPrice > 0 -> AssertCommittee -> overwrite
        // Prefix_RegisterPrice.
        let price = args
            .first()
            .map(|b| BigInt::from_signed_bytes_le(b))
            .and_then(|b| b.to_i64())
            .ok_or_else(|| {
                CoreError::invalid_operation("AtipicialCoin::setRegisterPrice requires a price")
            })?;
        if price <= 0 {
            return Err(CoreError::invalid_operation(format!(
                "RegisterPrice must be positive, got {price}"
            )));
        }
        crate::committee::assert_committee(engine, "setRegisterPrice")?;
        self.put_register_price(&engine.snapshot_cache(), price)?;
        Ok(Vec::new())
    }

    pub(super) fn invoke_set_gas_per_block<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        // C#: validate 0 <= gasPerBlock <= 10*GAS.Factor -> AssertCommittee
        // -> write a Prefix_AtipicialDollarPerBlock record at (persisting index + 1).
        let gas_per_block = args
            .first()
            .map(|b| BigInt::from_signed_bytes_le(b))
            .ok_or_else(|| {
                CoreError::invalid_operation(
                    "AtipicialCoin::setAtipicialDollarPerBlock requires a value",
                )
            })?;
        // GAS.Factor = 10^8; the inclusive upper bound is 10 GAS.
        let max = BigInt::from(10) * BigInt::from(100_000_000i64);
        if gas_per_block < BigInt::from(0) || gas_per_block > max {
            return Err(CoreError::invalid_operation(format!(
                "AtipicialDollarPerBlock must be between [0, {max}]"
            )));
        }
        crate::committee::assert_committee(engine, "setAtipicialDollarPerBlock")?;
        // C# `engine.PersistingBlock!.Index + 1`: the method runs during
        // block persistence, so a missing persisting block is a fault
        // (matching the C# null-forgiving deref throwing on null).
        let index = engine
            .persisting_block()
            .map(|b| b.index())
            .ok_or_else(|| {
                CoreError::invalid_operation(
                    "AtipicialCoin::setAtipicialDollarPerBlock requires a persisting block",
                )
            })?
            .checked_add(1)
            .ok_or_else(|| {
                CoreError::invalid_operation(
                    "AtipicialCoin::setAtipicialDollarPerBlock: block index overflow",
                )
            })?;
        self.put_gas_per_block(&engine.snapshot_cache(), index, &gas_per_block);
        Ok(Vec::new())
    }

    pub(super) fn invoke_get_committee<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        _args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        // C# returns ECPoint[] sorted ascending; marshaled as an Array of
        // compressed (33-byte) public-key byte strings.
        let snapshot = engine.snapshot_cache();
        Self::points_to_array_bytes(&self.committee_sorted(&snapshot)?)
    }

    pub(super) fn invoke_get_next_block_validators<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        _args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        // First ValidatorsCount committee members (stored order), sorted.
        let count = usize::try_from(engine.protocol_settings().validators_count).unwrap_or(0);
        let snapshot = engine.snapshot_cache();
        Self::points_to_array_bytes(&self.next_block_validators(&snapshot, count)?)
    }

    pub(super) fn invoke_get_candidates<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        _args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        let snapshot = engine.snapshot_cache();
        // C# `GetCandidatesInternal().Select(...).Take(256).ToArray()`
        // (AtipicialCoin.cs:528): at most the first 256 registered candidates.
        let mut candidates = self.read_registered_candidates(&snapshot)?;
        candidates.truncate(256);
        Self::candidates_to_array_bytes(&candidates)
    }

    pub(super) fn invoke_get_candidate_vote<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        let pubkey_bytes = args.first().ok_or_else(|| {
            CoreError::invalid_operation("AtipicialCoin::getCandidateVote requires a public key")
        })?;
        // C# takes an ECPoint; an invalid key faults at marshaling.
        let pubkey = ECPoint::from_bytes(pubkey_bytes).map_err(|e| {
            CoreError::invalid_operation(format!(
                "AtipicialCoin::getCandidateVote: bad public key: {e}"
            ))
        })?;
        let snapshot = engine.snapshot_cache();
        Ok(self
            .candidate_vote(&snapshot, &pubkey)?
            .to_signed_bytes_le())
    }

    pub(super) fn invoke_register_candidate<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        // C# RegisterCandidate: pre-HF_Echidna a failed witness returns
        // false WITHOUT charging the register fee (the early check is
        // skipped from Echidna because RegisterInternal repeats it); the
        // fee is charged only after that gate. RegisterInternal then
        // (re)checks the witness, creates/flips the CandidateState to
        // Registered, and emits CandidateStateChanged.
        let pubkey_bytes = args.first().ok_or_else(|| {
            CoreError::invalid_operation("AtipicialCoin::registerCandidate requires a public key")
        })?;
        let pubkey = ECPoint::from_bytes(pubkey_bytes).map_err(|e| {
            CoreError::invalid_operation(format!(
                "AtipicialCoin::registerCandidate: bad public key: {e}"
            ))
        })?;
        // Pre-Echidna only: a missing witness returns false before any fee.
        if !engine.is_hardfork_enabled(Hardfork::HfEchidna) {
            let account =
                UInt160::from_script(&Contract::create_signature_redeem_script(pubkey.clone()));
            let authorized = engine.check_witness_hash(&account).map_err(|e| {
                CoreError::invalid_operation(format!(
                    "AtipicialCoin::registerCandidate: witness: {e}"
                ))
            })?;
            if !authorized {
                return Ok(vec![0]);
            }
        }
        // C# v3.10.1: engine.AddFee(GetRegisterPrice, applyFactor: true).
        let price = self.register_price(&engine.snapshot_cache())?;
        let price = u64::try_from(price).map_err(|_| {
            CoreError::invalid_operation(
                "AtipicialCoin::registerCandidate fee must be non-negative",
            )
        })?;
        engine.charge_execution_fee(price).map_err(|e| {
            CoreError::invalid_operation(format!("AtipicialCoin::registerCandidate: fee: {e}"))
        })?;
        Ok(vec![u8::from(self.register_internal(
            engine,
            &pubkey,
            "registerCandidate",
        )?)])
    }

    pub(super) fn invoke_get_all_candidates<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        _args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        // C# GetAllCandidates (AtipicialCoin.cs:537-545): a StorageIterator
        // over the registered, non-blocked candidate entries with
        // RemovePrefix | DeserializeValues | PickField1 and prefix
        // length 1 — each element is Struct[33-byte pubkey, Votes]. The
        // 4-byte iterator id is decoded back into an InteropInterface
        // by the dispatcher.
        let results = self
            .registered_candidate_entries(&engine.snapshot_cache())?
            .into_iter()
            .map(|(_pubkey, _votes, key, item)| (key, item))
            .collect::<Vec<_>>();
        let iterator_id = engine
            .create_storage_iterator_with_options(
                results,
                1,
                FindOptions::RemovePrefix
                    | FindOptions::DeserializeValues
                    | FindOptions::PickField1,
            )
            .map_err(|e| {
                CoreError::invalid_operation(format!("AtipicialCoin::getAllCandidates: {e}"))
            })?;
        Ok(iterator_id.to_le_bytes().to_vec())
    }

    pub(super) fn invoke_on_aep17_payment<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        // C# AtipicialCoin.OnNEP17Payment (AtipicialCoin.cs:374-389, HF_Echidna):
        // candidate registration by paying the register price in GAS to
        // the ATC contract. The `from` argument is unused — the witness
        // requirement is RegisterInternal's, on the candidate account
        // derived from `data`'s public key.
        if engine.get_calling_script_hash() != Some(crate::AtipicialDollar::script_hash()) {
            return Err(CoreError::invalid_operation(
                "AtipicialCoin::onAEP17Payment: only the ATD contract can call this method",
            ));
        }
        let amount = crate::args::raw_required_integer_arg(
            args,
            1,
            "AtipicialCoin::onAEP17Payment",
            "an amount",
        )?;
        let price = self.register_price(&engine.snapshot_cache())?;
        if amount != BigInt::from(price) {
            return Err(CoreError::invalid_operation(format!(
                "AtipicialCoin::onAEP17Payment: incorrect ATD amount; expected {price}, received {amount}"
            )));
        }
        // `data` is an Any param (it arrives BinarySerialized); C#
        // decodes its span as a secp256r1 point, faulting on anything
        // that is not a valid public key (including Null).
        let data = args.get(2).map(Vec::as_slice).unwrap_or(&[]);
        let item =
            crate::support::codec::decode_stack_item(data, "AtipicialCoin::onAEP17Payment data")?;
        let pubkey_bytes = match item {
            StackItem::ByteString(bytes) => bytes.to_vec(),
            StackItem::Buffer(buffer) => buffer.data(),
            _ => {
                return Err(CoreError::invalid_operation(
                    "AtipicialCoin::onAEP17Payment data: cannot convert to bytes",
                ));
            }
        };
        let pubkey = ECPoint::from_bytes(&pubkey_bytes).map_err(|e| {
            CoreError::invalid_operation(format!(
                "AtipicialCoin::onAEP17Payment: bad public key: {e}"
            ))
        })?;
        if !self.register_internal(engine, &pubkey, crate::AEP17_PAYMENT_METHOD)? {
            return Err(CoreError::invalid_operation(
                "AtipicialCoin::onAEP17Payment: failed to register candidate",
            ));
        }
        // C# `await GAS.Burn(engine, Hash, amount)`: burn the GAS this
        // transfer just credited to the ATC contract's own account.
        crate::AtipicialDollar::new().gas_burn(engine, &Self::script_hash(), &amount)?;
        Ok(Vec::new())
    }

    pub(super) fn invoke_unregister_candidate<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        // C# UnregisterCandidate: witness on the candidate account, flip the
        // CandidateState to unregistered; CheckCandidate deletes the entry
        // once it has no remaining votes.
        let pubkey_bytes = args.first().ok_or_else(|| {
            CoreError::invalid_operation("AtipicialCoin::unregisterCandidate requires a public key")
        })?;
        let pubkey = ECPoint::from_bytes(pubkey_bytes).map_err(|e| {
            CoreError::invalid_operation(format!(
                "AtipicialCoin::unregisterCandidate: bad public key: {e}"
            ))
        })?;
        let account =
            UInt160::from_script(&Contract::create_signature_redeem_script(pubkey.clone()));
        let authorized = engine.check_witness_hash(&account).map_err(|e| {
            CoreError::invalid_operation(format!(
                "AtipicialCoin::unregisterCandidate: witness: {e}"
            ))
        })?;
        if !authorized {
            return Ok(vec![0u8]);
        }
        let snapshot = engine.snapshot_cache();
        let key = Self::candidate_key(&pubkey);
        let Some(item) = snapshot.get(&key) else {
            return Ok(vec![1u8]); // not a candidate -> true
        };
        let (registered, votes) = Self::decode_candidate_state(&item.value_bytes())?;
        if !registered {
            return Ok(vec![1u8]);
        }
        // C# `state.Registered = false; CheckCandidate(snapshot, pubkey,
        // state)` (AtipicialCoin.cs:443,191): flip to unregistered, then when no
        // votes remain delete BOTH the candidate entry and the
        // `Prefix_VoterRewardPerCommittee` entry (otherwise a candidate that
        // accrued committee voter rewards and then lost all votes would leave
        // a stale reward record — a state-root divergence). Retain as
        // unregistered when votes remain.
        self.check_candidate(&snapshot, &pubkey, false, &votes)?;
        // C# UnregisterCandidate (AtipicialCoin.cs:444) sends CandidateStateChanged
        // unconditionally; native SendNotification ignores AllowNotify.
        engine
            .send_notification(
                Self::script_hash(),
                ATC_CANDIDATE_STATE_CHANGED_EVENT.to_owned(),
                vec![
                    StackItem::from_byte_string(pubkey.to_bytes()),
                    StackItem::from_bool(false),
                    StackItem::from_int(votes),
                ],
            )
            .map_err(|e| {
                CoreError::invalid_operation(format!(
                    "AtipicialCoin::unregisterCandidate: notify: {e}"
                ))
            })?;
        Ok(vec![1u8])
    }

    pub(super) fn invoke_vote<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        // C# Vote -> VoteInternal: witness on the voter, then the vote
        // transition (extracted into `vote_internal` so PolicyContract's
        // blockAccount can clear a blocked account's vote the way C#
        // calls `ATC.VoteInternal` directly).
        let account = crate::args::raw_account(args, "AtipicialCoin::vote")?;
        // voteTo is a nullable PublicKey (bit 1 of the arg null-mask).
        let vote_to_is_null = engine.native_arg_is_null(1);
        let vote_to: Option<ECPoint> = if vote_to_is_null {
            None
        } else {
            let bytes = args.get(1).ok_or_else(|| {
                CoreError::invalid_operation("AtipicialCoin::vote requires a candidate (or null)")
            })?;
            Some(ECPoint::from_bytes(bytes).map_err(|e| {
                CoreError::invalid_operation(format!("AtipicialCoin::vote: bad candidate: {e}"))
            })?)
        };
        if !engine.check_witness_hash(&account).map_err(|e| {
            CoreError::invalid_operation(format!("AtipicialCoin::vote: witness: {e}"))
        })? {
            return Ok(vec![0u8]);
        }
        Ok(vec![u8::from(self.vote_internal(
            engine,
            &account,
            vote_to.as_ref(),
        )?)])
    }

    pub(super) fn invoke_get_committee_address<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        _args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        let snapshot = engine.snapshot_cache();
        Ok(self.compute_committee_address(&snapshot)?.to_bytes())
    }

    pub(super) fn invoke_get_account_state<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        let account = crate::args::raw_account(args, "AtipicialCoin::getAccountState")?;
        let snapshot = engine.snapshot_cache();
        // C# returns the AtipicialAccountState struct, or null (empty payload)
        // when the account has no entry.
        Ok(self
            .read_account_state(&snapshot, &account)
            .unwrap_or_default())
    }

    pub(super) fn invoke_unclaimed_atipicial_dollar<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        // C# UnclaimedGas(account, end): `end` must equal the persisting
        // block index (or Ledger.CurrentIndex + 1); compute CalculateBonus
        // for the account's AtipicialAccountState (zero when it has no entry).
        let account = crate::args::raw_account(args, "AtipicialCoin::unclaimedGas")?;
        let end = args
            .get(1)
            .map(|b| BigInt::from_signed_bytes_le(b))
            .and_then(|b| b.to_u32())
            .ok_or_else(|| {
                CoreError::invalid_operation("AtipicialCoin::unclaimedGas requires an end index")
            })?;
        let snapshot = engine.snapshot_cache();
        let expect_end = match engine.persisting_block() {
            Some(block) => block.index(),
            None => LedgerContract::new()
                .current_index(&snapshot)?
                .saturating_add(1),
        };
        if end != expect_end {
            return Err(CoreError::invalid_operation(format!(
                "AtipicialCoin::unclaimedGas: end {end} must equal {expect_end}"
            )));
        }
        let bonus = match self.read_account_state(&snapshot, &account) {
            Some(bytes) => {
                let state = Self::decode_atipicial_account_state(&bytes)?;
                self.calculate_bonus(&snapshot, &state, end)?
            }
            None => BigInt::from(0),
        };
        Ok(bonus.to_signed_bytes_le())
    }
}
