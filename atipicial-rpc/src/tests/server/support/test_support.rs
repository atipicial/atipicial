//! Shared fixtures for the RPC server test-suites.
//!
//! The legacy fixtures constructed a `AtipicialSystem`; the reth-style
//! replacement composes a [`Node`] over an in-memory store plus a
//! *live* [`atipicial_blockchain::BlockchainService`] loop so the relay
//! endpoints (`sendrawtransaction`, `submitblock`, wallet sends) get
//! real request/response round-trips. The blockchain service shares
//! the node's memory pool and header cache, exactly as the production
//! composition root is expected to wire them.

use std::sync::Arc;

use atipicial_blockchain::ledger_provider::{StorageLedgerProvider, TransactionAdmissionLedger};
use atipicial_blockchain::{BlockchainService, HeaderCache, LedgerContext, MempoolLike, SystemContext};
use atipicial_config::{ChainSpecProvider, AtipicialChainSpec, ProtocolSettings};
use atipicial_mempool::{MemoryPool, TransactionAdmissionOutcome, TransactionOrigin, TxPoolConfig};
use atipicial_network::NetworkHandle;
use atipicial_primitives::UInt160;
use atipicial_storage::persistence::providers::{MemoryStore, RuntimeStore};
use atipicial_storage::persistence::store::Store;
use atipicial_storage::persistence::{StoreCache, StoreCacheBacking, TransactionalStore};
use atipicial_storage::{StorageItem, StorageKey};
use atipicial_system::{Node, NodeBuilder};
use num_bigint::BigInt;
use parking_lot::Mutex;

/// Minimal [`SystemContext`] for the fixture blockchain service.
struct FixtureContext {
    chain_spec: Arc<AtipicialChainSpec>,
    snapshot: Arc<atipicial_storage::persistence::StoreDataCache<MemoryStore>>,
    store_cache: Mutex<StoreCache<MemoryStore>>,
    native_contract_provider: Arc<atipicial_native_contracts::StandardNativeProvider>,
}

impl std::fmt::Debug for FixtureContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FixtureContext")
            .field("network", &self.chain_spec.network_magic())
            .finish_non_exhaustive()
    }
}

impl ChainSpecProvider for FixtureContext {
    type ChainSpec = AtipicialChainSpec;

    fn chain_spec(&self) -> Arc<Self::ChainSpec> {
        Arc::clone(&self.chain_spec)
    }
}

impl SystemContext for FixtureContext {
    type NativeProvider = atipicial_native_contracts::StandardNativeProvider;
    type CacheBacking = StoreCacheBacking<MemoryStore>;

    fn current_height(&self) -> u32 {
        atipicial_native_contracts::LedgerContract::new()
            .current_index(&self.snapshot)
            .unwrap_or(0)
    }

    fn store_snapshot(
        &self,
    ) -> Option<Arc<atipicial_storage::persistence::DataCache<Self::CacheBacking>>> {
        Some(Arc::clone(&self.snapshot))
    }

    fn native_contract_provider(&self) -> Option<Arc<Self::NativeProvider>> {
        Some(Arc::clone(&self.native_contract_provider))
    }

    fn commit_to_store(&self) -> Result<(), String> {
        self.store_cache
            .lock()
            .try_commit_durable()
            .map_err(|error| error.to_string())
    }
}

/// [`MempoolLike`] adapter over the node's real [`MemoryPool`], so the
/// blockchain service's `add_transaction` admissions land in the same
/// pool the RPC handlers read.
///
/// The adapter uses the same canonical admission API as production while
/// selecting the fixture's store-backed Ledger provider.
struct NodeMempoolAdapter {
    pool: Arc<MemoryPool>,
    store: Arc<MemoryStore>,
}

impl std::fmt::Debug for NodeMempoolAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeMempoolAdapter")
            .field("total", &self.pool.total_count())
            .finish()
    }
}

