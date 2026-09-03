//! Composition, node-service, and durable-storage boundary invariants.

use super::*;

#[test]
fn test_composition_node_hides_component_layout() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let node_path = workspace_root
        .join("atipicial-system")
        .join("src")
        .join("composition")
        .join("node.rs");
    let source = fs::read_to_string(&node_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", node_path.display()));

    let public_fields = [
        "pub chain_spec:",
        "pub storage:",
        "pub wallets:",
        "pub blockchain:",
        "pub network:",
        "pub staged_sync_pipeline:",
        "pub live_block_import_pipeline:",
        "pub mempool:",
        "pub header_cache:",
        "pub native_contract_provider:",
        "pub shutdown:",
    ];
    let exposed = public_fields
        .into_iter()
        .filter(|field| source.contains(field))
        .collect::<Vec<_>>();
    assert!(
        exposed.is_empty(),
        "atipicial-system::Node is an application-facing facade; component layout must stay private: {exposed:?}"
    );

    for accessor in [
        "pub fn chain_spec(&self)",
        "pub fn storage(&self)",
        "pub fn blockchain(&self)",
        "pub fn network(&self)",
        "pub fn staged_sync_pipeline(",
        "pub fn live_block_import_pipeline(&self)",
        "pub fn mempool(&self)",
        "pub fn header_cache(&self)",
        "pub fn native_contract_provider(&self)",
    ] {
        assert!(
            source.contains(accessor),
            "atipicial-system::Node should expose named capability accessor `{accessor}`"
        );
    }

    assert!(
        !source.contains("cancellation_token(") && !source.contains("CancellationToken"),
        "process cancellation belongs to the application supervisor, not atipicial-system::Node"
    );
}

#[test]
fn test_rpc_node_context_hides_component_layout() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let context_path = workspace_root
        .join("atipicial-rpc")
        .join("src")
        .join("server")
        .join("node_context.rs");
    let source = fs::read_to_string(&context_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", context_path.display()));

    let public_fields = [
        "pub chain_spec:",
        "pub storage:",
        "pub blockchain:",
        "pub network:",
        "pub mempool:",
        "pub header_cache:",
        "pub services:",
        "pub native_contract_provider:",
    ];
    let exposed = public_fields
        .into_iter()
        .filter(|field| source.contains(field))
        .collect::<Vec<_>>();
    assert!(
        exposed.is_empty(),
        "RPC callers must use named NodeContext capabilities, not its component layout: {exposed:?}"
    );
}

#[test]
fn test_node_services_hide_command_loop_and_wire_module_layouts() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let blockchain_root = workspace_root.join("atipicial-blockchain").join("src");
    let blockchain_lib =
        fs::read_to_string(blockchain_root.join("lib.rs")).expect("read atipicial-blockchain crate root");
    let blockchain_service = fs::read_to_string(blockchain_root.join("service/mod.rs"))
        .expect("read atipicial-blockchain service module");

    assert!(
        blockchain_lib.contains("mod service;"),
        "atipicial-blockchain should expose handles and capabilities, not its service module tree"
    );
    for forbidden in [
        "pub mod service;",
        "pub use internal::{",
        "pub mod internal;",
    ] {
        assert!(
            !blockchain_lib.contains(forbidden) && !blockchain_service.contains(forbidden),
            "atipicial-blockchain command-loop internal `{forbidden}` must stay private"
        );
    }

    let network_lib_path = workspace_root.join("atipicial-network").join("src/lib.rs");
    let network_lib = fs::read_to_string(&network_lib_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", network_lib_path.display()));
    for forbidden in [
        "pub mod proto;",
        "pub mod wire;",
        "command, event, handle, local_node, remote_node,",
    ] {
        assert!(
            !network_lib.contains(forbidden),
            "atipicial-network should export typed capabilities, not module layout `{forbidden}`"
        );
    }

    let network_service_root = workspace_root.join("atipicial-network/src/service");
    for removed in ["block_sync_mode.rs", "task_manager.rs"] {
        assert!(
            !network_service_root.join(removed).exists(),
            "coordinator-owned range sync must not regain legacy service module `{removed}`"
        );
    }
    for removed_export in [
        "BlockRequestScheduler",
        "BlockSyncMode",
        "TaskManagerService",
        "TaskManagerHandle",
    ] {
        assert!(
            !network_lib.contains(removed_export),
            "atipicial-network must keep one coordinator-owned range-sync path; found `{removed_export}`"
        );
    }
}

