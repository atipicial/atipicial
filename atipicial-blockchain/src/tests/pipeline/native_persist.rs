use super::trace::{
    SlowTxFilter, TraceTxFilter, VmProfileFilter, format_vm_hardfork_context,
    format_vm_hottest_opcodes, format_vm_hottest_scripts, format_vm_opcode_classes,
    trace_tx_artifact,
};
use super::*;
use atipicial_manifest::{ContractManifest, ContractMethodDescriptor, AefFile};
use atipicial_payloads::Header;
use atipicial_primitives::{UInt160, UInt256};
use atipicial_serialization::BinarySerializer;
use atipicial_storage::StorageKey;
use atipicial_test_fixtures::test_chain_spec;
use atipicial_vm::ExecutionEngineLimits;
use atipicial_vm::script_builder::ScriptBuilder;
use num_bigint::BigInt;
use std::sync::atomic::{AtomicUsize, Ordering};

use atipicial_execution::ExecutionArtifactLimits;
use atipicial_execution::specialization::{
    CandidateRouteConfig, FLAMINGO_FACTORY_PAIR_KEY_CANDIDATE_ID,
    FLAMINGO_FACTORY_PAIR_KEY_CANDIDATE_VERSION, SpecializationControl,
    SpecializationControlConfig, SpecializationControlLimits,
};
use atipicial_vm::SpecializationMode;

#[path = "native_persist/dynamic_hooks.rs"]
mod dynamic_hooks;
#[path = "native_persist/trace.rs"]
mod trace_tests;

/// ATC `Prefix_Committee` (C# AtipicialCoin).
const ATC_PREFIX_COMMITTEE: u8 = 14;
/// ATC `Prefix_VotersCount`.
const ATC_PREFIX_VOTERS_COUNT: u8 = 1;
/// ATC `Prefix_AtipicialDollarPerBlock`.
const ATC_PREFIX_GAS_PER_BLOCK: u8 = 29;
/// ATC `Prefix_RegisterPrice`.
const ATC_PREFIX_REGISTER_PRICE: u8 = 13;
/// Shared AEP-17 `Prefix_Account` / `Prefix_TotalSupply`.
const AEP17_PREFIX_ACCOUNT: u8 = 20;
const AEP17_PREFIX_TOTAL_SUPPLY: u8 = 11;
/// Oracle `Prefix_Price` / `Prefix_RequestId`.
const ORACLE_PREFIX_PRICE: u8 = 5;
const ORACLE_PREFIX_REQUEST_ID: u8 = 9;

type StandardNativePersistResources =
    NativePersistResources<atipicial_native_contracts::StandardNativeProvider>;

fn standard_resources() -> StandardNativePersistResources {
    NativePersistResources::from_provider(Arc::new(
        atipicial_native_contracts::StandardNativeProvider::new(),
    ))
}

fn chain_spec_for_settings(settings: &ProtocolSettings) -> Arc<atipicial_config::AtipicialChainSpec> {
    test_chain_spec(settings.clone())
}

fn shadow_resources(
    strict_replay: bool,
    limits: ExecutionArtifactLimits,
) -> StandardNativePersistResources {
    let config = SpecializationControlConfig::try_enabled(
        strict_replay,
        SpecializationControlLimits::DEFAULT,
        [CandidateRouteConfig::new(
            FLAMINGO_FACTORY_PAIR_KEY_CANDIDATE_ID,
            FLAMINGO_FACTORY_PAIR_KEY_CANDIDATE_VERSION,
            SpecializationMode::Shadow,
        )],
    )
    .expect("valid test shadow configuration");
    standard_resources().with_specialization_shadow(SpecializationControl::new(config), limits)
}

fn persist_with_resources(
    snapshot: Arc<DataCache>,
    block: Arc<Block>,
    settings: &ProtocolSettings,
    resources: &StandardNativePersistResources,
) -> CoreResult<NativePersistOutcome> {
    persist_block_natives_with_resources(
        snapshot,
        block,
        Arc::new(settings.clone()),
        NativePersistOptions::default(),
        resources,
    )
}

struct CountingNativeProvider {
    inner: atipicial_native_contracts::StandardNativeProvider,
    all_contracts_calls: Arc<AtomicUsize>,
}

impl CountingNativeProvider {
    fn new(all_contracts_calls: Arc<AtomicUsize>) -> Self {
        Self {
            inner: atipicial_native_contracts::StandardNativeProvider::new(),
            all_contracts_calls,
        }
    }
}

impl atipicial_execution::native_contract_provider::NativeContractProvider for CountingNativeProvider {
    type Contract = atipicial_native_contracts::StandardNativeContract;

    fn get_native_contract(&self, hash: &UInt160) -> Option<Self::Contract> {
        self.inner.get_native_contract(hash)
    }

    fn all_native_contracts(&self) -> Vec<Self::Contract> {
        self.all_contracts_calls.fetch_add(1, Ordering::Relaxed);
        self.inner.all_native_contracts()
    }

    fn all_native_contract_hashes(&self) -> Vec<UInt160> {
        self.inner.all_native_contract_hashes()
    }

    fn current_block_index<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
    ) -> CoreResult<u32> {
        self.inner.current_block_index(snapshot)
    }

    fn policy_is_blocked<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        account: &UInt160,
    ) -> CoreResult<bool> {
        self.inner.policy_is_blocked(snapshot, account)
    }

    fn policy_whitelisted_fee<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        contract_hash: &UInt160,
        method: &str,
        param_count: u32,
    ) -> CoreResult<Option<i64>> {
        self.inner
            .policy_whitelisted_fee(snapshot, contract_hash, method, param_count)
    }

    fn exec_fee_factor_raw<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
    ) -> CoreResult<u32> {
        self.inner.exec_fee_factor_raw(snapshot)
    }

    fn storage_price<B: atipicial_storage::CacheRead>(&self, snapshot: &DataCache<B>) -> CoreResult<u32> {
        self.inner.storage_price(snapshot)
    }

    fn contract_state<B: atipicial_storage::CacheRead>(
        &self,
        snapshot: &DataCache<B>,
        hash: &UInt160,
    ) -> CoreResult<Option<atipicial_execution::ContractState>> {
        self.inner.contract_state(snapshot, hash)
    }
}

fn atipicial_id() -> i32 {
    atipicial_native_contracts::AtipicialCoin::ID
}

fn get(snapshot: &DataCache, id: i32, key: Vec<u8>) -> Option<Vec<u8>> {
    snapshot
        .get(&StorageKey::new(id, key))
        .map(|item| item.value_bytes().into_owned())
}