impl MempoolLike for NodeMempoolAdapter {
    fn add_transaction<B, L>(
        &self,
        origin: TransactionOrigin,
        transaction: &atipicial_payloads::Transaction,
        _snapshot: &atipicial_storage::DataCache<B>,
        _ledger_provider: &L,
    ) -> TransactionAdmissionOutcome
    where
        B: atipicial_storage::CacheRead,
        L: atipicial_mempool::AdmissionLedgerProvider,
    {
        let store = StoreCache::<MemoryStore>::new_from_store(Arc::clone(&self.store), true);
        let provider =
            TransactionAdmissionLedger::new(StorageLedgerProvider::new(store.data_cache()));
        self.pool
            .add_transaction(origin, transaction.clone(), store.data_cache(), &provider)
    }
}

/// Admits a locally submitted transaction through the canonical typed pool
/// boundary and a snapshot-bound Ledger provider.
pub(crate) fn admit_local_transaction<B: atipicial_storage::CacheRead>(
    pool: &MemoryPool,
    transaction: &atipicial_payloads::Transaction,
    snapshot: &atipicial_storage::DataCache<B>,
) -> TransactionAdmissionOutcome {
    let provider = TransactionAdmissionLedger::new(StorageLedgerProvider::new(snapshot));
    pool.add_transaction(
        TransactionOrigin::Local,
        transaction.clone(),
        snapshot,
        &provider,
    )
}

/// Builds an `Arc<NodeContext>` test system over an in-memory store.
///
/// When called from inside a Tokio runtime (the `#[tokio::test]`
/// fixtures), the blockchain service loop is spawned so handle
/// round-trips resolve; outside a runtime the service is dropped,
/// which makes handle sends fail fast instead of hanging — the
/// synchronous tests never exercise the relay path.
pub(crate) fn test_system_with_services(
    settings: ProtocolSettings,
    services: crate::server::RpcServices<RuntimeStore>,
) -> Arc<crate::server::NodeContext> {
    let (network, _cmd_rx, _event_tx) = NetworkHandle::channel(64, 64);
    test_system_with_services_and_network(settings, services, network)
}

fn test_system_with_services_and_network(
    settings: ProtocolSettings,
    services: crate::server::RpcServices<RuntimeStore>,
    network: NetworkHandle,
) -> Arc<crate::server::NodeContext> {
    let chain_spec = atipicial_test_fixtures::test_chain_spec(settings);
    let memory_store: Arc<MemoryStore> = Arc::new(MemoryStore::new());
    let storage: Arc<RuntimeStore> = Arc::new(RuntimeStore::Memory(memory_store.as_ref().clone()));
    let native_contract_provider = Arc::new(atipicial_native_contracts::StandardNativeProvider::new());
    let mempool = Arc::new(MemoryPool::new_with_native_contract_provider(
        Arc::clone(&chain_spec),
        TxPoolConfig::default(),
        Arc::clone(&native_contract_provider),
    ));
    let header_cache = Arc::new(HeaderCache::default());
    let store_cache = StoreCache::<MemoryStore>::new_from_store(Arc::clone(&memory_store), false);
    let snapshot = Arc::new(store_cache.data_cache().clone());

    let system_ctx = Arc::new(FixtureContext {
        chain_spec: Arc::clone(&chain_spec),
        snapshot,
        store_cache: Mutex::new(store_cache),
        native_contract_provider: Arc::clone(&native_contract_provider),
    });
    let mempool_like = Arc::new(NodeMempoolAdapter {
        pool: Arc::clone(&mempool),
        store: Arc::clone(&memory_store),
    });
    let (service, blockchain) = BlockchainService::with_defaults(
        system_ctx,
        Arc::new(LedgerContext::default()),
        Arc::clone(&header_cache),
        mempool_like,
    );
    if let Ok(runtime) = tokio::runtime::Handle::try_current() {
        runtime.spawn(service.run());
    }

    let node = NodeBuilder::new(
        chain_spec,
        storage,
        blockchain,
        network,
        mempool,
        header_cache,
        native_contract_provider,
    )
    .build();
    let node = Arc::new(node);
    seed_native_contract_records(&node);
    seed_genesis_state(&node);
    Arc::new(crate::server::NodeContext::from_parts(
        node.chain_spec(),
        node.storage(),
        node.blockchain(),
        node.network(),
        node.mempool(),
        node.header_cache(),
        services,
        node.native_contract_provider(),
        node.cold_ledger_provider(),
    ))
}

