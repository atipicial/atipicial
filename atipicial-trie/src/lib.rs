//! # atipicial-trie
//!
//! Atipicial-compatible Merkle Patricia Trie nodes, cache logic, and state-root
//! operations.
//!
//! This module is intentionally local even though generic MPT crates exist.
//! Atipicial state roots depend on the C# `Atipicial.Cryptography.MPTTrie` node types,
//! serialization, hashing, proof shape, empty-node behavior, and cache-prefix
//! storage layout. Ethereum/Substrate trie crates are useful references, but
//! their encodings and hash domains are consensus-incompatible with Atipicial.
//!
//! ## Boundary
//!
//! This infrastructure crate exclusively owns Atipicial's MPT node graph, proof
//! shape, deterministic bytes, and backend-independent mutation cache. It uses
//! `atipicial-crypto` for Hash256 and must not own durable stores, StateService
//! policy, RPC transport, or node lifecycle.
//!
//! ## Contents
//!
//! - `mpt`: The private implementation tree for cache, nodes, validation, and
//!   trie operations.
//! - `tests`: Module-local tests and regression coverage.

#![doc(html_root_url = "https://docs.rs/atipicial-trie/0.11.1")]

mod mpt;

#[cfg(test)]
#[path = "tests/mpt_trie.rs"]
mod tests;

pub use mpt::{
    MPT_NODE_PREFIX, MptCache, MptError, MptMutationStats, MptResult, MptStoreLookup,
    MptStoreSnapshot, Node, NodeType, PersistedMptGraphLimits, PersistedMptGraphReport, Trie,
    TrieEntry, UnresolvedDeferredNode, validate_persisted_root_graph,
};
