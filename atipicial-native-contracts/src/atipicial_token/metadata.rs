use atipicial_config::Hardfork;
use atipicial_execution::{NativeEvent, NativeMethod};
use atipicial_primitives::{CallFlags, ContractParameterType};
use std::sync::LazyLock;

use super::{
    ATC_CANDIDATE_STATE_CHANGED_EVENT, ATC_COMMITTEE_CHANGED_EVENT, ATC_VOTE_EVENT, AtipicialCoin,
};
use crate::support::invoke::{NativeMethodBinding, method_metadata};

pub(super) fn atipicial_coin_method_bindings<P, D, B>()
-> Vec<NativeMethodBinding<AtipicialCoin, P, D, B>>
where
    P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
    D: atipicial_execution::Diagnostic + 'static,
    B: atipicial_storage::CacheRead,
{
    let read_states = CallFlags::READ_STATES.bits();
    let int = ContractParameterType::Integer;
    vec![
        // AEP-17 metadata: `[ContractMethod]` with no CpuFee -> fee 0, no flags.
        NativeMethodBinding::new(crate::aep17_symbol_method(), AtipicialCoin::invoke_symbol),
        NativeMethodBinding::new(
            crate::aep17_decimals_method(),
            AtipicialCoin::invoke_decimals,
        ),
        // AEP-17 state reads: CpuFee 1<<15, RequiredCallFlags ReadStates.
        NativeMethodBinding::new(
            crate::aep17_total_supply_method(read_states),
            AtipicialCoin::invoke_total_supply,
        ),
        NativeMethodBinding::new(
            crate::aep17_balance_of_method(read_states),
            AtipicialCoin::invoke_balance_of,
        ),
        // AEP-17 transfer(from, to, amount, data) -> Boolean (CpuFee 1<<17,
        // States|AllowCall|AllowNotify; ATC governance runs in OnBalanceChanging).
        NativeMethodBinding::new(
            crate::aep17_transfer_method(),
            AtipicialCoin::invoke_transfer,
        ),
        // Governance reads.
        NativeMethodBinding::new(
            NativeMethod::new(
                "getAtipicialDollarPerBlock",
                1 << 15,
                true,
                read_states,
                vec![],
                int,
            ),
            AtipicialCoin::invoke_get_gas_per_block,
        ),
        NativeMethodBinding::new(
            NativeMethod::new("getRegisterPrice", 1 << 15, true, read_states, vec![], int),
            AtipicialCoin::invoke_get_register_price,
        ),
        // Committee reads (CpuFee 1<<16 in C#).
        NativeMethodBinding::new(
            NativeMethod::new(
                "getCommittee",
                1 << 16,
                true,
                read_states,
                vec![],
                ContractParameterType::Array,
            ),
            AtipicialCoin::invoke_get_committee,
        ),
        NativeMethodBinding::new(
            NativeMethod::new(
                "getCommitteeAddress",
                1 << 16,
                true,
                read_states,
                vec![],
                ContractParameterType::Hash160,
            )
            .with_active_in(Hardfork::HfCockatrice),
            AtipicialCoin::invoke_get_committee_address,
        ),
        // getAccountState(account) -> AtipicialAccountState struct (Array) or null.
        NativeMethodBinding::new(
            NativeMethod::new(
                "getAccountState",
                1 << 15,
                true,
                read_states,
                vec![ContractParameterType::Hash160],
                ContractParameterType::Array,
            )
            .with_parameter_names(["account"]),
            AtipicialCoin::invoke_get_account_state,
        ),
        // unclaimedGas(account, end) -> Integer (CpuFee 1<<17, ReadStates).
        NativeMethodBinding::new(
            NativeMethod::new(
                "unclaimedGas",
                1 << 17,
                true,
                read_states,
                vec![ContractParameterType::Hash160, int],
                int,
            )
            .with_parameter_names(["account", "end"]),
            AtipicialCoin::invoke_unclaimed_atipicial_dollar,
        ),
        // getNextBlockValidators -> ECPoint[] (Array), CpuFee 1<<16 in C#.
        NativeMethodBinding::new(
            NativeMethod::new(
                "getNextBlockValidators",
                1 << 16,
                true,
                read_states,
                vec![],
                ContractParameterType::Array,
            ),
            AtipicialCoin::invoke_get_next_block_validators,
        ),
        // getCandidates -> (ECPoint, BigInteger)[] (Array of Structs), CpuFee 1<<22.
        NativeMethodBinding::new(
            NativeMethod::new(
                "getCandidates",
                1 << 22,
                true,
                read_states,
                vec![],
                ContractParameterType::Array,
            ),
            AtipicialCoin::invoke_get_candidates,
        ),
        // getAllCandidates -> iterator over the registered candidates
        // (InteropInterface), CpuFee 1<<22, ReadStates (AtipicialCoin.cs:537).
        NativeMethodBinding::new(
            NativeMethod::new(
                "getAllCandidates",
                1 << 22,
                true,
                read_states,
                vec![],
                ContractParameterType::InteropInterface,
            ),
            AtipicialCoin::invoke_get_all_candidates,
        ),
        // getCandidateVote(pubKey) -> votes, or -1 if not a registered
        // candidate. (C# parameter is `ECPoint pubKey` — capital K, unlike
        // registerCandidate's `pubkey`.)
        NativeMethodBinding::new(
            NativeMethod::new(
                "getCandidateVote",
                1 << 15,
                true,
                read_states,
                vec![ContractParameterType::PublicKey],
                int,
            )
            .with_parameter_names(["pubKey"]),
            AtipicialCoin::invoke_get_candidate_vote,
        ),
        // Governance writers (committee-gated, States, Void; C# CpuFee 1<<15).
        NativeMethodBinding::new(
            NativeMethod::new(
                "setRegisterPrice",
                1 << 15,
                false,
                CallFlags::STATES.bits(),
                vec![ContractParameterType::Integer],
                ContractParameterType::Void,
            )
            .with_parameter_names(["registerPrice"]),
            AtipicialCoin::invoke_set_register_price,
        ),
        NativeMethodBinding::new(
            NativeMethod::new(
                "setAtipicialDollarPerBlock",
                1 << 15,
                false,
                CallFlags::STATES.bits(),
                vec![ContractParameterType::Integer],
                ContractParameterType::Void,
            )
            .with_parameter_names(["gasPerBlock"]),
            AtipicialCoin::invoke_set_gas_per_block,
        ),
        // Candidate registration (Echidna V1: States|AllowNotify). registerCandidate
        // has no manifest CpuFee (it charges GetRegisterPrice dynamically);
        // unregisterCandidate is CpuFee 1<<16. Both return Boolean.
        // registerCandidate / unregisterCandidate / vote are each a dual
        // registration (C# AtipicialCoin.cs:397/431/456): V0 is genesis-active with
        // RequiredCallFlags=States and DeprecatedIn=HF_Echidna; V1 is
        // ActiveIn=HF_Echidna and adds AllowNotify (the candidate-state-change
        // notification). Exactly one is active at any height.
        NativeMethodBinding::new(
            NativeMethod::new(
                "registerCandidate",
                0,
                false,
                CallFlags::STATES.bits(),
                vec![ContractParameterType::PublicKey],
                ContractParameterType::Boolean,
            )
            .with_parameter_names(["pubkey"])
            .with_deprecated_in(Hardfork::HfEchidna),
            AtipicialCoin::invoke_register_candidate,
        ),
        NativeMethodBinding::new(
            NativeMethod::new(
                "registerCandidate",
                0,
                false,
                CallFlags::STATES.bits() | CallFlags::ALLOW_NOTIFY.bits(),
                vec![ContractParameterType::PublicKey],
                ContractParameterType::Boolean,
            )
            .with_parameter_names(["pubkey"])
            .with_active_in(Hardfork::HfEchidna),
            AtipicialCoin::invoke_register_candidate,
        ),
        NativeMethodBinding::new(
            NativeMethod::new(
                "unregisterCandidate",
                1 << 16,
                false,
                CallFlags::STATES.bits(),
                vec![ContractParameterType::PublicKey],
                ContractParameterType::Boolean,
            )
            .with_parameter_names(["pubkey"])
            .with_deprecated_in(Hardfork::HfEchidna),
            AtipicialCoin::invoke_unregister_candidate,
        ),
        NativeMethodBinding::new(
            NativeMethod::new(
                "unregisterCandidate",
                1 << 16,
                false,
                CallFlags::STATES.bits() | CallFlags::ALLOW_NOTIFY.bits(),
                vec![ContractParameterType::PublicKey],
                ContractParameterType::Boolean,
            )
            .with_parameter_names(["pubkey"])
            .with_active_in(Hardfork::HfEchidna),
            AtipicialCoin::invoke_unregister_candidate,
        ),
        // vote(account, voteTo?) -> Boolean. voteTo is a nullable PublicKey
        // (null = clear the vote). V0 States / V1 States|AllowNotify at Echidna.
        NativeMethodBinding::new(
            NativeMethod::new(
                "vote",
                1 << 16,
                false,
                CallFlags::STATES.bits(),
                vec![
                    ContractParameterType::Hash160,
                    ContractParameterType::PublicKey,
                ],
                ContractParameterType::Boolean,
            )
            .with_parameter_names(["account", "voteTo"])
            .with_deprecated_in(Hardfork::HfEchidna),
            AtipicialCoin::invoke_vote,
        ),
        NativeMethodBinding::new(
            NativeMethod::new(
                "vote",
                1 << 16,
                false,
                CallFlags::STATES.bits() | CallFlags::ALLOW_NOTIFY.bits(),
                vec![
                    ContractParameterType::Hash160,
                    ContractParameterType::PublicKey,
                ],
                ContractParameterType::Boolean,
            )
            .with_parameter_names(["account", "voteTo"])
            .with_active_in(Hardfork::HfEchidna),
            AtipicialCoin::invoke_vote,
        ),
        // onAEP17Payment(from, amount, data) -> Void: candidate registration
        // by paying the register price in GAS to the ATC contract. C#
        // `[ContractMethod(Hardfork.HF_Echidna, RequiredCallFlags =
        // CallFlags.States | CallFlags.AllowNotify)]` with no CpuFee
        // (AtipicialCoin.cs:374).
        NativeMethodBinding::new(
            crate::aep17_payment_method(
                0,
                false,
                CallFlags::STATES.bits() | CallFlags::ALLOW_NOTIFY.bits(),
            )
            .with_active_in(Hardfork::HfEchidna),
            AtipicialCoin::invoke_on_aep17_payment,
        ),
    ]
}