pub(crate) fn test_system_with_network_commands(
    settings: ProtocolSettings,
) -> (
    Arc<crate::server::NodeContext>,
    tokio::sync::mpsc::Receiver<atipicial_network::NetworkCommand>,
) {
    let (network, cmd_rx, _event_tx) = NetworkHandle::channel(64, 64);
    let system =
        test_system_with_services_and_network(settings, crate::server::RpcServices::new(), network);
    (system, cmd_rx)
}

pub(crate) fn test_system(settings: ProtocolSettings) -> Arc<crate::server::NodeContext> {
    test_system_with_services(settings, crate::server::RpcServices::new())
}

/// Re-composes an existing fixture context with an immutable Ledger fallback
/// while retaining the same hot store and live service handles.
pub(crate) fn test_system_with_cold_ledger_provider(
    system: &Arc<crate::server::NodeContext>,
    cold_ledger_provider: atipicial_blockchain::OptionalStaticLedgerProvider,
) -> Arc<crate::server::NodeContext> {
    Arc::new(crate::server::NodeContext::from_parts(
        system.chain_spec(),
        system.storage(),
        system.blockchain(),
        system.network(),
        system.mempool(),
        system.header_cache(),
        system.services().clone(),
        system.native_contract_provider(),
        cold_ledger_provider,
    ))
}

/// `ContractManagement.PREFIX_CONTRACT` — the per-contract record
/// prefix (verified against `atipicial-native-contracts`).
const PREFIX_CONTRACT: u8 = 8;
/// `ContractManagement.PREFIX_CONTRACT_HASH` — the id → hash index
/// prefix.
const PREFIX_CONTRACT_HASH: u8 = 12;
/// `ContractManagement::ID`.
const CONTRACT_MANAGEMENT_ID: i32 = -1;
/// `ContractManagement.Prefix_MinimumDeploymentFee`.
const CONTRACT_MANAGEMENT_PREFIX_MINIMUM_DEPLOYMENT_FEE: u8 = 20;
/// `ContractManagement.Prefix_NextAvailableId`.
const CONTRACT_MANAGEMENT_PREFIX_NEXT_AVAILABLE_ID: u8 = 15;
/// C# `ContractManagement.DefaultMinimumDeploymentFee` (10 GAS in datoshi).
const CONTRACT_MANAGEMENT_DEFAULT_MINIMUM_DEPLOYMENT_FEE: i64 = 10_0000_0000;
/// C# genesis value for `ContractManagement.Prefix_NextAvailableId`.
const CONTRACT_MANAGEMENT_DEFAULT_NEXT_AVAILABLE_ID: i64 = 1;
/// `PolicyContract.Prefix_FeePerByte`.
const POLICY_PREFIX_FEE_PER_BYTE: u8 = 10;
/// `PolicyContract.Prefix_ExecFeeFactor`.
const POLICY_PREFIX_EXEC_FEE_FACTOR: u8 = 18;
/// `PolicyContract.Prefix_StoragePrice`.
const POLICY_PREFIX_STORAGE_PRICE: u8 = 19;
/// C# `PolicyContract.DefaultFeePerByte`.
const POLICY_DEFAULT_FEE_PER_BYTE: i64 = 1000;
/// C# `PolicyContract.DefaultExecFeeFactor`.
const POLICY_DEFAULT_EXEC_FEE_FACTOR: i64 = 30;
/// C# `PolicyContract.DefaultStoragePrice`.
const POLICY_DEFAULT_STORAGE_PRICE: i64 = 100_000;

