//! # atipicial-oracle-service::atipicialfs::json::headers
//!
//! HTTP header helpers for AtipicialFs JSON requests.
//!
//! ## Boundary
//!
//! This module belongs to `atipicial-oracle-service`. This service crate owns oracle
//! request handling and must not decide block import, consensus, or storage
//! backend policy.
//!
//! ## Contents
//!
//! - `attributes`: HTTP header attribute records.
//! - `payload`: Payload-domain primitives shared by protocol and network
//!   crates.

mod attributes;
mod payload;

pub(crate) use payload::build_atipicialfs_header_payload;
