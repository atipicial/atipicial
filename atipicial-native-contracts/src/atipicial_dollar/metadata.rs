use std::sync::LazyLock;

use atipicial_execution::{NativeEvent, NativeMethod};
use atipicial_primitives::CallFlags;

use super::AtipicialDollar;
use crate::support::invoke::{NativeMethodBinding, method_metadata};

pub(super) fn atipicial_dollar_method_bindings<P, D, B>()
-> Vec<NativeMethodBinding<AtipicialDollar, P, D, B>>
where
    P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
    D: atipicial_execution::Diagnostic + 'static,
    B: atipicial_storage::CacheRead,
{
    let read_states = CallFlags::READ_STATES.bits();
    vec![
        // AEP-17 metadata: `[ContractMethod]` with no CpuFee -> fee 0, no flags.
        NativeMethodBinding::new(crate::aep17_symbol_method(), AtipicialDollar::invoke_symbol),
        NativeMethodBinding::new(
            crate::aep17_decimals_method(),
            AtipicialDollar::invoke_decimals,
        ),
        // AEP-17 state reads: CpuFee 1<<15, RequiredCallFlags ReadStates.
        NativeMethodBinding::new(
            crate::aep17_total_supply_method(read_states),
            AtipicialDollar::invoke_total_supply,
        ),
        NativeMethodBinding::new(
            crate::aep17_balance_of_method(read_states),
            AtipicialDollar::invoke_balance_of,
        ),
        // AEP-17 transfer: CpuFee 1<<17, StorageFee 50, States|AllowCall|AllowNotify,
        // (from, to, amount, data) -> Boolean. Not safe.
        NativeMethodBinding::new(
            crate::aep17_transfer_method(),
            AtipicialDollar::invoke_transfer,
        ),
    ]
}

pub(super) static GAS_TOKEN_METHODS: LazyLock<Vec<NativeMethod>> = LazyLock::new(|| {
    method_metadata(&atipicial_dollar_method_bindings::<
        atipicial_execution::native_contract_provider::NoNativeContractProvider,
        atipicial_execution::NoDiagnostic,
        atipicial_storage::EmptyCacheBacking,
    >())
});

/// GAS declares no events of its own; the only manifest event is the
/// `Transfer` inherited from the C# `FungibleToken` base constructor.
pub(super) static GAS_TOKEN_EVENTS: LazyLock<Vec<NativeEvent>> =
    LazyLock::new(|| vec![crate::fungible_token_transfer_event()]);
