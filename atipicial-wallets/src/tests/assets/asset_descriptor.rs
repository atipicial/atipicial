use super::*;
use atipicial_config::ProtocolSettings;
use atipicial_execution::Aep17MetadataReaderImpl;
use atipicial_execution::contract_state::ContractState;
use atipicial_execution::native_contract::build_native_contract_state;
use atipicial_native_contracts::{AtipicialDollar, AtipicialCoin, StandardNativeProvider};
use atipicial_storage::{DataCache, StorageItem, StorageKey};

/// `ContractManagement.PREFIX_CONTRACT` — the per-contract storage prefix
/// (verified against `atipicial-native-contracts/src/contract_management.rs`).
const PREFIX_CONTRACT: u8 = 8;

/// Inserts a deployed `ContractState` for `state.hash` into `cache` under the
/// ContractManagement record key (the C# interoperable stack-item record),
/// mirroring a post-genesis snapshot so `get_contract_from_snapshot` can
/// resolve it.
fn deploy_contract_record(cache: &DataCache, state: &ContractState) {
    let record = state
        .serialize_contract_record()
        .expect("serialize contract record");

    let mut key = Vec::with_capacity(1 + 20);
    key.push(PREFIX_CONTRACT);
    key.extend_from_slice(&state.hash.to_bytes());

    cache.add(
        StorageKey::new(ContractManagement::ID, key),
        StorageItem::from_bytes(record),
    );
}

fn standard_native_provider() -> Arc<StandardNativeProvider> {
    Arc::new(StandardNativeProvider::new())
}

#[test]
fn nonexistent_asset_id_is_rejected() {
    // C# `TestConstructorWithNonexistAssetId`: an undeployed asset id throws
    // ArgumentException; here it maps to `invalid_argument`.
    let snapshot = Arc::new(DataCache::new(false));
    let settings = ProtocolSettings::default();
    let bogus = UInt160::from_bytes(&[0xAB; 20]).unwrap();
    let reader = Aep17MetadataReaderImpl::new_with_native_contract_provider(
        Arc::clone(&snapshot),
        settings,
        standard_native_provider(),
    );

    let err = AssetDescriptor::new(snapshot, &reader, bogus)
        .expect_err("undeployed asset must be rejected");
    assert!(
        err.to_string().contains("No asset contract found"),
        "unexpected error: {err}"
    );
}

#[test]
fn descriptor_reads_gas_metadata() {
    // C# `Check_GAS`: against a snapshot where GAS is deployed, the descriptor
    // exposes name=AtipicialDollar, symbol=GAS, decimals=8.
    let cache = DataCache::new(false);
    let settings = ProtocolSettings::default();
    let gas = AtipicialDollar;
    let gas_state = build_native_contract_state(&gas, &settings, 0);
    deploy_contract_record(&cache, &gas_state);

    let snapshot = Arc::new(cache);
    let gas_hash = gas_state.hash;
    let reader = Aep17MetadataReaderImpl::new_with_native_contract_provider(
        Arc::clone(&snapshot),
        settings,
        standard_native_provider(),
    );

    let descriptor =
        AssetDescriptor::new(snapshot, &reader, gas_hash).expect("GAS descriptor must build");

    assert_eq!(descriptor.asset_id, gas_hash);
    assert_eq!(descriptor.asset_name, "AtipicialDollar");
    assert_eq!(descriptor.to_string(), "AtipicialDollar");
    assert_eq!(descriptor.symbol, "ATD");
    assert_eq!(descriptor.decimals, 8);
}

#[test]
fn descriptor_reads_atipicial_metadata() {
    // C# `Check_ATC`: name=AtipicialCoin, symbol=ATC, decimals=0 (exercises the
    // zero-decimals extraction path).
    let cache = DataCache::new(false);
    let settings = ProtocolSettings::default();
    let atipicial = AtipicialCoin;
    let atipicial_state = build_native_contract_state(&atipicial, &settings, 0);
    deploy_contract_record(&cache, &atipicial_state);

    let snapshot = Arc::new(cache);
    let atipicial_hash = atipicial_state.hash;
    let reader = Aep17MetadataReaderImpl::new_with_native_contract_provider(
        Arc::clone(&snapshot),
        settings,
        standard_native_provider(),
    );

    let descriptor =
        AssetDescriptor::new(snapshot, &reader, atipicial_hash).expect("ATC descriptor must build");

    assert_eq!(descriptor.asset_id, atipicial_hash);
    assert_eq!(descriptor.asset_name, "AtipicialCoin");
    assert_eq!(descriptor.to_string(), "AtipicialCoin");
    assert_eq!(descriptor.symbol, "ATC");
    assert_eq!(descriptor.decimals, 0);
}