/// Seeds the deployed-contract records for every standard native
/// contract, mirroring the post-genesis chain state so
/// `System.Contract.Call` probes and `getcontractstate` queries can
/// resolve the natives (genesis performs these deployments on a real
/// chain).
fn seed_native_contract_records<S>(node: &Node<atipicial_native_contracts::StandardNativeProvider, S>)
where
    S: TransactionalStore + 'static,
{
    let settings = node.chain_spec().protocol_settings_arc();
    let mut store = node.store_cache();
    for contract in atipicial_native_contracts::standard_native_contracts() {
        let Some(state) = contract.contract_state(&settings, 0) else {
            continue;
        };
        let record = state
            .serialize_contract_record()
            .expect("serialize native contract record");

        let mut record_key = Vec::with_capacity(1 + 20);
        record_key.push(PREFIX_CONTRACT);
        record_key.extend_from_slice(&state.hash.to_bytes());
        store.add(
            StorageKey::new(CONTRACT_MANAGEMENT_ID, record_key),
            StorageItem::from_bytes(record),
        );

        let mut id_key = Vec::with_capacity(1 + 4);
        id_key.push(PREFIX_CONTRACT_HASH);
        id_key.extend_from_slice(&state.id.to_be_bytes());
        store.add(
            StorageKey::new(CONTRACT_MANAGEMENT_ID, id_key),
            StorageItem::from_bytes(state.hash.to_bytes().to_vec()),
        );
    }
    store.try_commit().expect("commit test store");
}

/// `FungibleToken.PREFIX_ACCOUNT` — the per-account AEP-17 balance
/// prefix (verified against `atipicial-native-contracts`'s
/// `AEP17_PREFIX_ACCOUNT`).
const AEP17_PREFIX_ACCOUNT: u8 = 20;
/// `FungibleToken.PREFIX_TOTAL_SUPPLY` (verified against
/// `atipicial-native-contracts`'s `AEP17_PREFIX_TOTAL_SUPPLY`).
const AEP17_PREFIX_TOTAL_SUPPLY: u8 = 11;
/// `AtipicialCoin.Prefix_Committee` (verified against `atipicial_coin.rs`).
const ATC_PREFIX_COMMITTEE: u8 = 14;
/// `AtipicialCoin.Prefix_VotersCount` (verified against `atipicial_coin.rs`).
const ATC_PREFIX_VOTERS_COUNT: u8 = 1;
/// `AtipicialCoin.Prefix_AtipicialDollarPerBlock` (verified against `atipicial_coin.rs`);
/// the key suffix is the big-endian block index.
const ATC_PREFIX_GAS_PER_BLOCK: u8 = 29;
/// `AtipicialCoin.Prefix_RegisterPrice` (verified against `atipicial_coin.rs`).
const ATC_PREFIX_REGISTER_PRICE: u8 = 13;
/// `LedgerContract.Prefix_Block` (verified against
/// `ledger_contract.rs`); the key suffix is the block hash.
const LEDGER_PREFIX_BLOCK: u8 = 5;
/// `LedgerContract.Prefix_BlockHash` (verified against
/// `ledger_contract.rs`); the key suffix is the little-endian index
/// (the byte order the Rust reader uses).
const LEDGER_PREFIX_BLOCK_HASH: u8 = 9;
/// `LedgerContract.Prefix_CurrentBlock` (verified against
/// `ledger_contract.rs`).
const LEDGER_PREFIX_CURRENT_BLOCK: u8 = 12;

/// C# `AtipicialCoin.TotalAmount` (100 million indivisible ATC).
const ATC_TOTAL_AMOUNT: i64 = 1_000_000_000;
/// C# `AtipicialCoin` initial ATD-per-block (5 GAS in datoshi).
const ATC_DEFAULT_GAS_PER_BLOCK: i64 = 5_0000_0000;
/// C# `AtipicialCoin` initial candidate-register price (1000 GAS in
/// datoshi).
const ATC_DEFAULT_REGISTER_PRICE: i64 = 1000_0000_0000;

