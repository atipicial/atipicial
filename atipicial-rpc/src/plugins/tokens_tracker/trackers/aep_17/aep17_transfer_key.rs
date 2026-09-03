//! AEP-17 transfer history key.
//!
//! Storage key for AEP-17 transfer records.

use super::super::token_transfer_key::TokenTransferKey;
use atipicial_io::{BinaryWriter, IoResult, MemoryReader, Serializable};
use atipicial_primitives::UInt160;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Key for AEP-17 transfer history records.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Aep17TransferKey(pub TokenTransferKey);

impl Aep17TransferKey {
    /// Creates a new transfer key.
    pub fn new(
        user_script_hash: UInt160,
        timestamp_ms: u64,
        asset_script_hash: UInt160,
        xfer_index: u32,
    ) -> Self {
        Self(TokenTransferKey::new(
            user_script_hash,
            timestamp_ms,
            asset_script_hash,
            xfer_index,
        ))
    }
}

impl PartialOrd for Aep17TransferKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Aep17TransferKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Serializable for Aep17TransferKey {
    fn size(&self) -> usize {
        self.0.size()
    }

    fn serialize(&self, writer: &mut BinaryWriter) -> IoResult<()> {
        Serializable::serialize(&self.0, writer)
    }

    fn deserialize(reader: &mut MemoryReader) -> IoResult<Self> {
        Ok(Self(<TokenTransferKey as Serializable>::deserialize(
            reader,
        )?))
    }
}

super::super::impl_token_transfer_key_as_ref!(Aep17TransferKey, 0);
