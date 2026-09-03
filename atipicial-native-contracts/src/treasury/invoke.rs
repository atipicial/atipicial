//! Treasury native-method handlers.
//!
//! Keeps AEP payment callbacks and committee-witness verification bodies out of
//! the contract root while preserving C# callback no-op behavior and
//! `Treasury.Verify` witness semantics. Dispatch is declared by the metadata
//! binding table and `native_contract_dispatch!`.

use super::Treasury;
use atipicial_error::CoreResult;
use atipicial_execution::ApplicationEngine;

impl Treasury {
    pub(super) fn invoke_nep_payment<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        _engine: &mut ApplicationEngine<P, D, B>,
        _args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        // Both callbacks are no-ops in C# (empty bodies); they return Void,
        // so an empty payload pushes nothing onto the stack.
        Ok(Vec::new())
    }

    pub(super) fn invoke_verify<
        P: atipicial_execution::native_contract_provider::NativeContractProvider + 'static,
        D: atipicial_execution::Diagnostic + 'static,
        B: atipicial_storage::CacheRead,
    >(
        &self,
        engine: &mut ApplicationEngine<P, D, B>,
        _args: &[Vec<u8>],
    ) -> CoreResult<Vec<u8>> {
        // C# `Treasury.Verify` (Treasury.cs:41-42) = `CheckCommittee(engine)`:
        // true iff the committee multi-sig address witnesses the current
        // container - the witness boundary for Treasury-signed transactions.
        let authorized = crate::committee::is_committee_witness(engine, "Treasury::verify")?;
        Ok(vec![u8::from(authorized)])
    }
}