fn fund_gas(snapshot: &DataCache, account: &UInt160, amount: i64) {
    let mut gas_key = vec![AEP17_PREFIX_ACCOUNT];
    gas_key.extend_from_slice(&account.to_bytes());
    let account_state = StackItem::from_struct(vec![StackItem::from_int(BigInt::from(amount))]);
    let account_bytes =
        BinarySerializer::serialize(&account_state, &ExecutionEngineLimits::default()).unwrap();
    snapshot.add(
        StorageKey::new(atipicial_native_contracts::AtipicialDollar::ID, gas_key),
        atipicial_storage::StorageItem::from_bytes(account_bytes),
    );
}

fn deploy_contract(snapshot: &DataCache, state: &atipicial_execution::ContractState) {
    let mut key = vec![0x08];
    key.extend_from_slice(&state.hash.to_bytes());
    snapshot.add(
        StorageKey::new(atipicial_native_contracts::ContractManagement::ID, key),
        atipicial_storage::StorageItem::from_bytes(state.serialize_contract_record().unwrap()),
    );
}

fn throwing_aep17_receiver_contract(hash: UInt160) -> atipicial_execution::ContractState {
    let aef = AefFile::new(
        "throwing-aep17-receiver".to_string(),
        vec![atipicial_vm::OpCode::PUSH1.byte(), atipicial_vm::OpCode::THROW.byte()],
    );
    let mut manifest = ContractManifest::new("ThrowingAep17Receiver".to_string());
    manifest.abi.methods.push(
        ContractMethodDescriptor::new(
            "onAEP17Payment".to_string(),
            vec![
                atipicial_manifest::ContractParameterDefinition::new(
                    "from".to_string(),
                    atipicial_primitives::ContractParameterType::Hash160,
                )
                .unwrap(),
                atipicial_manifest::ContractParameterDefinition::new(
                    "amount".to_string(),
                    atipicial_primitives::ContractParameterType::Integer,
                )
                .unwrap(),
                atipicial_manifest::ContractParameterDefinition::new(
                    "data".to_string(),
                    atipicial_primitives::ContractParameterType::Any,
                )
                .unwrap(),
            ],
            atipicial_primitives::ContractParameterType::Void,
            0,
            false,
        )
        .expect("method descriptor"),
    );
    atipicial_execution::ContractState::new(7, hash, aef, manifest)
}

fn policy_get_exec_fee_factor_script() -> Vec<u8> {
    let mut builder = ScriptBuilder::new();
    builder.emit_push_int(0);
    builder.emit_pack();
    builder.emit_push_int(i64::from(atipicial_primitives::CallFlags::READ_STATES.bits()));
    builder.emit_push_string("getExecFeeFactor");
    builder.emit_push(&atipicial_native_contracts::PolicyContract::script_hash().to_array());
    builder
        .emit_syscall("System.Contract.Call")
        .expect("System.Contract.Call");
    builder.to_array()
}

fn gas_transfer_script(from: &UInt160, to: &UInt160, amount: i64) -> Vec<u8> {
    let mut builder = ScriptBuilder::new();
    builder.emit_opcode(atipicial_vm::OpCode::PUSHNULL);
    builder.emit_push_int(amount);
    builder.emit_push(&to.to_array());
    builder.emit_push(&from.to_array());
    builder.emit_push_int(4);
    builder.emit_pack();
    builder.emit_push_int(i64::from(atipicial_primitives::CallFlags::ALL.bits()));
    builder.emit_push_string("transfer");
    builder.emit_push(&atipicial_native_contracts::AtipicialDollar::script_hash().to_array());
    builder
        .emit_syscall("System.Contract.Call")
        .expect("System.Contract.Call");
    builder.to_array()
}

fn signed_test_tx(sender: UInt160, nonce: u32, script: Vec<u8>) -> atipicial_payloads::Transaction {
    let mut tx = atipicial_payloads::Transaction::new();
    tx.set_nonce(nonce);
    tx.set_script(script);
    tx.set_system_fee(1_0000_0000);
    tx.set_signers(vec![atipicial_payloads::Signer::new(
        sender,
        atipicial_primitives::WitnessScope::NONE,
    )]);
    tx.set_witnesses(vec![atipicial_payloads::Witness::empty()]);
    tx
}

#[test]
fn genesis_block_matches_csharp_create_genesis_block() {
    let settings = ProtocolSettings::default();
    let block = genesis_block(&chain_spec_for_settings(&settings)).expect("genesis block");
    assert_eq!(block.index(), 0);
    assert_eq!(block.header.version(), 0);
    assert_eq!(*block.header.prev_hash(), UInt256::zero());
    assert_eq!(*block.header.merkle_root(), UInt256::zero());
    assert_eq!(block.header.timestamp(), 1_784_505_600_000);
    assert_eq!(block.header.nonce(), 2_083_236_893);
    assert_eq!(block.header.primary_index(), 0);
    assert!(block.transactions.is_empty());
    // NextConsensus = BFT address (m = n - (n-1)/3) of the standby validators.
    let validators = settings.standby_validators();
    let m = validators.len() - (validators.len() - 1) / 3;
    let script =
        atipicial_vm::script_builder::redeem_script::RedeemScript::multi_sig_redeem_script_from_points(
            m,
            &validators,
        )
        .unwrap();
    assert_eq!(
        *block.header.next_consensus(),
        UInt160::from_script(&script)
    );
    // Witness: empty invocation, PUSH1 verification.
    assert!(block.header.witness.invocation_script().is_empty());
    assert_eq!(
        block.header.witness.verification_script(),
        &[atipicial_vm::OpCode::PUSH1.byte()]
    );
}

#[test]
fn custom_chain_genesis_timestamp_and_nonce_are_authoritative() {
    let settings = ProtocolSettings::default();
    let mut genesis = atipicial_config::GenesisConfig::mainnet();
    genesis.timestamp = 1_700_000_000_123;
    genesis.nonce = 0x0123_4567_89ab_cdef;
    let chain_spec =
        atipicial_config::AtipicialChainSpec::private("custom-genesis-header", settings, genesis.clone(), None)
            .expect("valid custom chain spec");

    let block = genesis_block(&chain_spec).expect("custom genesis block");
    let mainnet = atipicial_config::AtipicialChainSpec::mainnet().expect("valid MainNet chain spec");
    let mainnet_hash = genesis_block(mainnet.as_ref())
        .expect("MainNet genesis block")
        .try_hash()
        .expect("MainNet genesis hash");

    assert_eq!(block.header.timestamp(), genesis.timestamp);
    assert_eq!(block.header.nonce(), genesis.nonce);
    assert_ne!(
        block.try_hash().expect("custom genesis hash"),
        mainnet_hash,
        "custom genesis identity fields must affect the canonical header hash"
    );
}

