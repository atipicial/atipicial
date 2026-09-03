//! Standard native-contract registry construction for RPC probes.
//!
//! `NativeRegistry::new()` is empty by design. RPC query tests and handlers
//! need the protocol-native contract set registered explicitly before resolving
//! native hashes, ids, and manifests.

use super::NativeQueries;

impl NativeQueries {
    /// Builds a [`atipicial_execution::NativeRegistry`] populated with the standard
    /// native contracts.
    pub(crate) fn native_registry()
    -> atipicial_execution::NativeRegistry<atipicial_native_contracts::StandardNativeProvider> {
        let mut registry =
            atipicial_execution::NativeRegistry::<atipicial_native_contracts::StandardNativeProvider>::new();
        for contract in atipicial_native_contracts::standard_native_contracts() {
            registry.register(contract);
        }
        registry
    }
}
