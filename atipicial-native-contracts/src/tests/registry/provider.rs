use super::*;
use crate::hashes::CRYPTO_LIB_HASH;

#[test]
fn provider_registers_exact_standard_catalog() {
    let provider = StandardNativeProvider::new();
    let specs = crate::standard_native_contract_specs();
    let contracts = provider.all_native_contracts();

    assert_eq!(contracts.len(), crate::STANDARD_NATIVE_CONTRACT_COUNT);
    assert_eq!(contracts.len(), specs.len());

    for (contract, spec) in contracts.iter().zip(specs) {
        assert_eq!(contract.id(), spec.id, "{} id", spec.name);
        assert_eq!(contract.name(), spec.name, "{} name", spec.name);
        assert_eq!(contract.hash(), spec.hash, "{} hash", spec.name);
        assert_eq!(
            provider
                .get_native_contract(&spec.hash)
                .expect("hash lookup")
                .name(),
            spec.name,
            "{} hash lookup",
            spec.name
        );
    }

    let expected_hashes = specs.iter().map(|spec| spec.hash).collect::<Vec<_>>();
    assert_eq!(provider.all_native_contract_hashes(), expected_hashes);
}

#[test]
fn provider_resolves_cryptolib_by_hash() {
    let provider = StandardNativeProvider::new();

    let by_hash = provider
        .get_native_contract(&CRYPTO_LIB_HASH)
        .expect("CryptoLib resolvable by hash");
    assert_eq!(by_hash.name(), "CryptoLib");
    assert_eq!(by_hash.id(), -3);
}

#[test]
fn provider_reads_policy_attribute_fee_without_leaking_storage_keys() {
    let provider = StandardNativeProvider::new();
    let snapshot = DataCache::new(false);
    let attribute_type = TransactionAttributeType::NotaryAssisted;
    snapshot.add(
        PolicyContract::attribute_fee_key(attribute_type.to_byte()),
        atipicial_storage::StorageItem::from_bytes(BigInt::from(1_234_567i64).to_signed_bytes_le()),
    );

    assert_eq!(
        provider
            .attribute_fee(&snapshot, attribute_type)
            .expect("typed Policy attribute fee"),
        1_234_567
    );
}

#[test]
fn standard_persist_hook_capabilities_match_protocol_implementations() {
    let contracts = StandardNativeProvider::new().all_native_contracts();
    let on_persist = contracts
        .iter()
        .filter(|contract| contract.has_on_persist_hook())
        .map(StandardNativeContract::name)
        .collect::<Vec<_>>();
    let post_persist = contracts
        .iter()
        .filter(|contract| contract.has_post_persist_hook())
        .map(StandardNativeContract::name)
        .collect::<Vec<_>>();

    assert_eq!(
        on_persist,
        [
            "ContractManagement",
            "LedgerContract",
            "AtipicialCoin",
            "AtipicialDollar",
            "Notary",
        ]
    );
    assert_eq!(
        post_persist,
        ["LedgerContract", "AtipicialCoin", "OracleContract"]
    );

    let mut transaction = atipicial_payloads::Transaction::new();
    transaction.set_script(vec![atipicial_vm::OpCode::RET.byte()]);
    let ordinary_block = atipicial_payloads::Block::from_parts(
        {
            let mut header = atipicial_payloads::Header::new();
            header.set_index(1);
            header
        },
        vec![transaction],
    );
    let settings = atipicial_config::ProtocolSettings::default();
    let ordinary_on_persist = contracts
        .iter()
        .filter(|contract| {
            <StandardNativeContract as NativeContract<StandardNativeProvider>>::should_run_on_persist(
                contract,
                &settings,
                &ordinary_block,
            )
        })
        .map(StandardNativeContract::name)
        .collect::<Vec<_>>();
    let ordinary_post_persist = contracts
        .iter()
        .filter(|contract| {
            <StandardNativeContract as NativeContract<StandardNativeProvider>>::should_run_post_persist(
                contract,
                &settings,
                &ordinary_block,
            )
        })
        .map(StandardNativeContract::name)
        .collect::<Vec<_>>();

    assert_eq!(
        ordinary_on_persist,
        ["LedgerContract", "AtipicialCoin", "AtipicialDollar"]
    );
    assert_eq!(ordinary_post_persist, ["LedgerContract", "AtipicialCoin"]);

    let mut attributed_transaction = atipicial_payloads::Transaction::new();
    attributed_transaction.set_script(vec![atipicial_vm::OpCode::RET.byte()]);
    attributed_transaction.set_attributes(vec![
        atipicial_payloads::TransactionAttribute::NotaryAssisted(
            atipicial_payloads::NotaryAssisted::new(1),
        ),
        atipicial_payloads::TransactionAttribute::OracleResponse(
            atipicial_payloads::OracleResponse::new(
                7,
                atipicial_primitives::OracleResponseCode::Success,
                Vec::new(),
            ),
        ),
    ]);
    let attributed_block = atipicial_payloads::Block::from_parts(
        {
            let mut header = atipicial_payloads::Header::new();
            header.set_index(1);
            header
        },
        vec![attributed_transaction],
    );
    let attributed_on_persist = contracts
        .iter()
        .filter(|contract| {
            <StandardNativeContract as NativeContract<StandardNativeProvider>>::should_run_on_persist(
                contract,
                &settings,
                &attributed_block,
            )
        })
        .map(StandardNativeContract::name)
        .collect::<Vec<_>>();
    let attributed_post_persist = contracts
        .iter()
        .filter(|contract| {
            <StandardNativeContract as NativeContract<StandardNativeProvider>>::should_run_post_persist(
                contract,
                &settings,
                &attributed_block,
            )
        })
        .map(StandardNativeContract::name)
        .collect::<Vec<_>>();

    assert_eq!(
        attributed_on_persist,
        [
            "LedgerContract",
            "AtipicialCoin",
            "AtipicialDollar",
            "Notary"
        ]
    );
    assert_eq!(
        attributed_post_persist,
        ["LedgerContract", "AtipicialCoin", "OracleContract"]
    );
}

#[test]
fn provider_current_block_index_feeds_engine_without_persisting_block() {
    use crate::LedgerContract;
    use atipicial_config::ProtocolSettings;
    use atipicial_execution::ApplicationEngine;
    use atipicial_payloads::Block;
    use atipicial_primitives::{TriggerType, UInt256};
    use atipicial_storage::StorageItem;
    use atipicial_storage::persistence::DataCache;
    use std::sync::Arc;

    let cache = Arc::new(DataCache::new(false));
    let current_hash = UInt256::from_bytes(&[0x34; 32]).unwrap();
    cache.add(
        LedgerContract::current_block_storage_key(),
        StorageItem::from_bytes(
            LedgerContract::new()
                .serialize_hash_index_state(&current_hash, 1234)
                .unwrap(),
        ),
    );

    let engine = ApplicationEngine::new_with_native_contract_provider(
        TriggerType::Application,
        None,
        Arc::clone(&cache),
        None::<Block>,
        ProtocolSettings::default(),
        1_000_000,
        atipicial_execution::NoDiagnostic,
        std::sync::Arc::new(crate::StandardNativeProvider::new()),
    )
    .expect("engine builds");

    assert_eq!(engine.current_block_index(), 1234);
}