#[test]
fn genesis_persist_seeds_native_state_and_mints() {
    let resources = standard_resources();
    let settings = ProtocolSettings::default();
    let snapshot = Arc::new(DataCache::new(false));
    let block =
        Arc::new(genesis_block(&chain_spec_for_settings(&settings)).expect("genesis block"));

    let outcome = persist_with_resources(Arc::clone(&snapshot), block, &settings, &resources)
        .expect("genesis persist");

    // Genesis-active natives initialized (AtipicialCoin + OracleContract among them).
    assert!(outcome.initialized.iter().any(|n| n == "AtipicialCoin"));
    assert!(outcome.initialized.iter().any(|n| n == "OracleContract"));
    // C# allApplicationExecuted for an empty block: the OnPersist
    // engine and the PostPersist engine.
    assert_eq!(outcome.application_executed.len(), 2);
    assert_eq!(
        outcome.application_executed[0].trigger,
        atipicial_primitives::TriggerType::OnPersist
    );
    assert_eq!(
        outcome.application_executed[1].trigger,
        atipicial_primitives::TriggerType::PostPersist
    );

    // --- AtipicialCoin.Initialize seeds (byte-exact) ---
    // Committee cache: Array of Struct[pubkey, 0] in standby order.
    let expected_committee = StackItem::from_array(
        settings
            .standby_committee
            .iter()
            .map(|p| {
                StackItem::from_struct(vec![
                    StackItem::from_byte_string(p.to_bytes()),
                    StackItem::from_int(BigInt::from(0)),
                ])
            })
            .collect::<Vec<_>>(),
    );
    let expected_committee_bytes =
        BinarySerializer::serialize(&expected_committee, &ExecutionEngineLimits::default())
            .unwrap();
    assert_eq!(
        get(&snapshot, atipicial_id(), vec![ATC_PREFIX_COMMITTEE]),
        Some(expected_committee_bytes)
    );
    // Voters count: BigInteger zero = empty bytes.
    assert_eq!(
        get(&snapshot, atipicial_id(), vec![ATC_PREFIX_VOTERS_COUNT]),
        Some(Vec::new())
    );
    // ATDPerBlock record at big-endian index 0 = 5 ATD.
    let mut gpb_key = vec![ATC_PREFIX_GAS_PER_BLOCK];
    gpb_key.extend_from_slice(&0u32.to_be_bytes());
    assert_eq!(
        get(&snapshot, atipicial_id(), gpb_key),
        Some(BigInt::from(500_000_000i64).to_signed_bytes_le())
    );
    // registerPrice = 1000 ATD.
    assert_eq!(
        get(&snapshot, atipicial_id(), vec![ATC_PREFIX_REGISTER_PRICE]),
        Some(BigInt::from(100_000_000_000i64).to_signed_bytes_le())
    );

    // --- PolicyContract.Initialize seeds (byte-exact) ---
    // Policy is genesis-active (ActiveIn == null) and its Initialize writes
    // FeePerByte=1000, ExecFeeFactor=30, StoragePrice=100000 at block 0
    // (PolicyContract.cs:141-143). These MUST be committed by genesis persist,
    // otherwise getExecFeeFactor reads empty storage and returns 0 (the
    // v3.10.1 consistency testnet failure: Policy_getExecFeeFactor).
    const POLICY_PREFIX_FEE_PER_BYTE: u8 = 10;
    const POLICY_PREFIX_EXEC_FEE_FACTOR: u8 = 18;
    const POLICY_PREFIX_STORAGE_PRICE: u8 = 19;
    let policy_id = atipicial_native_contracts::PolicyContract::ID;
    assert_eq!(
        get(&snapshot, policy_id, vec![POLICY_PREFIX_FEE_PER_BYTE]),
        Some(BigInt::from(1000i64).to_signed_bytes_le()),
        "Policy FeePerByte must be initialized at genesis"
    );
    assert_eq!(
        get(&snapshot, policy_id, vec![POLICY_PREFIX_EXEC_FEE_FACTOR]),
        Some(BigInt::from(30i64).to_signed_bytes_le()),
        "Policy ExecFeeFactor must be initialized at genesis (v3.10.1 parity)"
    );
    assert_eq!(
        get(&snapshot, policy_id, vec![POLICY_PREFIX_STORAGE_PRICE]),
        Some(BigInt::from(100_000i64).to_signed_bytes_le()),
        "Policy StoragePrice must be initialized at genesis"
    );

    // --- The genesis ATC mint: 100M ATC to the standby-validator BFT address ---
    let bft = bft_address(&settings.standby_validators()).unwrap();
    let mut account_key = vec![AEP17_PREFIX_ACCOUNT];
    account_key.extend_from_slice(&bft.to_bytes());
    let expected_account = StackItem::from_struct(vec![
        StackItem::from_int(BigInt::from(100_000_000)),
        StackItem::from_int(BigInt::from(0)),
        StackItem::null(),
        StackItem::from_int(BigInt::from(0)),
    ]);
    let expected_account_bytes =
        BinarySerializer::serialize(&expected_account, &ExecutionEngineLimits::default()).unwrap();
    assert_eq!(
        get(&snapshot, atipicial_id(), account_key),
        Some(expected_account_bytes)
    );
    assert_eq!(
        get(&snapshot, atipicial_id(), vec![AEP17_PREFIX_TOTAL_SUPPLY]),
        Some(BigInt::from(100_000_000).to_signed_bytes_le())
    );
    // The mint's Transfer(null, bft, 100M) notification was emitted by ATC.
    let transfer = outcome
        .on_persist_notifications
        .iter()
        .find(|n| n.event_name == "Transfer")
        .expect("genesis ATC Transfer notification");
    assert_eq!(
        transfer.script_hash,
        atipicial_native_contracts::AtipicialCoin::script_hash(),
        "the genesis mint Transfer is emitted by the ATC contract"
    );
    assert!(
        matches!(transfer.state[0], StackItem::Null),
        "from = null (mint)"
    );
    assert_eq!(
        transfer.state[1].as_bytes().expect("to address bytes"),
        bft.to_bytes(),
        "to = the standby-validator BFT address"
    );
    assert_eq!(
        transfer.state[2].as_int().expect("amount"),
        BigInt::from(100_000_000),
        "amount = the full ATC TotalAmount"
    );
    let first_on_persist_notification = outcome
        .on_persist_notifications
        .first()
        .expect("genesis OnPersist emits native deploy notifications");
    assert_eq!(
        first_on_persist_notification.event_name, "Deploy",
        "C# ContractManagement.OnPersist deploys genesis natives before ATC/ATD initialize Transfer events"
    );
    assert_eq!(
        first_on_persist_notification.script_hash,
        atipicial_native_contracts::ContractManagement::script_hash(),
        "Deploy is emitted by ContractManagement"
    );
    assert_eq!(
        first_on_persist_notification.state[0]
            .as_bytes()
            .expect("deployed native hash"),
        atipicial_native_contracts::ContractManagement::script_hash().to_bytes(),
        "the first genesis deployment is ContractManagement itself"
    );
    // No CommitteeChanged at genesis: the recomputed committee equals the
    // seeded standby committee.
    assert!(
        !outcome
            .on_persist_notifications
            .iter()
            .any(|n| n.event_name == "CommitteeChanged"),
        "genesis recompute must not change the committee"
    );

    // --- OracleContract.Initialize seeds ---
    let oracle_id = atipicial_native_contracts::OracleContract::ID;
    assert_eq!(
        get(&snapshot, oracle_id, vec![ORACLE_PREFIX_REQUEST_ID]),
        Some(Vec::new()),
        "RequestId seeds as BigInteger.Zero (empty bytes)"
    );
    assert_eq!(
        get(&snapshot, oracle_id, vec![ORACLE_PREFIX_PRICE]),
        Some(BigInt::from(50_000_000i64).to_signed_bytes_le()),
        "oracle price seeds as 0.5 ATD"
    );

    // --- AtipicialCoin.PostPersist: committee reward minted at genesis ---
    // gasPerBlock(5 ATD) * CommitteeRewardRatio(10) / 100 = 0.5 ATD to the
    // signature address of committee[0 % m] = standby_committee[0].
    let member = &settings.standby_committee[0];
    let script = atipicial_vm::script_builder::redeem_script::RedeemScript::signature_redeem_script(
        &member.to_bytes(),
    );
    let reward_account = UInt160::from_script(&script);
    let mut gas_key = vec![AEP17_PREFIX_ACCOUNT];
    gas_key.extend_from_slice(&reward_account.to_bytes());
    let gas_account = get(&snapshot, atipicial_native_contracts::AtipicialDollar::ID, gas_key)
        .expect("committee reward ATD account");
    let decoded =
        BinarySerializer::deserialize(&gas_account, &ExecutionEngineLimits::default(), None)
            .unwrap();
    let StackItem::Struct(fields) = decoded else {
        panic!("ATD account is not a struct");
    };
    assert_eq!(
        fields.items().first().unwrap().as_int().unwrap(),
        BigInt::from(50_000_000i64),
        "committee member 0 earns 0.5 GAS at genesis"
    );
    let gas_transfer_minted = outcome
        .post_persist_notifications
        .iter()
        .any(|n| n.event_name == "Transfer");
    assert!(gas_transfer_minted, "PostPersist ATD mint emits Transfer");
}

