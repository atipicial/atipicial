//! # atipicial-native-contracts::tests::atipicial_coin
//!
//! Test module grouping Native ATC token governance, voting, and committee
//! behavior. coverage for atipicial-native-contracts.
//!
//! ## Boundary
//!
//! This is test/benchmark-only code for atipicial-native-contracts; it may assemble
//! fixtures but must not introduce production behavior.
//!
//! ## Contents
//!
//! - `basic_tests`: basic token behavior coverage.
//! - `committee_recompute_tests`: committee recomputation coverage.
//! - `governance_writer_tests`: governance writer coverage.
//! - `persist_hook_tests`: persist-hook coverage.
//! - `storage_codec_tests`: storage codec coverage.
//! - `witness_harness_tests`: witness harness coverage.

use super::*;
use atipicial_primitives::{CallFlags, ContractParameterType};
use atipicial_serialization::BinarySerializer;
use atipicial_vm::ExecutionEngineLimits;

#[cfg(test)]
#[path = "basic_tests.rs"]
mod basic_tests;
#[cfg(test)]
#[path = "committee_recompute_tests.rs"]
mod committee_recompute_tests;
#[cfg(test)]
#[path = "governance_writer_tests/mod.rs"]
mod governance_writer_tests;
#[cfg(test)]
#[path = "persist_hook_tests.rs"]
mod persist_hook_tests;
#[cfg(test)]
#[path = "storage_codec_tests.rs"]
mod storage_codec_tests;
#[cfg(test)]
#[path = "witness_harness_tests.rs"]
mod witness_harness_tests;
