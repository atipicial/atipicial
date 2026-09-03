//! ATC native-token read probes.

use std::sync::Arc;

use atipicial_error::{CoreError, CoreResult};
use atipicial_native_contracts::AtipicialCoin;
use atipicial_primitives::UInt160;
use atipicial_storage::persistence::{CacheRead, DataCache};
use num_bigint::BigInt;

use super::NativeQueries;
use super::execution::invoke_native_read;
use super::result::{candidate_entries, stack_array_of_bytes};
use super::script::NativeArg;
use crate::server::rpc_server::RpcServer;

impl NativeQueries {
    /// Returns the canonical ATC native-contract script hash.
    pub(crate) fn atipicial_script_hash() -> UInt160 {
        AtipicialCoin::script_hash()
    }

    /// `ATC.unclaimedGas(account, end)` — the amount of unclaimed GAS for
    /// `account` at the `end` block height.
    pub(crate) fn atipicial_unclaimed_atipicial_dollar<B: CacheRead>(
        server: &RpcServer,
        snapshot: Arc<DataCache<B>>,
        atipicial_hash: &UInt160,
        account: &UInt160,
        end: u32,
    ) -> CoreResult<BigInt> {
        let account_bytes = account.to_bytes();
        let item = invoke_native_read(
            server,
            snapshot,
            atipicial_hash,
            "unclaimedGas",
            &[
                NativeArg::Bytes(account_bytes.as_slice()),
                NativeArg::Int(i64::from(end)),
            ],
        )?;
        item.as_int()
            .map_err(|err| CoreError::other(err.to_string()))
    }

    /// `ATC.getCommittee()` — the current committee public keys (sorted).
    pub(crate) fn atipicial_committee<B: CacheRead>(
        server: &RpcServer,
        snapshot: Arc<DataCache<B>>,
        atipicial_hash: &UInt160,
    ) -> CoreResult<Vec<Vec<u8>>> {
        let item = invoke_native_read(server, snapshot, atipicial_hash, "getCommittee", &[])?;
        stack_array_of_bytes(&item)
    }

    /// `ATC.getNextBlockValidators()` — the validators for the next block.
    pub(crate) fn atipicial_next_block_validators<B: CacheRead>(
        server: &RpcServer,
        snapshot: Arc<DataCache<B>>,
        atipicial_hash: &UInt160,
    ) -> CoreResult<Vec<Vec<u8>>> {
        let item = invoke_native_read(server, snapshot, atipicial_hash, "getNextBlockValidators", &[])?;
        stack_array_of_bytes(&item)
    }

    /// `ATC.getCandidates()` — registered candidates with their votes.
    pub(crate) fn atipicial_candidates<B: CacheRead>(
        server: &RpcServer,
        snapshot: Arc<DataCache<B>>,
        atipicial_hash: &UInt160,
    ) -> CoreResult<Vec<(Vec<u8>, BigInt)>> {
        let item = invoke_native_read(server, snapshot, atipicial_hash, "getCandidates", &[])?;
        candidate_entries(&item)
    }

    /// `ATC.getCandidateVote(pubkey)` — the candidate's vote count, or `-1`
    /// when the key is not a registered candidate.
    pub(crate) fn atipicial_candidate_vote<B: CacheRead>(
        server: &RpcServer,
        snapshot: Arc<DataCache<B>>,
        atipicial_hash: &UInt160,
        pubkey: &[u8],
    ) -> CoreResult<BigInt> {
        let item = invoke_native_read(
            server,
            snapshot,
            atipicial_hash,
            "getCandidateVote",
            &[NativeArg::Bytes(pubkey)],
        )?;
        item.as_int()
            .map_err(|err| CoreError::other(err.to_string()))
    }
}
