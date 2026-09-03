//! # atipicial-oracle-service::tests::atipicialfs
//!
//! Test module grouping AtipicialFs request signing, authentication, JSON, and
//! response helpers. coverage for atipicial-oracle-service.
//!
//! ## Boundary
//!
//! This is test/benchmark-only code for atipicial-oracle-service; it may assemble
//! fixtures but must not introduce production behavior.
//!
//! ## Contents
//!
//! - `auth`: AtipicialFs authentication and authorization helpers.
//! - `http`: AtipicialFs HTTP client helpers.
//! - `json`: JSON models and codecs for external service integration.
//! - `parse`: AtipicialFs response parsing helpers.

#[path = "auth.rs"]
mod auth;
#[path = "http.rs"]
mod http;
#[cfg(feature = "atipicialfs-grpc")]
#[path = "json.rs"]
mod json;
#[path = "parse.rs"]
mod parse;