#[test]
fn non_refresh_block_mints_to_rotating_member_without_recompute() {
    let resources = standard_resources();
    let settings = ProtocolSettings::default();
    let snapshot = Arc::new(DataCache::new(false));
    // Persist genesis first so the committee cache + gas records exist.
    let genesis =
        Arc::new(genesis_block(&chain_spec_for_settings(&settings)).expect("genesis block"));
    persist_with_resources(Arc::clone(&snapshot), genesis, &settings, &resources)
        .expect("genesis persist");

    // Block 1: not a refresh block for the 21-member committee.
    let mut header = Header::new();
    header.set_index(1);
    let block = Arc::new(Block::from_parts(header, Vec::new()));
    let outcome = persist_with_resources(Arc::clone(&snapshot), block, &settings, &resources)
        .expect("block 1 persist");
    assert!(
        outcome.initialized.is_empty(),
        "no native initializes after genesis"
    );

    // committee[1 % 21] = standby_committee[1] earns 0.5 GAS.
    let member = &settings.standby_committee[1];
    let script = atipicial_vm::script_builder::redeem_script::RedeemScript::signature_redeem_script(
        &member.to_bytes(),
    );
    let reward_account = UInt160::from_script(&script);
    let mut gas_key = vec![AEP17_PREFIX_ACCOUNT];
    gas_key.extend_from_slice(&reward_account.to_bytes());
    let gas_account = get(&snapshot, atipicial_native_contracts::AtipicialDollar::ID, gas_key)
        .expect("committee reward ATD account for member 1");
    let decoded =
        BinarySerializer::deserialize(&gas_account, &ExecutionEngineLimits::default(), None)
            .unwrap();
    let StackItem::Struct(fields) = decoded else {
        panic!("ATD account is not a struct");
    };
    assert_eq!(
        fields.items().first().unwrap().as_int().unwrap(),
        BigInt::from(50_000_000i64)
    );
}

#[test]
fn probe_constants_pin_the_real_native_ids() {
    // The probe hardcodes protocol constants because the blockchain
    // crate only reaches natives through the type-erased provider;
    // pin them against the canonical definitions.
    assert_eq!(LEDGER_CONTRACT_ID, atipicial_native_contracts::LedgerContract::ID);
    assert_eq!(ATC_TOKEN_ID, atipicial_native_contracts::AtipicialCoin::ID);
    assert_eq!(ATC_PREFIX_COMMITTEE_KEY, ATC_PREFIX_COMMITTEE);
}

#[test]
fn chain_state_initialized_flips_after_genesis_persist() {
    let resources = standard_resources();
    let settings = ProtocolSettings::default();
    let snapshot = Arc::new(DataCache::new(false));
    assert!(
        !chain_state_initialized(&snapshot),
        "fresh store is uninitialized"
    );

    let block =
        Arc::new(genesis_block(&chain_spec_for_settings(&settings)).expect("genesis block"));
    persist_with_resources(Arc::clone(&snapshot), block, &settings, &resources)
        .expect("genesis persist");
    assert!(
        chain_state_initialized(&snapshot),
        "genesis persist initializes the chain"
    );

    // The C#-faithful leg of the probe: a LedgerContract Prefix_Block
    // record alone also reports initialized.
    let ledger_only = DataCache::new(false);
    let mut key = vec![LEDGER_PREFIX_BLOCK];
    key.extend_from_slice(&[0u8; 32]);
    ledger_only.add(
        StorageKey::new(LEDGER_CONTRACT_ID, key),
        atipicial_storage::StorageItem::from_bytes(vec![1]),
    );
    assert!(chain_state_initialized(&ledger_only));
}

