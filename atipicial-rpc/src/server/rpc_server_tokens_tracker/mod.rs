//! # atipicial-rpc::server::rpc_server_tokens_tracker
//!
//! Token tracker RPC endpoint handlers.
//!
//! ## Boundary
//!
//! This module belongs to `atipicial-rpc`. This API crate owns JSON-RPC surfaces and
//! transport adapters and must not implement consensus, VM semantics, or
//! storage engines.
//!
//! ## Contents
//!
//! - `balances`: AEP-11 and AEP-17 account-balance handlers.
//! - `helpers`: Shared helper functions for the surrounding module.
//! - `properties`: AEP-11 token property handler.
//! - `request`: Typed JSON-RPC request parsing helpers.
//! - `response`: Typed JSON-RPC response construction helpers.
//! - `transfers`: AEP-11 and AEP-17 transfer-history handlers.
//! - `tests`: Module-local tests and regression coverage.

use crate::server::rpc_server::RpcHandler;

mod balances;
mod helpers;
mod properties;
mod request;
mod response;
#[cfg(test)]
#[path = "../../tests/server/handlers/rpc_server_tokens_tracker.rs"]
mod tests;
mod transfers;

/// RPC handler group for AEP-11 and AEP-17 token tracker methods.
pub struct RpcServerTokensTracker;

impl RpcServerTokensTracker {
    /// Register token tracker RPC handlers.
    pub fn register_handlers() -> Vec<RpcHandler> {
        super::rpc_handlers![
            "getaep11balances" => Self::get_aep11_balances,
            "getaep11transfers" => Self::get_aep11_transfers,
            "getaep11properties" => Self::get_aep11_properties,
            "getaep17balances" => Self::get_aep17_balances,
            "getaep17transfers" => Self::get_aep17_transfers,
        ]
    }
}
