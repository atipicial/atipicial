//! # atipicial-oracle-service::atipicialfs::json
//!
//! JSON models and codecs for external service integration.
//!
//! ## Boundary
//!
//! This module belongs to `atipicial-oracle-service`. This service crate owns oracle
//! request handling and must not decide block import, consensus, or storage
//! backend policy.
//!
//! ## Contents
//!
//! - `headers`: HTTP header helpers for AtipicialFs JSON requests.
//! - `helpers`: Shared helper functions for the surrounding module.
//! - `object`: AtipicialFs JSON object models.
//! - `session`: AtipicialFs JSON session token models.

// Rationale: AtipicialFs JSON models mirror generated protobuf shapes; some fields
// are only exercised by optional AtipicialFs request/response variants.
#![allow(dead_code)]

mod headers;
mod helpers;
mod object;
mod session;

pub(crate) use headers::build_atipicialfs_header_payload;
pub(crate) use helpers::normalize_atipicialfs_hex_header;
#[cfg(feature = "atipicialfs-grpc")]
// Rationale: object JSON helpers are re-exported for the optional gRPC bridge
// even when a given build does not call every helper.
#[allow(unused)]
pub(crate) use object::{
    build_atipicialfs_object_payload, atipicialfs_json_header, atipicialfs_json_object_id, atipicialfs_json_version,
};
#[cfg(feature = "atipicialfs-grpc")]
// Rationale: session-token JSON projection is part of the optional AtipicialFs gRPC
// surface and may be unused in HTTP-only builds.
#[allow(unused)]
pub(crate) use session::atipicialfs_json_session_token;