#[test]
fn chain_state_initialized_uses_the_constant_time_committee_probe_first() {
    #[derive(Clone)]
    struct CommitteeBacking {
        prefix_scans: Arc<AtomicUsize>,
    }

    impl atipicial_storage::CacheRead for CommitteeBacking {
        fn get(&self, key: &StorageKey) -> Option<atipicial_storage::StorageItem> {
            (key == &StorageKey::new(ATC_TOKEN_ID, vec![ATC_PREFIX_COMMITTEE_KEY]))
                .then(|| atipicial_storage::StorageItem::from_bytes(vec![1]))
        }

        fn find(
            &self,
            _prefix: Option<&StorageKey>,
            _direction: atipicial_storage::persistence::SeekDirection,
        ) -> Option<Vec<(StorageKey, atipicial_storage::StorageItem)>> {
            self.prefix_scans.fetch_add(1, Ordering::Relaxed);
            Some(Vec::new())
        }
    }

    let prefix_scans = Arc::new(AtomicUsize::new(0));
    let snapshot = DataCache::with_backing(
        false,
        CommitteeBacking {
            prefix_scans: Arc::clone(&prefix_scans),
        },
        atipicial_storage::persistence::data_cache::DataCacheConfig::default(),
    );

    assert!(chain_state_initialized(&snapshot));
    assert_eq!(
        prefix_scans.load(Ordering::Relaxed),
        0,
        "an initialized chain must not materialize the Ledger block prefix"
    );
}

/// Mainnet genesis-hash pin. Oracle:
/// `atipicial_csharp/tests/Atipicial.UnitTests/SmartContract/UT_InteropService.cs:872`
/// (`TestGetBlockHash`) asserts block 0's hash under
/// `TestProtocolSettings.Default`, whose `StandbyCommittee` /
/// `ValidatorsCount` are byte-identical to
/// `atipicial_csharp/src/Atipicial.CLI/config.mainnet.json` (verified 2026-06-10).
/// The header hash covers only the serialized unsigned header
/// (`Atipicial.Network.P2P.Helper.CalculateHash` — single SHA-256, no
/// network magic), so the test-chain genesis hash IS the mainnet
/// genesis hash. This transitively pins `NextConsensus`, the
/// standby-validator multisig redeem script, and hash160.
#[test]
fn mainnet_genesis_hash_matches_csharp() {
    let chain_spec = atipicial_config::AtipicialChainSpec::mainnet().expect("valid MainNet chain spec");
    let block = genesis_block(chain_spec.as_ref()).expect("genesis block");
    let hash = block.header.try_hash().expect("genesis hash");
    assert_eq!(
        hash.to_string(),
        "0x1f4d1defa46faa5e7b9b8d3f79a06bec777d7c26c4aa5f6f5899a291daa87c15",
        "mainnet genesis hash must match the C# oracle \
         (UT_InteropService.TestGetBlockHash)"
    );
}

/// The transaction stage of `Blockchain.Persist`: a HALTing and a
/// FAULTing transaction in one block both execute and get ledger
/// records carrying their final VM state, and the
/// `ApplicationExecuted` list has the C# shape (OnPersist, one per
/// tx, PostPersist).
#[test]
fn persist_executes_transactions_and_records_vm_states() {
    let resources = standard_resources();
    let settings = ProtocolSettings::default();
    let snapshot = Arc::new(DataCache::new(false));
    let genesis =
        Arc::new(genesis_block(&chain_spec_for_settings(&settings)).expect("genesis block"));
    persist_with_resources(Arc::clone(&snapshot), genesis, &settings, &resources)
        .expect("genesis persist");

    // Fund the fee-paying signer first: C# AtipicialDollar.OnPersist burns
    // each transaction's system+network fee from its sender, so a
    // block whose sender holds no GAS faults the OnPersist engine.
    let signer_account = atipicial_primitives::UInt160::from_bytes(&[0x33; 20]).unwrap();
    let mut gas_key = vec![AEP17_PREFIX_ACCOUNT];
    gas_key.extend_from_slice(&signer_account.to_bytes());
    let account_state =
        StackItem::from_struct(vec![StackItem::from_int(BigInt::from(10_0000_0000i64))]);
    let account_bytes =
        BinarySerializer::serialize(&account_state, &ExecutionEngineLimits::default()).unwrap();
    snapshot.add(
        StorageKey::new(atipicial_native_contracts::AtipicialDollar::ID, gas_key),
        atipicial_storage::StorageItem::from_bytes(account_bytes),
    );

    // tx1 faults (ABORT), tx2 halts (PUSH1).
    let signer = atipicial_payloads::Signer::new(signer_account, atipicial_primitives::WitnessScope::NONE);
    let mut tx1 = atipicial_payloads::Transaction::new();
    tx1.set_nonce(1);
    tx1.set_script(vec![atipicial_vm::OpCode::ABORT.byte()]);
    tx1.set_system_fee(1_0000_0000);
    tx1.set_signers(vec![signer.clone()]);
    tx1.set_witnesses(vec![atipicial_payloads::Witness::empty()]);
    let mut tx2 = atipicial_payloads::Transaction::new();
    tx2.set_nonce(2);
    tx2.set_script(vec![atipicial_vm::OpCode::PUSH1.byte()]);
    tx2.set_system_fee(1_0000_0000);
    tx2.set_signers(vec![signer]);
    tx2.set_witnesses(vec![atipicial_payloads::Witness::empty()]);
    let tx1_hash = tx1.try_hash().unwrap();
    let tx2_hash = tx2.try_hash().unwrap();

    let mut header = Header::new();
    header.set_index(1);
    let block = Arc::new(Block::from_parts(header, vec![tx1, tx2]));
    let block_hash = block.header.try_hash().unwrap();
    let outcome = persist_with_resources(Arc::clone(&snapshot), block, &settings, &resources)
        .expect("block 1 persist");

    // C# allApplicationExecuted: OnPersist, tx1, tx2, PostPersist.
    assert_eq!(outcome.application_executed.len(), 4);
    let tx1_exec = &outcome.application_executed[1];
    assert_eq!(tx1_exec.trigger, atipicial_primitives::TriggerType::Application);
    assert_eq!(tx1_exec.vm_state, atipicial_vm::VmState::FAULT);
    assert!(tx1_exec.transaction.is_some());
    let tx2_exec = &outcome.application_executed[2];
    assert_eq!(tx2_exec.vm_state, atipicial_vm::VmState::HALT);
    // PUSH1 leaves the integer 1 on the result stack.
    assert_eq!(tx2_exec.stack.len(), 1);
    let actual = tx2_exec.stack[0]
        .as_int()
        .expect("expected integer stack item");
    assert_eq!(actual, BigInt::from(1));

    // Ledger records carry the final VM states (C# mutates the
    // TransactionState stored by Ledger.OnPersist) and the block
    // records exist.
    let ledger = atipicial_native_contracts::LedgerContract::new();
    let s1 = ledger
        .get_transaction_state(&snapshot, &tx1_hash)
        .unwrap()
        .expect("tx1 record");
    assert_eq!(s1.state, atipicial_vm::VmState::FAULT);
    assert_eq!(s1.block_index, 1);
    let s2 = ledger
        .get_transaction_state(&snapshot, &tx2_hash)
        .unwrap()
        .expect("tx2 record");
    assert_eq!(s2.state, atipicial_vm::VmState::HALT);
    assert_eq!(
        ledger.get_block_hash(&snapshot, 1).unwrap(),
        Some(block_hash)
    );
    let trimmed = ledger
        .get_trimmed_block(&snapshot, &block_hash)
        .unwrap()
        .expect("trimmed block");
    assert_eq!(trimmed.hashes, vec![tx1_hash, tx2_hash]);
    // PostPersist current-block pointer.
    assert_eq!(ledger.current_index(&snapshot).unwrap(), 1);
    assert_eq!(ledger.current_hash(&snapshot).unwrap(), block_hash);
}

