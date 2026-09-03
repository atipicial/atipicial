use super::*;
use crate::server::rpc_error::RpcError;
use crate::server::rpc_server::{RpcHandler, RpcServer};
use crate::server::rpc_server_settings::RpcServerConfig;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use atipicial_config::ProtocolSettings;
use atipicial_io::SerializableExtensions;
use atipicial_native_contracts::LedgerContract;
use atipicial_native_contracts::PolicyContract;
use atipicial_payloads::OracleResponseCode;
use atipicial_payloads::oracle_response::{MAX_RESULT_SIZE, OracleResponse};
use atipicial_payloads::signer::Signer;
use atipicial_payloads::transaction::Transaction;
use atipicial_payloads::witness::Witness;
use atipicial_payloads::{Block, Header, TransactionAttribute, get_sign_data_vec};
use atipicial_primitives::VerifyResult;
use atipicial_primitives::{UInt160, UInt256, WitnessScope};
use atipicial_serialization::json::JToken;
use atipicial_storage::persistence::StoreCache;
use atipicial_storage::{StorageItem, StorageKey};
use atipicial_vm::VmState as VMState;
use atipicial_vm::{Contract, OpCode};
use atipicial_wallets::KeyPair;
use num_bigint::BigInt;
use serde_json::{Value, json};

fn find_handler<'a>(handlers: &'a [RpcHandler], name: &str) -> &'a RpcHandler {
    handlers
        .iter()
        .find(|handler| handler.descriptor().name.eq_ignore_ascii_case(name))
        .expect("handler present")
}

fn parse_object(value: &Value) -> atipicial_serialization::json::JObject {
    let json = serde_json::to_string(value).expect("serialize");
    let token = JToken::parse(&json, 128).expect("parse");
    token.as_object().cloned().expect("expected JSON object")
}

fn build_signed_transaction_custom(
    settings: &ProtocolSettings,
    keypair: &KeyPair,
    nonce: u32,
    system_fee: i64,
    network_fee: i64,
    script: Vec<u8>,
) -> Transaction {
    let mut tx = Transaction::new();
    tx.set_nonce(nonce);
    tx.set_network_fee(network_fee);
    tx.set_system_fee(system_fee);
    tx.set_valid_until_block(1);
    tx.set_script(script);
    tx.set_signers(vec![Signer::new(
        keypair.script_hash(),
        WitnessScope::GLOBAL,
    )]);

    let sign_data = get_sign_data_vec(&tx, settings.network).expect("sign data");
    let signature = keypair.sign(&sign_data).expect("sign");
    let mut invocation = Vec::with_capacity(signature.len() + 2);
    invocation.push(OpCode::PUSHDATA1.byte());
    invocation.push(signature.len() as u8);
    invocation.extend_from_slice(&signature);
    let verification_script = keypair.verification_script();
    tx.set_witnesses(vec![Witness::new_with_scripts(
        invocation,
        verification_script,
    )]);
    tx
}

fn build_signed_transaction(
    settings: &ProtocolSettings,
    keypair: &KeyPair,
    nonce: u32,
    system_fee: i64,
) -> Transaction {
    build_signed_transaction_custom(
        settings,
        keypair,
        nonce,
        system_fee,
        1_0000_0000,
        vec![OpCode::PUSH1.byte()],
    )
}

#[allow(clippy::too_many_arguments)]
fn build_signed_transaction_with(
    settings: &ProtocolSettings,
    keypair: &KeyPair,
    nonce: u32,
    system_fee: i64,
    network_fee: i64,
    valid_until_block: u32,
    script: Vec<u8>,
    attributes: Vec<TransactionAttribute>,
) -> Transaction {
    let mut tx = Transaction::new();
    tx.set_nonce(nonce);
    tx.set_network_fee(network_fee);
    tx.set_system_fee(system_fee);
    tx.set_valid_until_block(valid_until_block);
    tx.set_script(script);
    tx.set_signers(vec![Signer::new(
        keypair.script_hash(),
        WitnessScope::GLOBAL,
    )]);
    tx.set_attributes(attributes);

    let sign_data = get_sign_data_vec(&tx, settings.network).expect("sign data");
    let signature = keypair.sign(&sign_data).expect("sign");
    let mut invocation = Vec::with_capacity(signature.len() + 2);
    invocation.push(OpCode::PUSHDATA1.byte());
    invocation.push(signature.len() as u8);
    invocation.extend_from_slice(&signature);
    let verification_script = keypair.verification_script();
    tx.set_witnesses(vec![Witness::new_with_scripts(
        invocation,
        verification_script,
    )]);
    tx
}

