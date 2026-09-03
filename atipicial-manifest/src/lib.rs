//! # atipicial-manifest
//!
//! Contract manifest, AEF, ABI, permission, and method-token domain types.
//!
//! ## Boundary
//!
//! This module belongs to atipicial-manifest and must respect the workspace layer
//! boundaries.
//!
//! ## Contents
//!
//! - `manifest`: Contract manifest, ABI, permission, and AEF-adjacent metadata
//!   types.
//! - `method_token`: AEF method-token records.
//! - `aef_file`: AEF file records and checksum logic.

#![doc(html_root_url = "https://docs.rs/atipicial-manifest/0.11.1")]

pub mod manifest;
#[path = "aef/method_token.rs"]
pub mod method_token;
#[path = "aef/aef_file.rs"]
pub mod aef_file;

pub use manifest::{
    ContractAbi, ContractEventDescriptor, ContractGroup, ContractManifest,
    ContractMethodDescriptor, ContractParameterDefinition, ContractPermission,
    ContractPermissionDescriptor, ManifestExtra, ManifestFeatures, WildCardContainer,
};
pub use method_token::MethodToken;
pub use aef_file::AefFile;
