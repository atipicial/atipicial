//! # atipicial-static-files tests
//!
//! ## Boundary
//!
//! This harness validates the protocol-blind archive provider and its durable
//! index. Higher-level Atipicial Ledger reconciliation remains in `atipicial-blockchain`.
//!
//! ## Contents
//!
//! - `archive`: Append, lookup, ownership, recovery, and MDBX index behavior.

mod archive;
