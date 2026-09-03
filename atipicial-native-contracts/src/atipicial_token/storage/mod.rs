//! # atipicial-native-contracts::atipicial_coin::storage
//!
//! Storage contexts, key builders, and storage item helpers for execution.
//!
//! ## Boundary
//!
//! This module belongs to `atipicial-native-contracts`. This execution-domain crate
//! owns native contract logic and storage codecs and must not own node startup,
//! RPC transport, or P2P sync.
//!
//! ## Contents
//!
//! - `account`: ATC account-state codecs and balance reads.
//! - `candidates`: ATC candidate storage codecs.
//! - `committee`: committee cache readers, address derivation, and validator
//!   key helpers.
//! - `economics`: ATC economic storage records and unclaimed-GAS calculation.
//! - `keys`: ATC storage key constructors.
//! - `points`: EC-point return encoders for public committee/validator arrays.
//! - `views`: native contract storage read views.

use super::*;

mod account;
mod candidates;
mod committee;
mod economics;
mod keys;
mod points;
mod views;

pub(crate) use candidates::candidate_signature_account;
pub(crate) use views::CachedCommittee;
pub(super) use views::{AtipicialAccountStateView, CandidateState};
