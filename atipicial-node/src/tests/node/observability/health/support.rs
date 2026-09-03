use std::sync::Arc;

use atipicial_primitives::UInt256;
use atipicial_storage::persistence::providers::memory_store::MemoryStore;

use crate::node::services::NodeServiceHandles;

pub(super) fn test_node() -> atipicial_system::Node {
    atipicial_system::Node::for_test(
        atipicial_config::AtipicialChainSpec::mainnet().expect("valid MainNet chain spec"),
    )
}

pub(super) fn empty_services() -> Arc<NodeServiceHandles<MemoryStore>> {
    service_handles(None, None)
}

pub(super) fn service_handles(
    indexer: Option<Arc<atipicial_indexer::IndexerService>>,
    remote_ledger: Option<Arc<crate::node::remote_ledger::RemoteLedgerStatus>>,
) -> Arc<NodeServiceHandles<MemoryStore>> {
    Arc::new(NodeServiceHandles::new(
        None,
        None,
        indexer,
        None,
        None,
        remote_ledger,
    ))
}

pub(super) fn remote_ledger_node(
    height: u32,
) -> (atipicial_system::Node, Arc<NodeServiceHandles<MemoryStore>>) {
    remote_ledger_node_with_height(Some(height))
}

pub(super) fn remote_ledger_node_with_height(
    height: Option<u32>,
) -> (atipicial_system::Node, Arc<NodeServiceHandles<MemoryStore>>) {
    let node = test_node();
    let services = service_handles(
        None,
        Some(Arc::new(
            crate::node::remote_ledger::RemoteLedgerStatus::new(
                "https://rpc.example.invalid",
                height,
            ),
        )),
    );
    (node, services)
}

pub(super) fn remote_ledger_node_with_error(
    error: &str,
) -> (atipicial_system::Node, Arc<NodeServiceHandles<MemoryStore>>) {
    let node = test_node();
    let services = service_handles(
        None,
        Some(Arc::new(
            crate::node::remote_ledger::RemoteLedgerStatus::unavailable(
                "https://rpc.example.invalid",
                error,
            ),
        )),
    );
    (node, services)
}

pub(super) fn seed_ledger_height(node: &atipicial_system::Node, height: u32) {
    let pointer = atipicial_native_contracts::LedgerContract::new()
        .serialize_hash_index_state(&UInt256::zero(), height)
        .expect("serialize current ledger pointer");
    let mut store = node.store_cache();
    store.add(
        atipicial_storage::StorageKey::new(atipicial_native_contracts::LedgerContract::ID, vec![12]),
        atipicial_storage::StorageItem::from_bytes(pointer),
    );
    store
        .try_commit()
        .expect("commit health-test Ledger height");
}

pub(super) fn indexed_service_at(height: u32) -> Arc<atipicial_indexer::IndexerService> {
    let indexer = Arc::new(atipicial_indexer::IndexerService::new());
    let mut header = atipicial_payloads::Header::new();
    header.set_index(height);
    indexer
        .index_block(&atipicial_payloads::Block::from_parts(header, Vec::new()))
        .expect("index block");
    indexer
}