/// Serializes a [`atipicial_vm::StackItem`] with the canonical
/// `BinarySerializer` wire format the native contracts read.
fn serialize_stack_item(item: &atipicial_vm::StackItem) -> Vec<u8> {
    atipicial_serialization::BinarySerializer::serialize(item, &atipicial_vm::ExecutionEngineLimits::default())
        .expect("serialize stack item")
}

/// C# `Contract.GetBFTAddress(validators)`: the script hash of the
/// `m = n - (n - 1) / 3` multi-signature contract over the validators.
fn bft_address(validators: &[atipicial_crypto::ECPoint]) -> UInt160 {
    let m = validators.len() - (validators.len() - 1) / 3;
    atipicial_execution::Helper::to_script_hash(&atipicial_vm::Contract::create(
        vec![],
        atipicial_vm::Contract::create_multi_sig_redeem_script(m, validators),
    ))
}

/// C# `AtipicialSystem.CreateGenesisBlock(settings)` (verified against the
/// in-tree v3.10.1 reference): version 0, zero previous/merkle hashes,
/// the 2026-07-20T00:00:00Z timestamp, the Bitcoin-genesis nonce, and
/// a `PUSH1` witness, with `NextConsensus` set to the BFT address of
/// the standby validators.
fn genesis_header(settings: &ProtocolSettings) -> atipicial_payloads::Header {
    use atipicial_vm::OpCode;

    let mut header = atipicial_payloads::Header::new();
    header.set_version(0);
    header.set_prev_hash(atipicial_primitives::UInt256::default());
    header.set_merkle_root(atipicial_primitives::UInt256::default());
    header.set_timestamp(1_784_505_600_000);
    header.set_nonce(2_083_236_893);
    header.set_index(0);
    header.set_primary_index(0);
    header.set_next_consensus(bft_address(&settings.standby_validators()));
    header.witness = atipicial_payloads::Witness::new_with_scripts(vec![], vec![OpCode::PUSH1.byte()]);
    header
}