pub(super) static ATC_TOKEN_METHODS: LazyLock<Vec<NativeMethod>> = LazyLock::new(|| {
    method_metadata(&atipicial_coin_method_bindings::<
        atipicial_execution::native_contract_provider::NoNativeContractProvider,
        atipicial_execution::NoDiagnostic,
        atipicial_storage::EmptyCacheBacking,
    >())
});

/// ATC's `[ContractEvent]` declarations (AtipicialCoin.cs:63-74) plus the inherited
/// `FungibleToken.Transfer` at order 0. C# concatenates the contract
/// constructor's attributes with the base type's and sorts by order, so the
/// manifest lists Transfer, CandidateStateChanged, Vote, CommitteeChanged.
pub(super) static ATC_TOKEN_EVENTS: LazyLock<Vec<NativeEvent>> = LazyLock::new(|| {
    vec![
        crate::fungible_token_transfer_event(),
        NativeEvent::new(
            1,
            ATC_CANDIDATE_STATE_CHANGED_EVENT,
            &[
                ("pubkey", ContractParameterType::PublicKey),
                ("registered", ContractParameterType::Boolean),
                ("votes", ContractParameterType::Integer),
            ],
        ),
        NativeEvent::new(
            2,
            ATC_VOTE_EVENT,
            &[
                ("account", ContractParameterType::Hash160),
                ("from", ContractParameterType::PublicKey),
                ("to", ContractParameterType::PublicKey),
                ("amount", ContractParameterType::Integer),
            ],
        ),
        NativeEvent::new(
            3,
            ATC_COMMITTEE_CHANGED_EVENT,
            &[
                ("old", ContractParameterType::Array),
                ("new", ContractParameterType::Array),
            ],
        )
        .with_active_in(Hardfork::HfCockatrice),
    ]
});
