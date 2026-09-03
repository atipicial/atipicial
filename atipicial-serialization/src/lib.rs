//! # atipicial-serialization
//!
//! Binary, JSON, and compression codecs for Atipicial data.
//!
//! ## Boundary
//!
//! This codec crate owns serialization adapters and must not run services,
//! import blocks, or mutate ledger state.
//!
//! ## Contents
//!
//! - `binary_serializer`: binary serializer implementation.
//! - `compression`: Compression codecs and deterministic envelope helpers.
//! - `json`: JSON models and codecs for external service integration.
//! - `json_serializer`: JSON serializer implementation.
//! - `serialization`: serialization codecs and compatibility checks.

#![doc(html_root_url = "https://docs.rs/atipicial-serialization/0.11.1")]

#[path = "codec/binary_serializer.rs"]
pub mod binary_serializer;
pub mod compression;
/// C#-compatible JSON token model and JSONPath support.
pub mod json;
#[path = "codec/json_serializer.rs"]
pub mod json_serializer;
#[path = "codec/serialization.rs"]
pub mod serialization;

pub use binary_serializer::BinarySerializer;
pub use compression::{Compression, CompressionAlgorithm, CompressionResult};
pub use json_serializer::JsonSerializer;
pub use serialization::{
    compress_data, decompress_data, deserialize_json, deserialize_atipicial_binary, serialize_json,
    serialize_atipicial_binary,
};
