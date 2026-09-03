use std::sync::Arc;

use atipicial_wallets::wallet_helper::WalletAddress as address_helper;
use serde_json::Value;

use crate::server::ledger_queries;
use crate::server::native_queries;

use crate::server::rpc_exception::RpcException;
use crate::server::rpc_server::RpcServer;

use super::helpers::internal_error;
use super::request::GetUnclaimedGasRequest;
use super::response::unclaimed_atipicial_dollar_to_json;

pub(super) fn get_unclaimed_gas(
    server: &RpcServer,
    params: &[Value],
) -> Result<Value, RpcException> {
    let version = server.system().settings().address_version;
    let request = GetUnclaimedGasRequest::parse(params, version)?;

    let store = server.system().store_cache();
    let height = ledger_queries::current_index(store.data_cache())
        .map_err(|err| internal_error(err.to_string()))?
        .saturating_add(1);
    let atipicial_hash = native_queries::NativeQueries::atipicial_script_hash();
    let snapshot = Arc::new(store.data_cache().clone());
    let unclaimed = native_queries::NativeQueries::atipicial_unclaimed_atipicial_dollar(
        server,
        snapshot,
        &atipicial_hash,
        &request.script_hash,
        height,
    )
    .map_err(internal_error)?;
    let address = address_helper::to_address(&request.script_hash, version);

    Ok(unclaimed_atipicial_dollar_to_json(address, unclaimed))
}
