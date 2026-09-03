//! # atipicial-io::codec
//!
//! Deterministic byte codecs and compression helpers used by Atipicial wire data.
//!
//! ## Boundary
//!
//! This module belongs to `atipicial-io`. This codec crate owns byte-level IO
//! contracts and must not decide protocol policy, storage layout, or node
//! orchestration.
//!
//! ## Contents
//!
//! - `compression`: Compression codecs and deterministic envelope helpers.

pub mod compression;
