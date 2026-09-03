//! Validator information record.

use atipicial_crypto::ECPoint;
use atipicial_primitives::UInt160;

/// Validator information
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    /// Validator index (0 to n-1)
    pub index: u8,
    /// Public key
    pub public_key: ECPoint,
    /// Script hash (account)
    pub script_hash: UInt160,
}