fn single_validator_settings(keypair: &KeyPair) -> ProtocolSettings {
    let mut settings = ProtocolSettings::default();
    let validator = keypair.public_key_point().expect("validator public key");
    settings.standby_committee = vec![validator];
    settings.validators_count = 1;
    settings
}

fn build_signed_block<S>(
    settings: &ProtocolSettings,
    store: &StoreCache<S>,
    validator: &KeyPair,
    transactions: Vec<Transaction>,
) -> Block
where
    S: atipicial_storage::persistence::Store,
{
    let snapshot = store.data_cache();
    let ledger = LedgerContract::new();
    let prev_hash = ledger.current_hash(snapshot).expect("current hash");
    let prev_trimmed = ledger
        .get_trimmed_block(snapshot, &prev_hash)
        .expect("prev trimmed query")
        .expect("prev trimmed block");
    let prev_index = prev_trimmed.header.index();
    let prev_timestamp = prev_trimmed.header.timestamp();

    let validators = settings.standby_validators();
    // C# Contract.GetBFTAddress: multisig contract with m = n - (n-1)/3.
    let m = validators.len() - (validators.len() - 1) / 3;
    let next_consensus = atipicial_execution::Helper::to_script_hash(&Contract::create(
        vec![],
        Contract::create_multi_sig_redeem_script(m, &validators),
    ));

    let mut header = Header::new();
    header.set_prev_hash(prev_hash);
    header.set_index(prev_index + 1);
    header.set_timestamp(prev_timestamp + settings.milliseconds_per_block as u64);
    header.set_primary_index(0);
    header.set_next_consensus(next_consensus);
    header.set_nonce(0);

    let mut block = Block::new();
    block.header = header;
    block.transactions = transactions;
    block.rebuild_merkle_root();

    let sign_data = get_sign_data_vec(&block.header, settings.network).expect("sign data");
    let signature = validator.sign(&sign_data).expect("sign header");
    let mut invocation = Vec::with_capacity(signature.len() + 2);
    invocation.push(OpCode::PUSHDATA1.byte());
    invocation.push(signature.len() as u8);
    invocation.extend_from_slice(&signature);
    let verification_script = Contract::create_multi_sig_redeem_script(1, &validators);
    block.header.witness = Witness::new_with_scripts(invocation, verification_script);

    block
}

fn mint_gas<S>(
    store: &mut atipicial_storage::persistence::StoreCache<S>,
    _settings: &ProtocolSettings,
    account: UInt160,
    amount: BigInt,
) where
    S: atipicial_storage::persistence::Store,
{
    // Seeds the byte-exact AEP-17 account-state record the native
    // `balanceOf` reads; the legacy fixture invoked `ATD.Mint` through
    // an engine, which produces the same storage record.
    crate::server::test_support::seed_gas_balance(store, &account, amount);
}

fn persist_transaction_record<S>(store: &mut StoreCache<S>, tx: &Transaction, block_index: u32)
where
    S: atipicial_storage::persistence::Store,
{
    const PREFIX_TRANSACTION: u8 = 0x0b;

    // `Prefix_Transaction` value: the C# `TransactionState` interoperable
    // stack item serialized with `BinarySerializer`, matching the reader.
    let record = atipicial_native_contracts::LedgerContract::new()
        .serialize_persisted_transaction_state(block_index, VMState::NONE, tx)
        .expect("serialize TransactionState record");

    let mut key_bytes = Vec::with_capacity(1 + 32);
    key_bytes.push(PREFIX_TRANSACTION);
    key_bytes.extend_from_slice(&tx.hash().to_bytes());
    let key = StorageKey::new(LedgerContract::ID, key_bytes);
    store.add(key, StorageItem::from_bytes(record));
    store.try_commit().expect("commit test store");
}

#[path = "../rpc_server_node/connection_count.rs"]
mod connection_count;
#[path = "../rpc_server_node/get_peers.rs"]
mod get_peers;
#[path = "../rpc_server_node/get_version.rs"]
mod get_version;
#[path = "../rpc_server_node/no_params.rs"]
mod no_params;
#[path = "../rpc_server_node/send_raw_transaction.rs"]
mod send_raw_transaction;
#[path = "../rpc_server_node/submit_block.rs"]
mod submit_block;
