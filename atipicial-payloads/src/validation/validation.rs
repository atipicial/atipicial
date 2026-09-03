//! Block validation constants re-exported for the payload layer.
//!
//! Stateful block-validation helpers live in `atipicial-blockchain`, which can read
//! the canonical snapshot and native-contract state. The constants below are
//! pure values used for structural payload validation.

/// Minimum valid timestamp (Atipicial genesis block timestamp: July 15, 2026)
pub const MIN_TIMESTAMP_MS: u64 = 1784505600000;

/// Maximum allowed timestamp drift from current time (15 minutes in milliseconds)
pub const MAX_TIMESTAMP_DRIFT_MS: u64 = 15 * 60 * 1000;
