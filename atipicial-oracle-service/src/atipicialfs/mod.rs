//! # atipicial-oracle-service::atipicialfs
//!
//! AtipicialFs request signing, authentication, JSON, and response helpers.
//!
//! ## Boundary
//!
//! This module belongs to `atipicial-oracle-service`. This service crate owns oracle
//! request handling and must not decide block import, consensus, or storage
//! backend policy.
//!
//! ## Contents
//!
//! - `auth`: AtipicialFs authentication and authorization helpers.
//! - `grpc`: AtipicialFs gRPC client helpers.
//! - `http`: AtipicialFs HTTP client helpers.
//! - `json`: JSON models and codecs for external service integration.
//! - `parse`: AtipicialFs response parsing helpers.
//! - `proto`: Protocol message definitions and network payload framing.
//! - `tests`: Module-local tests and regression coverage.

mod auth;
#[cfg(feature = "atipicialfs-grpc")]
mod grpc;
mod http;
mod json;
mod parse;
#[cfg(feature = "atipicialfs-grpc")]
mod proto;

#[cfg(test)]
#[path = "../tests/atipicialfs/mod.rs"]
mod tests;

use super::OracleServiceSettings;
use crate::service::OracleServiceError;
use http::normalize_atipicialfs_endpoint;
use atipicial_payloads::OracleResponseCode;
use atipicial_wallets::KeyPair;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AtipicialFsRange {
    offset: u64,
    length: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AtipicialFsCommand {
    Payload,
    Range(AtipicialFsRange),
    Header,
    Hash(Option<AtipicialFsRange>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AtipicialFsRequest {
    container: String,
    object: String,
    command: AtipicialFsCommand,
}

#[derive(Clone, Debug, Default)]
struct AtipicialFsAuth {
    token: Option<String>,
    signature: Option<String>,
    signature_key: Option<String>,
    wallet_connect: bool,
}

fn decode_raw_base58(value: &str, expected_len: Option<usize>) -> Option<Vec<u8>> {
    let decoded = atipicial_crypto::base58::decode(value).ok()?;
    if expected_len.is_some_and(|len| decoded.len() != len) {
        return None;
    }
    Some(decoded)
}

pub(crate) struct OracleAtipicialFsProtocol {
    client: reqwest::Client,
}

impl OracleAtipicialFsProtocol {
    pub(crate) fn new() -> Result<Self, OracleServiceError> {
        let version = env!("CARGO_PKG_VERSION");
        Self::from_builder(
            reqwest::Client::builder().user_agent(format!("AtipicialOracleService/{}", version)),
        )
    }

    fn from_builder(builder: reqwest::ClientBuilder) -> Result<Self, OracleServiceError> {
        let client = builder
            .build()
            .map_err(|err| OracleServiceError::HttpClientInitialization(err.to_string()))?;
        Ok(Self { client })
    }

    pub(crate) async fn process(
        &self,
        settings: &OracleServiceSettings,
        url: &str,
        oracle_key: Option<&KeyPair>,
    ) -> (OracleResponseCode, String) {
        let request = match AtipicialFsRequest::parse_atipicialfs_request(url) {
            Ok(request) => request,
            Err(_) => return (OracleResponseCode::Error, String::new()),
        };
        let endpoint = match normalize_atipicialfs_endpoint(&settings.atipicialfs_endpoint) {
            Ok(endpoint) => endpoint,
            Err(_) => return (OracleResponseCode::Error, String::new()),
        };
        let auth = AtipicialFsAuth::build_atipicialfs_auth(settings, oracle_key);

        if settings.atipicialfs_use_grpc {
            #[cfg(feature = "atipicialfs-grpc")]
            {
                let Some(key) = oracle_key else {
                    return (OracleResponseCode::Error, String::new());
                };
                let fut = self.execute_grpc_request(&endpoint, request, &auth, key);
                return match tokio::time::timeout(settings.atipicialfs_timeout, fut).await {
                    Ok(result) => result,
                    Err(_) => (OracleResponseCode::Error, String::new()),
                };
            }
            #[cfg(not(feature = "atipicialfs-grpc"))]
            {
                return (OracleResponseCode::Error, String::new());
            }
        }

        let fut = self.execute_request(&endpoint, request, &auth);
        match tokio::time::timeout(settings.atipicialfs_timeout, fut).await {
            Ok(result) => result,
            Err(_) => (OracleResponseCode::Timeout, String::new()),
        }
    }
}
