//! # atipicial-oracle-service::atipicialfs::auth
//!
//! AtipicialFs authentication and authorization helpers.
//!
//! ## Boundary
//!
//! This module belongs to `atipicial-oracle-service`. This service crate owns oracle
//! request handling and must not decide block import, consensus, or storage
//! backend policy.
//!
//! ## Contents
//!
//! - `bearer`: AtipicialFs bearer token helpers.
//! - `signing`: Witness, signer, and signature validation helpers.

#![allow(dead_code)]
#![allow(unused_imports)]

mod bearer;
mod signing;

pub(crate) use bearer::strip_bearer_prefix;
pub(crate) use signing::AtipicialFsBearerSigner;
