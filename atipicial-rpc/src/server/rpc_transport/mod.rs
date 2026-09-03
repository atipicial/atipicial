//! # atipicial-rpc::server::rpc_transport
//!
//! RPC transport startup, binding, and shutdown helpers.
//!
//! ## Boundary
//!
//! This module belongs to `atipicial-rpc`. This API crate owns JSON-RPC surfaces and
//! transport adapters and must not implement consensus, VM semantics, or
//! storage engines.
//!
//! ## Contents
//!
//! - `rpc_transport`: RPC listener binding, transport startup, and shutdown.

use tracing::{error, warn};

pub(crate) fn log_join_error(error: tokio::task::JoinError) {
    if error.is_cancelled() {
        warn!(target: "atipicial", "rpc server task cancelled before completion");
    } else {
        match error.try_into_panic() {
            Ok(payload) => {
                if let Some(message) = payload.downcast_ref::<&str>() {
                    error!(target: "atipicial", message = %message, "rpc server panicked");
                } else if let Some(message) = payload.downcast_ref::<String>() {
                    error!(target: "atipicial", message = %message, "rpc server panicked");
                } else {
                    error!(target: "atipicial", "rpc server panicked");
                }
            }
            Err(join_err) => {
                error!(target: "atipicial", error = %join_err, "rpc server task failed");
            }
        }
    }
}