#[test]
fn test_application_context_contains_only_commit_hooks() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let context_dir = workspace_root
        .join("atipicial-node")
        .join("src")
        .join("node")
        .join("context");
    let mut violations = Vec::new();

    for entry in fs::read_dir(&context_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", context_dir.display()))
    {
        let path = entry.expect("context directory entry").path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for forbidden in [
            "atipicial_system::Node",
            "impl SystemContext",
            "StoreCache<",
            "\n    native_contract_provider:",
        ] {
            if source.contains(forbidden) {
                violations.push(format!("{} contains `{forbidden}`", path.display()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "application commit hooks must not reclaim composition-owned core context responsibilities:\n{}",
        violations.join("\n")
    );
}

#[tokio::test]
async fn test_indexer_remains_reusable_node_service_not_rpc_submodule() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    assert_eq!(
        Layer::from_crate_name("atipicial-indexer"),
        Some(Layer::NodeServices),
        "atipicial-indexer should stay classified as a node service, not as an RPC plugin"
    );

    let indexer_deps =
        parse_atipicial_dependencies(&workspace_root.join("atipicial-indexer").join("Cargo.toml"));
    let forbidden_owners = ["atipicial-rpc", "atipicial-system", "atipicial-node"];
    for forbidden_owner in forbidden_owners {
        assert!(
            !indexer_deps.iter().any(|dep| dep == forbidden_owner),
            "atipicial-indexer must not depend on {forbidden_owner}; it should remain reusable by RPC, node, and future service surfaces"
        );
    }

    let same_or_higher_layer_deps = indexer_deps
        .iter()
        .filter(|dep| {
            matches!(
                Layer::from_crate_name(dep),
                Some(layer) if layer >= Layer::NodeServices
            )
        })
        .collect::<Vec<_>>();
    assert!(
        same_or_higher_layer_deps.is_empty(),
        "atipicial-indexer should index persisted protocol data and depend only on lower layers, but found: {:?}",
        same_or_higher_layer_deps
    );

    let rpc_deps = parse_atipicial_dependencies(&workspace_root.join("atipicial-rpc").join("Cargo.toml"));
    assert!(
        rpc_deps.iter().any(|dep| dep == "atipicial-indexer"),
        "atipicial-rpc should expose AtipicialIndexer methods by consuming the node-service crate"
    );

    let node_deps = parse_atipicial_dependencies(&workspace_root.join("atipicial-node").join("Cargo.toml"));
    assert!(
        node_deps.iter().any(|dep| dep == "atipicial-indexer"),
        "atipicial-node should own the live indexer lifecycle and register it for RPC"
    );
}

#[tokio::test]
async fn test_active_architecture_docs_do_not_reference_retired_crates() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let checked_files = [
        "Cargo.toml",
        "README.md",
        "docs/architecture.md",
        "docs/protocol-compatibility.md",
        "docs/STYLE.md",
        "atipicial-runtime/Cargo.toml",
        "atipicial-runtime/src/lib.rs",
        "atipicial-rpc/Cargo.toml",
        "atipicial-rpc/src/client/contracts/contract_client.rs",
        "atipicial-rpc/src/server/rpc_server_blockchain/mod.rs",
        "atipicial-rpc/src/tests/server/handlers/rpc_server_node.rs",
        "atipicial-system/Cargo.toml",
        "atipicial-system/src/lib.rs",
        "atipicial-network/Cargo.toml",
        "atipicial-network/src/lib.rs",
        "atipicial-network/src/wire/mod.rs",
        "atipicial-network/src/proto/mod.rs",
        "atipicial-network/src/service/mod.rs",
    ];

    let retired_terms = [
        "atipicial-core",
        "atipicial-p2p",
        "Layer 1 (runtime)",
        "Layer 1 (service)",
        "Layer 1 (protocol types)",
        "Layer 2 (services we talk to)",
        "Stage A",
        "Stage B/C",
        "Actor runtime",
    ];
    let mut violations = Vec::new();

    for relative_path in checked_files {
        let path = workspace_root.join(relative_path);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

        for term in retired_terms {
            if content.contains(term) {
                violations.push(format!("{relative_path} still references `{term}`"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Active architecture documentation should use the current crate/layer vocabulary:\n{}",
        violations.join("\n")
    );
}

#[tokio::test]
async fn test_protocol_compatibility_doc_pins_v3101_release_audit() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let doc_path = workspace_root
        .join("docs")
        .join("protocol-compatibility.md");
    let doc = fs::read_to_string(&doc_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", doc_path.display()));

    let required_markers = [
        "https://github.com/atipicial-project/atipicial/releases/tag/v3.10.1",
        "d10e9ceecdabe3fcff719ee68ea5b76ba7e62c3d",
        "v3.10.0...v3.10.1",
        "df402675",
        "e66e4dfc",
        "6b1c90c6",
        "f5ae5e82",
        "55c14029",
        "7f8454f4",
        "9f4795ab",
        "abbc3a25",
        "7bb91ff5",
        "d10e9cee",
        "HF_Huyao",
        "ExtensiblePayload",
        "StorageKey.ToString()",
    ];

    let missing = required_markers
        .iter()
        .filter(|marker| !doc.contains(**marker))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "protocol compatibility doc must pin the full Atipicial v3.10.1 release audit; missing: {missing:?}"
    );
}

#[tokio::test]
async fn test_atipicial_v3101_release_delta_has_source_guards() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let checked_markers = [
        (
            "atipicial-primitives/src/protocol/chain/hardfork.rs",
            &["HfHuyao = 7 => \"HF_Huyao\""][..],
        ),
        (
            "atipicial-config/src/settings/hardfork.rs",
            &[
                "with_omitted_leading_at_genesis",
                "for hardfork in Hardfork::ALL",
            ][..],
        ),
        (
            "atipicial-execution/src/application_engine/fees_events_native.rs",
            &[
                "C# v3.10.1 validates `AddFee` arguments before applying the",
                "add_native_method_fee",
                "checked_add(storage_pico)",
            ][..],
        ),
        (
            "atipicial-native-contracts/src/atipicial_coin/persist.rs",
            &[
                "Hardfork::HfGorgon",
                "self.candidate_vote(&snapshot, member)?",
            ][..],
        ),
        (
            "atipicial-native-contracts/src/notary/persist.rs",
            &["Deposit not found", "Insufficient deposit"][..],
        ),
        (
            "atipicial-payloads/src/protocol/extensible_payload.rs",
            &["Witness script hash {witness_hash} does not match sender"][..],
        ),
        (
            "atipicial-storage/src/types/storage_key.rs",
            &["StorageKey{{Id={}}}", "StorageKey{{Id={},Key={}}}"][..],
        ),
        (
            "atipicial-native-contracts/src/std_lib/numeric.rs",
            &["CultureInfo.InvariantCulture", "dotnet_bigint_to_hex"][..],
        ),
        (
            "atipicial-vm/src/runtime/reference_counter.rs",
            &[
                "C# `RemoveStackReference(item)`",
                "compounds lower the total only when `IsStackReferenced` is true",
                "self.remove_stack_reference(&sub_item)",
            ][..],
        ),
        (
            "atipicial-vm/src/stack_item/array.rs",
            &[
                "C# v3.10.1 CLEARITEMS snapshots sub-items",
                "inner.items.clear();",
                "rc.remove_stack_reference(&item)",
            ][..],
        ),
        (
            "atipicial-vm/src/stack_item/map.rs",
            &[
                "C# v3.10.1 CLEARITEMS snapshots `Map.SubItems`",
                "inner.items.clear();",
                "rc.remove_stack_reference(&item)",
            ][..],
        ),
        (
            "atipicial-vm/src/tests/runtime/reference_counter.rs",
            &[
                "removing_unreferenced_compound_does_not_underflow",
                "clearing_self_referenced_array_releases_all_references",
                "clearing_self_referenced_map_releases_all_references",
            ][..],
        ),
    ];

    let mut missing = Vec::new();
    for (relative_path, markers) in checked_markers {
        let path = workspace_root.join(relative_path);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for marker in markers {
            if !source.contains(marker) {
                missing.push(format!("{relative_path} missing `{marker}`"));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "Atipicial v3.10.1 release-delta source guards are missing:\n{}",
        missing.join("\n")
    );
}

#[test]
fn test_operational_metadata_uses_typed_tables_and_commits_remain_fallible() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let typed_consumers = [
        "atipicial-runtime/src/service/sync_pipeline/checkpoint_store.rs",
        "atipicial-runtime/src/service/sync_pipeline/verified_header_store.rs",
        "atipicial-blockchain/src/ledger/static_archive/pruning.rs",
    ];
    let mut violations = Vec::new();

    for relative_path in typed_consumers {
        let path = workspace_root.join(relative_path);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        for forbidden in [
            ".maintenance_metadata(",
            ".put_metadata(",
            ".delete_metadata(",
        ] {
            if source.contains(forbidden) {
                violations.push(format!("{relative_path} contains `{forbidden}`"));
            }
        }
        if !source.contains("table_get::<") || !source.contains(".put::<") {
            violations.push(format!(
                "{relative_path} must read and write through typed table identities"
            ));
        }
    }

    let table_api =
        fs::read_to_string(workspace_root.join("atipicial-storage/src/persistence/table/definition.rs"))
            .expect("read typed table API");
    for marker in [
        "pub trait Table:",
        "type KeyCodec:",
        "type ValueCodec:",
        "const NAMESPACE: TableNamespace",
    ] {
        if !table_api.contains(marker) {
            violations.push(format!("typed table API missing `{marker}`"));
        }
    }

    for relative_path in [
        "atipicial-storage/src/persistence/traits/store_snapshot.rs",
        "atipicial-storage/src/persistence/cache/store_cache.rs",
    ] {
        let source = fs::read_to_string(workspace_root.join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"));
        if source.contains("pub fn commit(&mut self)") || source.contains("fn commit(&mut self)") {
            violations.push(format!(
                "{relative_path} reintroduces a backend commit API that can discard errors"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "typed storage architecture regressed:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_canonical_storage_requires_transactional_store_and_uses_data_cache_overlay() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut violations = Vec::new();

    let store_source =
        fs::read_to_string(workspace_root.join("atipicial-storage/src/persistence/traits/store.rs"))
            .expect("read Store trait");
    for forbidden in [
        "try_commit_durable_borrowed_raw_overlay",
        "try_commit_durable_maintenance",
        "fn maintenance_metadata",
    ] {
        if store_source.contains(forbidden) {
            violations.push(format!(
                "Store reintroduces runtime-optional canonical capability `{forbidden}`"
            ));
        }
    }

    let transactional_source = fs::read_to_string(
        workspace_root.join("atipicial-storage/src/persistence/traits/transactional_store.rs"),
    )
    .expect("read TransactionalStore trait");
    for required in [
        "pub trait TransactionalStore: Store",
        "fn commit_canonical_overlay",
        "fn maintenance_metadata",
        "fn commit_maintenance",
    ] {
        if !transactional_source.contains(required) {
            violations.push(format!("TransactionalStore is missing `{required}`"));
        }
    }

    let table_provider =
        fs::read_to_string(workspace_root.join("atipicial-storage/src/persistence/table/provider.rs"))
            .expect("read TableProvider trait");
    if !table_provider.contains("pub trait TableProvider: TransactionalStore") {
        violations
            .push("TableProvider must require the maintenance transaction capability".to_string());
    }

    for relative_path in [
        "atipicial-system/src/composition/builder.rs",
        "atipicial-system/src/composition/core.rs",
        "atipicial-system/src/composition/node.rs",
        "atipicial-system/src/composition/system_context.rs",
    ] {
        let source = fs::read_to_string(workspace_root.join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"));
        if !source.contains("S: TransactionalStore") {
            violations.push(format!(
                "{relative_path} does not enforce transactional canonical storage"
            ));
        }
    }

    let cache_source =
        fs::read_to_string(workspace_root.join("atipicial-storage/src/persistence/data_cache/cache.rs"))
            .expect("read DataCache overlay");
    for required in [
        "pub fn clone_cache(&self)",
        "CacheBacking::Parent",
        "parent.merge_tracked_items_from(self)",
        "pub fn visit_tracked_items_sorted",
    ] {
        if !cache_source.contains(required) {
            violations.push(format!("DataCache overlay is missing `{required}`"));
        }
    }

    assert!(
        violations.is_empty(),
        "canonical transactional-storage architecture regressed:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_node_commit_policy_does_not_redefine_coordinated_storage() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let context_source =
        fs::read_to_string(workspace_root.join("atipicial-node/src/node/context/mod.rs"))
            .expect("read daemon commit-hook composition");
    let policy_source =
        fs::read_to_string(workspace_root.join("atipicial-node/src/node/context/plugins.rs"))
            .expect("read daemon commit policy");
    let storage_source = fs::read_to_string(
        workspace_root.join("atipicial-storage/src/persistence/providers/runtime_store.rs"),
    )
    .expect("read RuntimeStore coordinated storage implementation");

    for forbidden in ["CoordinatedNodeStoreWith", "commit_node_overlays"] {
        assert!(
            !context_source.contains(forbidden) && !policy_source.contains(forbidden),
            "atipicial-node must not redefine coordinated storage through `{forbidden}`"
        );
    }
    for required in [
        "commit_coordinated_overlays(",
        "commit_coordinated_overlays_with_shadow(",
        "commit_coordinated_overlays_with_required_marker(",
    ] {
        assert!(
            storage_source.contains(required),
            "RuntimeStore must own `{required}`"
        );
        assert!(
            policy_source.contains(required),
            "daemon commit policy must call storage-owned `{required}` directly"
        );
    }
    assert!(
        context_source.contains("StateServiceCommitHandlers<RuntimeStore>")
            && context_source.contains("StoreCacheBacking<RuntimeStore>"),
        "daemon hooks must bind canonical Ledger and StateService storage to RuntimeStore"
    );
}