#[test]
fn persist_records_fault_when_aep17_receiver_callback_faults() {
    let resources = standard_resources();
    let settings = ProtocolSettings::default();
    let snapshot = Arc::new(DataCache::new(false));
    let genesis =
        Arc::new(genesis_block(&chain_spec_for_settings(&settings)).expect("genesis block"));
    persist_with_resources(Arc::clone(&snapshot), genesis, &settings, &resources)
        .expect("genesis persist");

    let signer_account = atipicial_primitives::UInt160::from_bytes(&[0x55; 20]).unwrap();
    let receiver = atipicial_primitives::UInt160::from_bytes(&[0x66; 20]).unwrap();
    fund_gas(&snapshot, &signer_account, 10_0000_0000);
    deploy_contract(&snapshot, &throwing_aep17_receiver_contract(receiver));

    let script = gas_transfer_script(&signer_account, &receiver, 10_0000);
    let mut tx = signed_test_tx(signer_account, 55, script);
    tx.set_signers(vec![atipicial_payloads::Signer::new(
        signer_account,
        atipicial_primitives::WitnessScope::GLOBAL,
    )]);
    let tx_hash = tx.try_hash().unwrap();

    let mut header = Header::new();
    header.set_index(1);
    let block = Arc::new(Block::from_parts(header, vec![tx]));
    let outcome = persist_with_resources(Arc::clone(&snapshot), block, &settings, &resources)
        .expect("block 1 persist");

    assert_eq!(outcome.application_executed.len(), 3);
    let tx_exec = &outcome.application_executed[1];
    assert_eq!(tx_exec.trigger, atipicial_primitives::TriggerType::Application);
    assert_eq!(
        tx_exec.vm_state,
        atipicial_vm::VmState::FAULT,
        "a receiver onAEP17Payment exception must fault the transaction"
    );

    let ledger = atipicial_native_contracts::LedgerContract::new();
    let state = ledger
        .get_transaction_state(&snapshot, &tx_hash)
        .unwrap()
        .expect("tx record");
    assert_eq!(
        state.state,
        atipicial_vm::VmState::FAULT,
        "persisted ledger TransactionState must carry the callback-induced fault"
    );
}

#[test]
fn bulk_sync_native_persist_skips_replay_artifacts_but_keeps_vm_state() {
    let resources = standard_resources();
    let settings = ProtocolSettings::default();
    let snapshot = Arc::new(DataCache::new(false));
    let genesis =
        Arc::new(genesis_block(&chain_spec_for_settings(&settings)).expect("genesis block"));
    persist_with_resources(Arc::clone(&snapshot), genesis, &settings, &resources)
        .expect("genesis persist");

    let signer_account = atipicial_primitives::UInt160::from_bytes(&[0x44; 20]).unwrap();
    let mut gas_key = vec![AEP17_PREFIX_ACCOUNT];
    gas_key.extend_from_slice(&signer_account.to_bytes());
    let account_state =
        StackItem::from_struct(vec![StackItem::from_int(BigInt::from(10_0000_0000i64))]);
    let account_bytes =
        BinarySerializer::serialize(&account_state, &ExecutionEngineLimits::default()).unwrap();
    snapshot.add(
        StorageKey::new(atipicial_native_contracts::AtipicialDollar::ID, gas_key),
        atipicial_storage::StorageItem::from_bytes(account_bytes),
    );

    let mut tx = atipicial_payloads::Transaction::new();
    tx.set_nonce(44);
    tx.set_script(vec![atipicial_vm::OpCode::PUSH1.byte()]);
    tx.set_system_fee(1_0000_0000);
    tx.set_signers(vec![atipicial_payloads::Signer::new(
        signer_account,
        atipicial_primitives::WitnessScope::NONE,
    )]);
    tx.set_witnesses(vec![atipicial_payloads::Witness::empty()]);
    let tx_hash = tx.try_hash().unwrap();

    let mut header = Header::new();
    header.set_index(1);
    let block = Arc::new(Block::from_parts(header, vec![tx]));
    let staged = stage_block_natives_with_resources(
        Arc::clone(&snapshot),
        block,
        Arc::new(settings.clone()),
        NativePersistOptions {
            capture_replay_artifacts: false,
        },
        &resources,
    )
    .expect("bulk-sync block stages");

    assert!(
        staged.outcome.application_executed.is_empty(),
        "bulk sync should not materialize ApplicationExecuted replay payloads"
    );
    assert!(
        staged.outcome.on_persist_notifications.is_empty(),
        "bulk sync should not clone OnPersist notification artifacts"
    );
    assert!(
        staged.outcome.post_persist_notifications.is_empty(),
        "bulk sync should not clone PostPersist notification artifacts"
    );
    staged.commit();

    let state = atipicial_native_contracts::LedgerContract::new()
        .get_transaction_state(&snapshot, &tx_hash)
        .unwrap()
        .expect("tx ledger state remains consensus-visible");
    assert_eq!(state.state, atipicial_vm::VmState::HALT);
    assert_eq!(state.block_index, 1);
}