/// Seeds the storage records that genesis block-0 persistence writes
/// on a real chain — the `AtipicialCoin` / `AtipicialDollar` `Initialize` effects
/// and the `LedgerContract` block records — so the RPC handlers under
/// test observe post-genesis chain state. Every record format is the
/// byte-exact write counterpart of the corresponding
/// `atipicial-native-contracts` reader.
fn seed_genesis_state<S>(node: &Node<atipicial_native_contracts::StandardNativeProvider, S>)
where
    S: TransactionalStore + 'static,
{
    use atipicial_io::Serializable;
    use atipicial_vm::StackItem;

    let settings = node.chain_spec().protocol_settings_arc();
    let mut store = node.store_cache();

    let signed_le = |value: i64| BigInt::from(value).to_signed_bytes_le();

    // --- ContractManagement.Initialize / PolicyContract.Initialize ---
    // Genesis-active native initializers seed scalar settings before contract
    // calls and transaction verification can read them.
    store.update(
        StorageKey::new(
            CONTRACT_MANAGEMENT_ID,
            vec![CONTRACT_MANAGEMENT_PREFIX_MINIMUM_DEPLOYMENT_FEE],
        ),
        StorageItem::from_bytes(signed_le(
            CONTRACT_MANAGEMENT_DEFAULT_MINIMUM_DEPLOYMENT_FEE,
        )),
    );
    store.update(
        StorageKey::new(
            CONTRACT_MANAGEMENT_ID,
            vec![CONTRACT_MANAGEMENT_PREFIX_NEXT_AVAILABLE_ID],
        ),
        StorageItem::from_bytes(signed_le(CONTRACT_MANAGEMENT_DEFAULT_NEXT_AVAILABLE_ID)),
    );
    store.update(
        StorageKey::new(
            atipicial_native_contracts::PolicyContract::ID,
            vec![POLICY_PREFIX_FEE_PER_BYTE],
        ),
        StorageItem::from_bytes(signed_le(POLICY_DEFAULT_FEE_PER_BYTE)),
    );
    store.update(
        StorageKey::new(
            atipicial_native_contracts::PolicyContract::ID,
            vec![POLICY_PREFIX_EXEC_FEE_FACTOR],
        ),
        StorageItem::from_bytes(signed_le(POLICY_DEFAULT_EXEC_FEE_FACTOR)),
    );
    store.update(
        StorageKey::new(
            atipicial_native_contracts::PolicyContract::ID,
            vec![POLICY_PREFIX_STORAGE_PRICE],
        ),
        StorageItem::from_bytes(signed_le(POLICY_DEFAULT_STORAGE_PRICE)),
    );

    // --- AtipicialCoin.Initialize (C# v3.10.1) ---
    // Committee cache: Array of Struct[pubkey, votes = 0] in standby
    // order (C# `CachedCommittee` of `StandbyCommittee.Select(p => (p, 0))`).
    let committee_items: Vec<StackItem> = settings
        .standby_committee
        .iter()
        .map(|point| {
            StackItem::from_struct(vec![
                StackItem::from_byte_string(point.as_bytes().to_vec()),
                StackItem::from_int(BigInt::from(0)),
            ])
        })
        .collect();
    let atipicial_id = atipicial_native_contracts::AtipicialCoin::ID;
    let committee_array = StackItem::from_array(committee_items);
    store.update(
        StorageKey::new(atipicial_id, vec![ATC_PREFIX_COMMITTEE]),
        StorageItem::from_bytes(serialize_stack_item(&committee_array)),
    );
    // Voters count: an empty byte array (BigInteger zero), C#-exact.
    store.update(
        StorageKey::new(atipicial_id, vec![ATC_PREFIX_VOTERS_COUNT]),
        StorageItem::from_bytes(Vec::new()),
    );
    // Gas-per-block record at index 0 (big-endian index suffix).
    let mut atipicial_dollar_per_block_key = vec![ATC_PREFIX_GAS_PER_BLOCK];
    atipicial_dollar_per_block_key.extend_from_slice(&0u32.to_be_bytes());
    store.update(
        StorageKey::new(atipicial_id, atipicial_dollar_per_block_key),
        StorageItem::from_bytes(signed_le(ATC_DEFAULT_GAS_PER_BLOCK)),
    );
    store.update(
        StorageKey::new(atipicial_id, vec![ATC_PREFIX_REGISTER_PRICE]),
        StorageItem::from_bytes(signed_le(ATC_DEFAULT_REGISTER_PRICE)),
    );

    // Genesis ATC mint: 100M ATC to the standby-validator BFT address.
    // `AtipicialAccountState` = Struct[Balance, BalanceHeight, VoteTo(Null),
    // LastGasPerVote] (the write counterpart of
    // `decode_atipicial_account_state`).
    let bft = bft_address(&settings.standby_validators());
    let atipicial_account_state = StackItem::from_struct(vec![
        StackItem::from_int(BigInt::from(ATC_TOTAL_AMOUNT)),
        StackItem::from_int(BigInt::from(0)),
        StackItem::null(),
        StackItem::from_int(BigInt::from(0)),
    ]);
    let mut atipicial_account_key = Vec::with_capacity(1 + 20);
    atipicial_account_key.push(AEP17_PREFIX_ACCOUNT);
    atipicial_account_key.extend_from_slice(&bft.to_bytes());
    store.update(
        StorageKey::new(atipicial_id, atipicial_account_key),
        StorageItem::from_bytes(serialize_stack_item(&atipicial_account_state)),
    );
    store.update(
        StorageKey::new(atipicial_id, vec![AEP17_PREFIX_TOTAL_SUPPLY]),
        StorageItem::from_bytes(signed_le(ATC_TOTAL_AMOUNT)),
    );

    // --- AtipicialDollar.Initialize: mint InitialGasDistribution to the BFT
    // address (plain AEP-17 account state: Struct[balance]).
    let initial_gas = BigInt::from(settings.initial_gas_distribution);
    let gas_account_state = StackItem::from_struct(vec![StackItem::from_int(initial_gas.clone())]);
    let mut gas_account_key = Vec::with_capacity(1 + 20);
    gas_account_key.push(AEP17_PREFIX_ACCOUNT);
    gas_account_key.extend_from_slice(&bft.to_bytes());
    store.update(
        StorageKey::new(atipicial_native_contracts::AtipicialDollar::ID, gas_account_key),
        StorageItem::from_bytes(serialize_stack_item(&gas_account_state)),
    );
    store.update(
        StorageKey::new(
            atipicial_native_contracts::AtipicialDollar::ID,
            vec![AEP17_PREFIX_TOTAL_SUPPLY],
        ),
        StorageItem::from_bytes(initial_gas.to_signed_bytes_le()),
    );

    // --- LedgerContract genesis-block records ---
    let header = genesis_header(&settings);
    let genesis_hash = header.try_hash().expect("genesis header hash");
    let ledger_id = atipicial_native_contracts::LedgerContract::ID;

    // Trimmed block record (`Prefix_Block` + hash).
    let trimmed = atipicial_payloads::TrimmedBlock::new(header, Vec::new());
    let mut writer = atipicial_io::BinaryWriter::new();
    trimmed
        .serialize(&mut writer)
        .expect("serialize genesis trimmed block");
    let mut block_key = Vec::with_capacity(1 + 32);
    block_key.push(LEDGER_PREFIX_BLOCK);
    block_key.extend_from_slice(&genesis_hash.to_bytes());
    store.update(
        StorageKey::new(ledger_id, block_key),
        StorageItem::from_bytes(writer.into_bytes()),
    );

    // Index -> hash record (`Prefix_BlockHash` + big-endian index, the
    // C# `CreateStorageKey(prefix, uint)` / `KeyBuilder.AddBigEndian`
    // layout the `LedgerContract` reader expects).
    let mut hash_key = Vec::with_capacity(1 + 4);
    hash_key.push(LEDGER_PREFIX_BLOCK_HASH);
    hash_key.extend_from_slice(&0u32.to_be_bytes());
    store.update(
        StorageKey::new(ledger_id, hash_key),
        StorageItem::from_bytes(genesis_hash.to_bytes().to_vec()),
    );

    // Current-block pointer (`Prefix_CurrentBlock`): the C# `HashIndexState`
    // interoperable stack item (`Struct[ByteString(hash), Integer(index)]`)
    // serialized with `BinarySerializer` — exactly what the `LedgerContract`
    // reader (`current_index` / `current_hash`) decodes.
    let pointer = atipicial_native_contracts::LedgerContract::new()
        .serialize_hash_index_state(&genesis_hash, 0)
        .expect("serialize genesis HashIndexState pointer");
    store.update(
        StorageKey::new(ledger_id, vec![LEDGER_PREFIX_CURRENT_BLOCK]),
        StorageItem::from_bytes(pointer),
    );

    store.try_commit().expect("commit test store");
}

/// Seeds a ATD balance for `account` directly into `store`, writing
/// the byte-exact AEP-17 account-state record the native `balanceOf`
/// reads: a `BinarySerializer`-encoded `Struct { Integer(balance) }`
/// under `AtipicialDollar::ID` / `[PREFIX_ACCOUNT | account]`.
pub(crate) fn seed_gas_balance<S>(store: &mut StoreCache<S>, account: &UInt160, amount: BigInt)
where
    S: Store,
{
    let state = atipicial_vm::StackItem::from_struct(vec![atipicial_vm::StackItem::from_int(amount)]);
    let bytes = atipicial_serialization::BinarySerializer::serialize(
        &state,
        &atipicial_vm::ExecutionEngineLimits::default(),
    )
    .expect("serialize AEP-17 account state");

    let mut key_bytes = Vec::with_capacity(1 + 20);
    key_bytes.push(AEP17_PREFIX_ACCOUNT);
    key_bytes.extend_from_slice(&account.to_bytes());
    store.add(
        StorageKey::new(atipicial_native_contracts::AtipicialDollar::ID, key_bytes),
        StorageItem::from_bytes(bytes),
    );
}
