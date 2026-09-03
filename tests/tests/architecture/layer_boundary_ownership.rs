//! Exclusive crate ownership and compatibility-facade invariants.

use super::*;

#[test]
fn test_execution_uses_atipicial_vm_contract_without_a_compatibility_facade() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let execution_root = workspace_root.join("atipicial-execution");

    let execution_deps = parse_atipicial_dependencies(&execution_root.join("Cargo.toml"));
    assert!(
        execution_deps
            .iter()
            .any(|dependency| dependency == "atipicial-vm"),
        "atipicial-execution must consume the workspace atipicial-vm semantic authority directly"
    );
    assert!(
        !execution_root.join("src/contracts/contract.rs").exists(),
        "atipicial-execution must not restore a Contract wrapper or re-export module over atipicial-vm"
    );
    assert!(
        !execution_root.join("src/runtime/interoperable.rs").exists(),
        "atipicial-execution must not restore an Interoperable or StackItem facade over atipicial-vm"
    );
    assert!(
        !execution_root
            .join("src/runtime/notify_event_args.rs")
            .exists(),
        "atipicial-execution must not restore a NotifyEventArgs facade over atipicial-payloads"
    );

    let checked_files = [
        "atipicial-execution/src/lib.rs",
        "atipicial-execution/src/contracts/mod.rs",
        "atipicial-execution/src/runtime/mod.rs",
    ];
    let forbidden = [
        "pub mod contract;",
        "mod contract;",
        "pub use contract::Contract",
        "pub use contracts::Contract",
        "pub use atipicial_vm",
        "pub type Contract =",
        "pub mod interoperable",
        "pub use interoperable::Interoperable",
        "SmartContractStackItem",
        "pub mod notify_event_args",
        "pub use notify_event_args::NotifyEventArgs",
        "pub use atipicial_primitives::TriggerType",
    ];
    let mut violations = Vec::new();
    for relative_path in checked_files {
        let source = fs::read_to_string(workspace_root.join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"));
        for marker in forbidden {
            if source.contains(marker) {
                violations.push(format!("{relative_path} contains `{marker}`"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "atipicial-vm::Contract must remain the only VM contract type exposed to execution callers:\n{}",
        violations.join("\n")
    );

    let vm_interoperable =
        fs::read_to_string(workspace_root.join("atipicial-vm/src/runtime/interoperable.rs"))
            .expect("read canonical atipicial-vm interoperable module");
    assert!(
        !vm_interoperable.contains("SmartContractStackItem"),
        "atipicial-vm callers must use StackItem directly instead of a duplicate type alias"
    );
}

#[test]
fn test_call_flags_have_one_primitive_owner() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let manifest_root = workspace_root.join("atipicial-manifest");

    assert!(
        !manifest_root.join("src/protocol/call_flags.rs").exists(),
        "CallFlags belongs exclusively to atipicial-primitives"
    );

    let library =
        fs::read_to_string(manifest_root.join("src/lib.rs")).expect("read atipicial-manifest crate root");
    for forbidden in [
        "pub mod call_flags",
        "pub use call_flags::CallFlags",
        "pub use atipicial_primitives::CallFlags",
    ] {
        assert!(
            !library.contains(forbidden),
            "atipicial-manifest must not restore CallFlags facade `{forbidden}`"
        );
    }

    for crate_directory in [
        "benches-package",
        "atipicial-blockchain",
        "atipicial-execution",
        "atipicial-manifest",
        "atipicial-native-contracts",
        "atipicial-node",
        "atipicial-oracle-service",
        "atipicial-payloads",
        "atipicial-rpc",
        "atipicial-vm",
    ] {
        let dependencies =
            parse_atipicial_dependencies(&workspace_root.join(crate_directory).join("Cargo.toml"));
        assert!(
            dependencies
                .iter()
                .any(|dependency| dependency == "atipicial-primitives"),
            "CallFlags consumer `{crate_directory}` must depend directly on atipicial-primitives"
        );
    }

    for relative_path in [
        "atipicial-manifest/src/aef/method_token.rs",
        "atipicial-execution/src/application_engine/mod.rs",
        "atipicial-execution/src/tests/application_engine/shadow.rs",
        "atipicial-rpc/src/server/session/execution.rs",
    ] {
        let source = fs::read_to_string(workspace_root.join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"));
        let compact = source.split_whitespace().collect::<String>();
        let grouped_imports_call_flags = |owner: &str| {
            let prefix = format!("{owner}::{{");
            compact.split(&prefix).skip(1).any(|tail| {
                tail.split_once('}').is_some_and(|(items, _)| {
                    items
                        .split(',')
                        .any(|item| item == "CallFlags" || item.starts_with("CallFlagsas"))
                })
            })
        };
        assert!(
            source.contains("atipicial_primitives::CallFlags")
                || grouped_imports_call_flags("atipicial_primitives"),
            "CallFlags consumer `{relative_path}` must import the canonical primitive owner"
        );
        assert!(
            !source.contains("atipicial_manifest::CallFlags")
                && !source.contains("atipicial_manifest::call_flags")
                && !grouped_imports_call_flags("atipicial_manifest"),
            "CallFlags consumer `{relative_path}` must not restore the removed manifest facade"
        );
    }
}

#[test]
fn test_network_does_not_reexport_foundation_primitives() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let network_root = workspace_root.join("atipicial-network");
    let library =
        fs::read_to_string(network_root.join("src/lib.rs")).expect("read atipicial-network crate root");
    let protocol = fs::read_to_string(network_root.join("src/proto/mod.rs"))
        .expect("read atipicial-network protocol module");

    let primitive_types = [
        "ContainsTransactionType",
        "InvalidWitnessScopeError",
        "InventoryType",
        "NodeCapabilityType",
        "OracleResponseCode",
        "TransactionAttributeType",
        "TransactionRemovalReason",
        "VerifyResult",
        "WitnessConditionType",
        "WitnessRuleAction",
        "WitnessScope",
    ];
    let mut violations = Vec::new();
    for primitive_type in primitive_types {
        if library.contains(primitive_type) {
            violations.push(format!(
                "atipicial-network/src/lib.rs re-exports `{primitive_type}`"
            ));
        }
        if protocol.contains(primitive_type) {
            violations.push(format!(
                "atipicial-network/src/proto/mod.rs wraps `{primitive_type}`"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "shared protocol values belong to atipicial-primitives, while atipicial-network owns only network-specific protocol and service types:\n{}",
        violations.join("\n")
    );

    for removed in ["src/proto/error.rs", "src/proto/inventory_type.rs"] {
        assert!(
            !network_root.join(removed).exists(),
            "obsolete network compatibility module must stay removed: {removed}"
        );
    }
    assert!(
        !library.contains("P2PError") && !library.contains("P2PResult"),
        "atipicial-network must expose its canonical NetworkError vocabulary only"
    );
}

#[test]
fn test_p2p_message_command_is_owned_exclusively_by_network() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let primitives_root = workspace_root.join("atipicial-primitives");
    let network_command = workspace_root.join("atipicial-network/src/proto/message_command.rs");

    for removed in [
        "src/macros/p2p_message_command.rs",
        "src/errors/network_error.rs",
        "src/tests/errors/network_error.rs",
    ] {
        assert!(
            !primitives_root.join(removed).exists(),
            "network-specific primitive must stay out of atipicial-primitives: {removed}"
        );
    }

    let mut pending = vec![primitives_root.join("src")];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        {
            let path = entry
                .expect("primitive source entry should be readable")
                .path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                continue;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            for forbidden in [
                "MessageCommand",
                "p2p_message_command",
                "network_error",
                "P2pError",
                "P2pResult",
                "pub use network_error",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "foundation source `{}` must not own P2P marker `{forbidden}`",
                    path.display()
                );
            }
        }
    }

    let source = fs::read_to_string(&network_command)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", network_command.display()));
    for required in [
        "pub MessageCommand",
        "pub struct MessageCommandParseError",
        "impl FromStr for MessageCommand",
        "impl serde::Serialize for MessageCommand",
        "impl<'de> serde::Deserialize<'de> for MessageCommand",
        "pub const fn allows_compression",
    ] {
        assert!(
            source.contains(required),
            "canonical network command owner is missing `{required}`"
        );
    }
    for forbidden in [
        "atipicial_primitives::p2p_message_command!",
        "atipicial_primitives::__p2p_message_command",
        "atipicial_primitives::NetworkError",
    ] {
        assert!(
            !source.contains(forbidden),
            "network command owner must not restore primitive facade `{forbidden}`"
        );
    }
}

#[test]
fn test_transaction_admission_is_owned_exclusively_by_mempool() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    for removed in [
        "atipicial-mempool/src/admission/transaction_router.rs",
        "atipicial-mempool/src/tests/admission/transaction_router.rs",
        "atipicial-blockchain/src/messages/fill_memory_pool.rs",
        "atipicial-blockchain/src/handlers/providers/transaction.rs",
        "atipicial-system/src/composition/tx_admission_provider.rs",
    ] {
        assert!(
            !workspace_root.join(removed).exists(),
            "obsolete transaction-admission owner must stay deleted: {removed}"
        );
    }

    let mempool_root = workspace_root.join("atipicial-mempool/src");
    let memory_pool = fs::read_to_string(mempool_root.join("pool/memory_pool.rs"))
        .expect("read canonical memory pool");
    let origin = fs::read_to_string(mempool_root.join("admission/origin.rs"))
        .expect("read transaction origin");
    let outcome = fs::read_to_string(mempool_root.join("admission/outcome.rs"))
        .expect("read transaction admission outcome");
    for required in [
        "pub fn add_transaction<B, L>",
        "validate_state_independent(",
        "ledger_provider.contains_transaction",
        "ledger_provider.contains_conflict_hash",
        "let mut guard = self.inner.write()",
        "guard.insert_validated(",
    ] {
        assert!(
            memory_pool.contains(required),
            "canonical mempool admission is missing `{required}`"
        );
    }
    assert!(origin.contains("pub enum TransactionOrigin"));
    assert!(origin.contains("External") && origin.contains("Local") && origin.contains("Private"));
    assert!(outcome.contains("pub enum TransactionAdmissionOutcome"));
    assert!(
        outcome.contains("Accepted") && outcome.contains("Rejected") && outcome.contains("Error")
    );

    let handler =
        fs::read_to_string(workspace_root.join("atipicial-blockchain/src/handlers/transactions.rs"))
            .expect("read blockchain transaction handler");
    assert!(handler.contains("TransactionAdmissionLedger::new"));
    assert!(handler.contains("self.mempool") && handler.contains(".add_transaction(origin"));
    for forbidden in [
        "contains_transaction(",
        "contains_conflict_hash(",
        "verify_state_independent(",
        "verify_state_dependent(",
        "PolicyContract::new()",
    ] {
        assert!(
            !handler.contains(forbidden),
            "blockchain service duplicated mempool policy `{forbidden}`"
        );
    }

    let ledger_context =
        fs::read_to_string(workspace_root.join("atipicial-blockchain/src/ledger/ledger_context.rs"))
            .expect("read ledger context");
    for forbidden in [
        "transactions_by_hash",
        "fn insert_transaction",
        "fn get_transaction",
    ] {
        assert!(
            !ledger_context.contains(forbidden),
            "LedgerContext must not become a second unconfirmed transaction owner: `{forbidden}`"
        );
    }

    let mut violations = Vec::new();
    for crate_name in [
        "atipicial-blockchain",
        "atipicial-system",
        "atipicial-rpc",
        "atipicial-node",
        "atipicial-oracle-service",
    ] {
        let source_root = workspace_root.join(crate_name).join("src");
        let mut pending = vec![source_root.clone()];
        while let Some(directory) = pending.pop() {
            for entry in fs::read_dir(&directory)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
            {
                let path = entry.expect("source entry should be readable").path();
                if path.is_dir() {
                    let relative = path
                        .strip_prefix(&source_root)
                        .expect("source-relative path");
                    if !relative
                        .components()
                        .any(|part| part.as_os_str() == "tests")
                    {
                        pending.push(path);
                    }
                    continue;
                }
                if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                    continue;
                }
                let source = fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
                for forbidden in [
                    "pub fn try_add(",
                    "try_add_cached",
                    "PreverifyCompleted",
                    "try_enqueue_preverify",
                    "TxRouterHandle",
                    "FillMemoryPool",
                    "TransactionRouter",
                ] {
                    if source.contains(forbidden) {
                        violations.push(format!("{} contains `{forbidden}`", path.display()));
                    }
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "only atipicial-mempool may own transaction admission:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_serialization_does_not_reexport_storage_providers() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let serialization_root = workspace_root.join("atipicial-serialization");

    let dependencies = parse_atipicial_dependencies(&serialization_root.join("Cargo.toml"));
    assert!(
        !dependencies
            .iter()
            .any(|dependency| dependency == "atipicial-storage"),
        "atipicial-serialization owns codecs only; storage providers belong exclusively to atipicial-storage"
    );
    assert!(
        !serialization_root.join("src/providers").exists(),
        "atipicial-serialization must not restore a storage-provider re-export facade"
    );

    let library = fs::read_to_string(serialization_root.join("src/lib.rs"))
        .expect("read atipicial-serialization crate root");
    for forbidden in [
        "pub mod providers",
        "MemorySnapshot",
        "MemoryStore",
        "MemoryStoreProvider",
    ] {
        assert!(
            !library.contains(forbidden),
            "atipicial-serialization crate root must not expose storage symbol `{forbidden}`"
        );
    }
}

#[test]
fn test_payloads_own_protocol_data_without_storage_or_service_facades() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let payloads_root = workspace_root.join("atipicial-payloads");

    let dependencies = parse_atipicial_dependencies(&payloads_root.join("Cargo.toml"));
    assert!(
        !dependencies
            .iter()
            .any(|dependency| dependency == "atipicial-storage"),
        "atipicial-payloads must not read storage; stateful verification belongs to domain/node services"
    );
    for removed in [
        "src/execution/event_handlers.rs",
        "src/validation/verify_result.rs",
    ] {
        assert!(
            !payloads_root.join(removed).exists(),
            "obsolete payload facade must stay removed: {removed}"
        );
    }

    let mut pending = vec![payloads_root.join("src")];
    let mut violations = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        {
            let path = entry
                .expect("payload source entry should be readable")
                .path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                continue;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            for forbidden in [
                "atipicial_storage",
                "DataCache",
                "CommittedHandler",
                "CommittingHandler",
                "FinalizedHandler",
                "WalletChangedHandler",
                "pub use atipicial_primitives::VerifyResult",
            ] {
                if source.contains(forbidden) {
                    violations.push(format!("{} contains `{forbidden}`", path.display()));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "payload code must contain only protocol data and storage-independent mechanics:\n{}",
        violations.join("\n")
    );

    let runtime_lifecycle = workspace_root.join("atipicial-runtime/src/service/lifecycle.rs");
    assert!(
        runtime_lifecycle.is_file(),
        "atipicial-runtime must remain the canonical owner of service lifecycle contracts"
    );
}

#[test]
fn test_rpc_server_does_not_depend_on_client_transport() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let rpc_root = workspace_root.join("atipicial-rpc");
    let manifest = read_toml_manifest(&rpc_root.join("Cargo.toml"));
    let server_features = manifest
        .get("features")
        .and_then(|features| features.get("server"))
        .and_then(toml::Value::as_array)
        .expect("atipicial-rpc should declare a server feature");
    assert!(
        !server_features
            .iter()
            .any(|feature| feature.as_str() == Some("client")),
        "atipicial-rpc server must not enable the outbound client transport"
    );

    let mut pending = vec![rpc_root.join("src/server")];
    let mut violations = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        {
            let path = entry.expect("RPC source entry should be readable").path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                continue;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            for forbidden in ["crate::client", "atipicial_rpc::client"] {
                if source.contains(forbidden) {
                    violations.push(format!("{} contains `{forbidden}`", path.display()));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "RPC server code must consume transport-neutral types and codecs:\n{}",
        violations.join("\n")
    );

    for removed in [
        "src/errors/error.rs",
        "src/client/models/contracts/rpc_contract_state.rs",
        "src/client/models/contracts/rpc_method_token.rs",
        "src/client/models/contracts/rpc_aef_file.rs",
        "src/client/models/ledger/rpc_raw_mem_pool.rs",
        "src/client/models/network/rpc_get_peers.rs",
        "src/client/models/network/rpc_peers.rs",
    ] {
        assert!(
            !rpc_root.join(removed).exists(),
            "obsolete RPC ownership path must stay removed: {removed}"
        );
    }
    for required in [
        "src/client/errors/client.rs",
        "src/protocol/address.rs",
        "src/types/contract_state.rs",
        "src/types/method_token.rs",
        "src/types/aef_file.rs",
        "src/types/peers.rs",
        "src/types/raw_mempool.rs",
    ] {
        assert!(
            rpc_root.join(required).is_file(),
            "canonical RPC ownership path is missing: {required}"
        );
    }

    let crate_root = fs::read_to_string(rpc_root.join("src/lib.rs"))
        .expect("atipicial-rpc crate root should be readable");
    assert!(
        !crate_root.contains("RpcClientError"),
        "RpcClientError belongs under atipicial_rpc::client, not a crate-root compatibility facade"
    );
}

#[test]
fn test_storage_backend_selection_is_closed_and_statically_dispatched() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let storage_root = workspace_root.join("atipicial-storage");

    assert!(
        !storage_root
            .join("src/persistence/traits/store_provider.rs")
            .exists(),
        "atipicial-storage must not restore the unused StoreProvider extension trait"
    );

    let factory = fs::read_to_string(storage_root.join("src/persistence/traits/store_factory.rs"))
        .expect("read atipicial-storage store factory");
    for required in [
        "enum StoreBackend",
        "Self::Memory => MemoryStoreProvider::new()",
        "Self::Mdbx => MdbxStoreProvider::new",
    ] {
        assert!(
            factory.contains(required),
            "closed storage backend selection is missing `{required}`"
        );
    }
    for forbidden in [
        "pub enum StoreBackend",
        "pub trait StoreProvider",
        "P: StoreProvider",
    ] {
        assert!(
            !factory.contains(forbidden),
            "storage factory must not restore open provider facade `{forbidden}`"
        );
    }
}

#[test]
fn test_composition_surfaces_retain_one_chain_spec() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    for crate_name in ["atipicial-network", "atipicial-system", "atipicial-rpc"] {
        let dependencies =
            parse_atipicial_dependencies(&workspace_root.join(crate_name).join("Cargo.toml"));
        assert!(
            dependencies
                .iter()
                .any(|dependency| dependency == "atipicial-config"),
            "{crate_name} must obtain AtipicialChainSpec and ChainSpecProvider from atipicial-config"
        );
    }

    let checked_surfaces = [
        (
            "atipicial-network/src/service/local_node.rs",
            &["chain_spec: Arc<AtipicialChainSpec>"][..],
        ),
        (
            "atipicial-system/src/composition/node.rs",
            &[
                "chain_spec: Arc<AtipicialChainSpec>",
                "impl<P, S> ChainSpecProvider for Node<P, S>",
            ][..],
        ),
        (
            "atipicial-system/src/composition/builder.rs",
            &["chain_spec: Arc<AtipicialChainSpec>"][..],
        ),
        (
            "atipicial-system/src/composition/core.rs",
            &["chain_spec: Arc<AtipicialChainSpec>"][..],
        ),
        (
            "atipicial-system/src/composition/system_context.rs",
            &["chain_spec: Arc<AtipicialChainSpec>"][..],
        ),
        (
            "atipicial-rpc/src/server/node_context.rs",
            &[
                "chain_spec: Arc<AtipicialChainSpec>",
                "impl<P, S> ChainSpecProvider for NodeContext<P, S>",
            ][..],
        ),
    ];
    let duplicate_settings_fields = [
        "\n    settings: Arc<ProtocolSettings>",
        "\n    protocol_settings: Arc<ProtocolSettings>",
        "\n    settings: ProtocolSettings",
        "\n    protocol_settings: ProtocolSettings",
    ];
    let mut violations = Vec::new();

    for (relative_path, required) in checked_surfaces {
        let source = fs::read_to_string(workspace_root.join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"));
        for marker in required {
            if !source.contains(marker) {
                violations.push(format!("{relative_path} is missing `{marker}`"));
            }
        }
        for marker in duplicate_settings_fields {
            if source.contains(marker) {
                violations.push(format!(
                    "{relative_path} contains duplicate field `{marker}`"
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "node composition must retain one immutable AtipicialChainSpec and derive protocol settings from it:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_genesis_builder_uses_the_authoritative_chain_spec() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let genesis_path = workspace_root.join("atipicial-blockchain/src/pipeline/native_persist/genesis.rs");
    let source = fs::read_to_string(&genesis_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", genesis_path.display()));

    for required in [
        "pub fn genesis_block(chain_spec: &AtipicialChainSpec)",
        "let genesis = chain_spec.genesis();",
        "header.set_timestamp(genesis.timestamp);",
        "header.set_nonce(genesis.nonce);",
        "let settings = chain_spec.protocol_settings();",
    ] {
        assert!(
            source.contains(required),
            "canonical genesis construction is missing `{required}`"
        );
    }
    for forbidden in [
        "pub fn genesis_block(settings: &ProtocolSettings)",
        "header.set_timestamp(GENESIS_TIMESTAMP_MS)",
        "header.set_nonce(GENESIS_NONCE)",
    ] {
        assert!(
            !source.contains(forbidden),
            "canonical genesis construction must not restore `{forbidden}`"
        );
    }
}