#[test]
fn reusable_native_persist_resources_fetch_contract_list_once_for_batch() {
    let all_contracts_calls = Arc::new(AtomicUsize::new(0));
    let provider = Arc::new(CountingNativeProvider::new(Arc::clone(
        &all_contracts_calls,
    )));

    let settings = ProtocolSettings::default();
    let snapshot = Arc::new(DataCache::new(false));
    let resources = NativePersistResources::from_provider(provider);
    assert_eq!(
        all_contracts_calls.load(Ordering::Relaxed),
        1,
        "resource construction should fetch the canonical contract list once"
    );

    let genesis =
        Arc::new(genesis_block(&chain_spec_for_settings(&settings)).expect("genesis block"));
    let staged = stage_block_natives_with_resources(
        Arc::clone(&snapshot),
        genesis,
        Arc::new(settings.clone()),
        NativePersistOptions {
            capture_replay_artifacts: false,
        },
        &resources,
    )
    .expect("genesis stages with cached resources");
    staged.commit();

    let mut header = Header::new();
    header.set_index(1);
    let block = Arc::new(Block::from_parts(header, Vec::new()));
    let staged = stage_block_natives_with_resources(
        Arc::clone(&snapshot),
        block,
        Arc::new(settings.clone()),
        NativePersistOptions {
            capture_replay_artifacts: false,
        },
        &resources,
    )
    .expect("second block stages with cached resources");
    staged.commit();

    assert_eq!(
        all_contracts_calls.load(Ordering::Relaxed),
        1,
        "bulk persistence should reuse the cached native-contract list"
    );
}

#[test]
fn reusable_native_persist_resources_keep_provider_consistent_across_blocks() {
    let all_contracts_calls = Arc::new(AtomicUsize::new(0));
    let provider = Arc::new(CountingNativeProvider::new(Arc::clone(
        &all_contracts_calls,
    )));

    let settings = ProtocolSettings::default();
    let snapshot = Arc::new(DataCache::new(false));
    let resources = NativePersistResources::from_provider(provider);

    let genesis =
        Arc::new(genesis_block(&chain_spec_for_settings(&settings)).expect("genesis block"));
    let staged = stage_block_natives_with_resources(
        Arc::clone(&snapshot),
        genesis,
        Arc::new(settings.clone()),
        NativePersistOptions {
            capture_replay_artifacts: false,
        },
        &resources,
    )
    .expect("genesis stages with original provider");
    staged.commit();

    let signer_account = UInt160::from_bytes(&[0x55; 20]).unwrap();
    fund_gas(&snapshot, &signer_account, 10_0000_0000);
    let tx = signed_test_tx(signer_account, 55, policy_get_exec_fee_factor_script());

    let mut header = Header::new();
    header.set_index(1);
    let block = Arc::new(Block::from_parts(header, vec![tx]));
    let staged = stage_block_natives_with_resources(
        Arc::clone(&snapshot),
        block,
        Arc::new(settings.clone()),
        NativePersistOptions::default(),
        &resources,
    )
    .expect("transaction native lookup should use the batch resource provider");
    let tx_exec = staged
        .outcome
        .application_executed
        .iter()
        .find(|executed| executed.trigger == atipicial_primitives::TriggerType::Application)
        .expect("transaction application execution");
    assert_eq!(tx_exec.vm_state, atipicial_vm::VmState::HALT);
    assert_eq!(
        tx_exec.stack,
        vec![StackItem::from_i64(30)],
        "Policy.getExecFeeFactor should resolve through the batch resource provider"
    );
}

#[test]
fn specialization_shadow_pipeline_preserves_ordinary_transaction_and_ledger_result() {
    let settings = ProtocolSettings::default();
    let normal_snapshot = Arc::new(DataCache::new(false));
    let shadow_snapshot = Arc::new(DataCache::new(false));
    let normal_resources = standard_resources();
    let shadow_resources = shadow_resources(false, ExecutionArtifactLimits::DEFAULT);

    for (snapshot, resources) in [
        (&normal_snapshot, &normal_resources),
        (&shadow_snapshot, &shadow_resources),
    ] {
        let genesis =
            Arc::new(genesis_block(&chain_spec_for_settings(&settings)).expect("genesis block"));
        persist_with_resources(Arc::clone(snapshot), genesis, &settings, resources)
            .expect("genesis persist");
    }

    let signer = UInt160::from_bytes(&[0x55; 20]).expect("test signer");
    fund_gas(&normal_snapshot, &signer, 10_0000_0000);
    fund_gas(&shadow_snapshot, &signer, 10_0000_0000);
    let tx = signed_test_tx(signer, 77, policy_get_exec_fee_factor_script());
    let tx_hash = tx.try_hash().expect("transaction hash");
    let mut header = Header::new();
    header.set_index(1);
    let block = Block::from_parts(header, vec![tx]);

    let normal = persist_with_resources(
        Arc::clone(&normal_snapshot),
        Arc::new(block.clone()),
        &settings,
        &normal_resources,
    )
    .expect("ordinary transaction persist");
    let shadow = persist_with_resources(
        Arc::clone(&shadow_snapshot),
        Arc::new(block),
        &settings,
        &shadow_resources,
    )
    .expect("shadow transaction persist");

    let normal_tx = &normal.application_executed[1];
    let shadow_tx = &shadow.application_executed[1];
    assert_eq!(shadow_tx.vm_state, normal_tx.vm_state);
    assert_eq!(shadow_tx.exception, normal_tx.exception);
    assert_eq!(shadow_tx.gas_consumed, normal_tx.gas_consumed);
    assert_eq!(shadow_tx.stack, normal_tx.stack);
    assert_eq!(shadow_tx.notifications.len(), normal_tx.notifications.len());
    assert_eq!(shadow_tx.logs.len(), normal_tx.logs.len());

    let ledger = atipicial_native_contracts::LedgerContract::new();
    let normal_state = ledger
        .get_transaction_state(&normal_snapshot, &tx_hash)
        .expect("normal ledger read")
        .expect("normal transaction record");
    let shadow_state = ledger
        .get_transaction_state(&shadow_snapshot, &tx_hash)
        .expect("shadow ledger read")
        .expect("shadow transaction record");
    assert_eq!(shadow_state.state, normal_state.state);
    assert_eq!(shadow_state.block_index, normal_state.block_index);
}

#[test]
fn strict_specialization_shadow_artifact_failure_aborts_block_publication() {
    let settings = ProtocolSettings::default();
    let snapshot = Arc::new(DataCache::new(false));
    let normal_resources = standard_resources();
    let genesis =
        Arc::new(genesis_block(&chain_spec_for_settings(&settings)).expect("genesis block"));
    persist_with_resources(Arc::clone(&snapshot), genesis, &settings, &normal_resources)
        .expect("genesis persist");

    let signer = UInt160::from_bytes(&[0x66; 20]).expect("test signer");
    fund_gas(&snapshot, &signer, 10_0000_0000);
    let tx = signed_test_tx(signer, 78, policy_get_exec_fee_factor_script());
    let tx_hash = tx.try_hash().expect("transaction hash");
    let mut header = Header::new();
    header.set_index(1);
    let block = Arc::new(Block::from_parts(header, vec![tx]));
    let resources = shadow_resources(
        true,
        ExecutionArtifactLimits {
            max_invocation_frames: 0,
            ..ExecutionArtifactLimits::DEFAULT
        },
    );

    let error = persist_with_resources(Arc::clone(&snapshot), block, &settings, &resources)
        .expect_err("strict incomplete artifact must fail replay");
    assert!(
        error
            .to_string()
            .contains("strict specialization shadow failed")
    );
    assert!(
        atipicial_native_contracts::LedgerContract::new()
            .get_transaction_state(&snapshot, &tx_hash)
            .expect("ledger read after aborted block")
            .is_none(),
        "failed shadow replay must not publish the transaction record"
    );
}

#[test]
fn native_persist_resources_do_not_install_thread_scoped_provider() {
    let source = include_str!("../../pipeline/native_persist.rs");
    let resources = include_str!("../../pipeline/native_persist/types.rs");
    assert!(
        !source.contains("with_scoped_provider"),
        "native persistence resources must pass providers directly into engines, not mutate the thread-scoped global provider"
    );
    assert!(
        resources.contains("pub struct NativePersistResources<P>"),
        "native persistence resources should preserve the captured provider type"
    );
    assert!(
        resources.contains("provider: Arc<P>"),
        "native persistence resources should store Arc<P>, not erase the provider internally"
    );
}

#[test]
fn native_persist_exposes_explicit_resource_commit_path() {
    let source = include_str!("../../pipeline/native_persist.rs");
    assert!(
        source.contains("pub fn persist_block_natives_with_resources"),
        "native persistence should expose a committing path for callers that already own explicit provider resources"
    );
    assert!(
        source.contains("pub fn stage_block_natives_with_resources"),
        "native persistence should expose a staging path for callers that already own explicit provider resources"
    );
    assert!(
        !source.contains("from_installed_provider"),
        "native persistence must not expose installed-provider compatibility constructors"
    );
    assert!(
        !source.contains("NativeContractLookup::native_contract_provider"),
        "native persistence must not read the process-global provider"
    );
}

#[test]
fn reusable_native_persist_resources_cross_echidna_activation_height() {
    let resources = standard_resources();
    let mut settings = ProtocolSettings::default();
    settings.hardforks = atipicial_config::HardforkSchedule::new()
        .with_activation(atipicial_config::Hardfork::HfEchidna, 1)
        .with_omitted_leading_at_genesis();
    let snapshot = Arc::new(DataCache::new(false));

    let genesis =
        Arc::new(genesis_block(&chain_spec_for_settings(&settings)).expect("genesis block"));
    let staged = stage_block_natives_with_resources(
        Arc::clone(&snapshot),
        genesis,
        Arc::new(settings.clone()),
        NativePersistOptions {
            capture_replay_artifacts: false,
        },
        &resources,
    )
    .expect("genesis stages before Echidna");
    staged.commit();
    assert!(
        atipicial_native_contracts::ContractManagement::get_contract_from_snapshot(
            &snapshot,
            &atipicial_native_contracts::Notary::script_hash(),
        )
        .expect("notary lookup before Echidna")
        .is_none(),
        "Notary must not deploy before its configured Echidna activation block"
    );

    let mut header = Header::new();
    header.set_index(1);
    let block = Arc::new(Block::from_parts(header, Vec::new()));
    let staged = stage_block_natives_with_resources(
        Arc::clone(&snapshot),
        block,
        Arc::new(settings.clone()),
        NativePersistOptions {
            capture_replay_artifacts: false,
        },
        &resources,
    )
    .expect("Echidna block stages with reused resources");
    assert!(
        staged
            .outcome
            .initialized
            .iter()
            .any(|name| name == "Notary"),
        "reused resources must still recompute hardfork activation per block"
    );
    staged.commit();

    let notary = atipicial_native_contracts::ContractManagement::get_contract_from_snapshot(
        &snapshot,
        &atipicial_native_contracts::Notary::script_hash(),
    )
    .expect("notary lookup after Echidna")
    .expect("Notary deploys at Echidna");
    assert_eq!(notary.update_counter, 0);
}

/// Genesis persist now writes the C#-faithful Ledger records: the
/// `Prefix_Block` probe of [`chain_state_initialized`] (the literal
/// C# `Ledger.Initialized` check) and the current-block pointer.
#[test]
fn genesis_persist_writes_ledger_records() {
    let resources = standard_resources();
    let settings = ProtocolSettings::default();
    let snapshot = Arc::new(DataCache::new(false));
    let genesis =
        Arc::new(genesis_block(&chain_spec_for_settings(&settings)).expect("genesis block"));
    let genesis_hash = genesis.header.try_hash().unwrap();
    persist_with_resources(Arc::clone(&snapshot), genesis, &settings, &resources)
        .expect("genesis persist");

    let ledger = atipicial_native_contracts::LedgerContract::new();
    assert_eq!(
        ledger.get_block_hash(&snapshot, 0).unwrap(),
        Some(genesis_hash)
    );
    assert_eq!(ledger.current_index(&snapshot).unwrap(), 0);
    assert_eq!(ledger.current_hash(&snapshot).unwrap(), genesis_hash);
    let block_prefix = StorageKey::new(LEDGER_CONTRACT_ID, vec![LEDGER_PREFIX_BLOCK]);
    assert!(
        snapshot
            .find(
                Some(&block_prefix),
                atipicial_storage::persistence::SeekDirection::Forward
            )
            .next()
            .is_some(),
        "the C# Ledger.Initialized probe (any Prefix_Block record) must hit"
    );
}

/// The staging contract the per-block atomicity rests on: writes
/// into a `clone_cache()` child are invisible to the parent until
/// `commit()`, and dropping the child discards them. The persist
/// pipeline stages every block write in such a child, so a
/// mid-sequence error can never leave partial block state in the
/// caller's snapshot.
#[test]
fn block_staging_cache_isolates_until_commit() {
    let parent = DataCache::new(false);
    let key = StorageKey::new(-4, vec![5, 0xAA]);

    // Discard leg: child writes never reach the parent.
    {
        let child = parent.clone_cache();
        child.add(key.clone(), atipicial_storage::StorageItem::from_bytes(vec![1]));
        assert!(child.get(&key).is_some());
        assert!(parent.get(&key).is_none(), "uncommitted child write leaked");
    }
    assert!(parent.get(&key).is_none(), "dropped child write leaked");

    // Commit leg: the child write lands atomically on commit.
    let child = parent.clone_cache();
    child.add(key.clone(), atipicial_storage::StorageItem::from_bytes(vec![2]));
    assert!(parent.get(&key).is_none());
    child.commit();
    assert_eq!(
        parent.get(&key).map(|i| i.value_bytes().into_owned()),
        Some(vec![2])
    );
}
