//! # atipicial-native-contracts::tests::style
//!
//! Test module grouping style behavior coverage for atipicial-native-contracts.
//!
//! ## Boundary
//!
//! This is test/benchmark-only code for atipicial-native-contracts; it may assemble
//! fixtures but must not introduce production behavior.
//!
//! ## Contents
//!
//! - `events`: Mempool event records emitted to subscribers.

use crate::{
    AEP11_PAYMENT_METHOD, AEP17_PAYMENT_METHOD, AEP17_STANDARD, AEP17_TRANSFER_EVENT,
    AEP26_STANDARD, AEP27_STANDARD, AEP30_STANDARD, STANDARD_NATIVE_CONTRACT_COUNT,
};

#[path = "events.rs"]
mod events;

pub(super) fn standard_contract_sources()
-> [(&'static str, &'static str); STANDARD_NATIVE_CONTRACT_COUNT] {
    [
        (
            "ContractManagement",
            concat!(
                include_str!("../../contract_management/constants.rs"),
                "\n",
                include_str!("../../contract_management/operations/storage.rs"),
                "\n",
                include_str!("../../contract_management/operations/validation.rs"),
                "\n",
                include_str!("../../contract_management/operations.rs"),
                "\n",
                include_str!("../../contract_management/initialize.rs"),
                "\n",
                include_str!("../../contract_management/invoke.rs"),
                "\n",
                include_str!("../../contract_management/metadata.rs"),
                "\n",
                include_str!("../../contract_management/persist.rs"),
                "\n",
                include_str!("../../contract_management/mod.rs"),
            ),
        ),
        (
            "StdLib",
            concat!(
                include_str!("../../std_lib/args.rs"),
                "\n",
                include_str!("../../std_lib/encoding.rs"),
                "\n",
                include_str!("../../std_lib/invoke.rs"),
                "\n",
                include_str!("../../std_lib/memory.rs"),
                "\n",
                include_str!("../../std_lib/serialization.rs"),
                "\n",
                include_str!("../../std_lib/numeric.rs"),
                "\n",
                include_str!("../../std_lib/strings.rs"),
                "\n",
                include_str!("../../std_lib/metadata.rs"),
                "\n",
                include_str!("../../std_lib/mod.rs"),
            ),
        ),
        (
            "CryptoLib",
            concat!(
                include_str!("../../crypto_lib/bls.rs"),
                "\n",
                include_str!("../../crypto_lib/hashing.rs"),
                "\n",
                include_str!("../../crypto_lib/metadata.rs"),
                "\n",
                include_str!("../../crypto_lib/signatures.rs"),
                "\n",
                include_str!("../../crypto_lib/invoke.rs"),
                "\n",
                include_str!("../../crypto_lib/mod.rs")
            ),
        ),
        (
            "LedgerContract",
            concat!(
                include_str!("../../ledger_contract/storage.rs"),
                "\n",
                include_str!("../../ledger_contract/wire.rs"),
                "\n",
                include_str!("../../ledger_contract/invoke.rs"),
                "\n",
                include_str!("../../ledger_contract/metadata.rs"),
                "\n",
                include_str!("../../ledger_contract/queries.rs"),
                "\n",
                include_str!("../../ledger_contract/mod.rs"),
            ),
        ),
        (
            "AtipicialCoin",
            concat!(
                include_str!("../../atipicial_token/constants.rs"),
                "\n",
                include_str!("../../atipicial_token/storage/account.rs"),
                "\n",
                include_str!("../../atipicial_token/storage/candidates.rs"),
                "\n",
                include_str!("../../atipicial_token/storage/committee.rs"),
                "\n",
                include_str!("../../atipicial_token/storage/economics.rs"),
                "\n",
                include_str!("../../atipicial_token/storage/keys.rs"),
                "\n",
                include_str!("../../atipicial_token/storage/mod.rs"),
                "\n",
                include_str!("../../atipicial_token/storage/points.rs"),
                "\n",
                include_str!("../../atipicial_token/storage/views.rs"),
                "\n",
                include_str!("../../atipicial_token/transfers.rs"),
                "\n",
                include_str!("../../atipicial_token/fast_forward.rs"),
                "\n",
                include_str!("../../atipicial_token/initialize.rs"),
                "\n",
                include_str!("../../atipicial_token/metadata.rs"),
                "\n",
                include_str!("../../atipicial_token/persist.rs"),
                "\n",
                include_str!("../../atipicial_token/providers.rs"),
                "\n",
                include_str!("../../atipicial_token/invoke.rs"),
                "\n",
                include_str!("../../atipicial_token/mod.rs"),
            ),
        ),
        (
            "AtipicialDollar",
            concat!(
                include_str!("../../atipicial_dollar/metadata.rs"),
                "\n",
                include_str!("../../atipicial_dollar/initialize.rs"),
                "\n",
                include_str!("../../atipicial_dollar/persist.rs"),
                "\n",
                include_str!("../../atipicial_dollar/invoke.rs"),
                "\n",
                include_str!("../../atipicial_dollar/storage.rs"),
                "\n",
                include_str!("../../atipicial_dollar/transfers.rs"),
                "\n",
                include_str!("../../atipicial_dollar/mod.rs"),
            ),
        ),
        (
            "PolicyContract",
            concat!(
                include_str!("../../policy_contract/constants.rs"),
                "\n",
                include_str!("../../policy_contract/storage/recovery.rs"),
                "\n",
                include_str!("../../policy_contract/storage/whitelist.rs"),
                "\n",
                include_str!("../../policy_contract/storage.rs"),
                "\n",
                include_str!("../../policy_contract/initialize.rs"),
                "\n",
                include_str!("../../policy_contract/invoke.rs"),
                "\n",
                include_str!("../../policy_contract/metadata.rs"),
                "\n",
                include_str!("../../policy_contract/provider.rs"),
                "\n",
                include_str!("../../policy_contract/mod.rs"),
            ),
        ),
        (
            "RoleManagement",
            concat!(
                include_str!("../../role_management/storage.rs"),
                "\n",
                include_str!("../../role_management/node_list.rs"),
                "\n",
                include_str!("../../role_management/providers.rs"),
                "\n",
                include_str!("../../role_management/invoke.rs"),
                "\n",
                include_str!("../../role_management/metadata.rs"),
                "\n",
                include_str!("../../role_management/mod.rs"),
            ),
        ),
        (
            "OracleContract",
            concat!(
                include_str!("../../oracle_contract/constants.rs"),
                "\n",
                include_str!("../../oracle_contract/request.rs"),
                "\n",
                include_str!("../../oracle_contract/storage.rs"),
                "\n",
                include_str!("../../oracle_contract/initialize.rs"),
                "\n",
                include_str!("../../oracle_contract/invoke.rs"),
                "\n",
                include_str!("../../oracle_contract/metadata.rs"),
                "\n",
                include_str!("../../oracle_contract/persist.rs"),
                "\n",
                include_str!("../../oracle_contract/mod.rs"),
            ),
        ),
        (
            "Notary",
            concat!(
                include_str!("../../notary/constants.rs"),
                "\n",
                include_str!("../../notary/storage.rs"),
                "\n",
                include_str!("../../notary/initialize.rs"),
                "\n",
                include_str!("../../notary/invoke.rs"),
                "\n",
                include_str!("../../notary/metadata.rs"),
                "\n",
                include_str!("../../notary/persist.rs"),
                "\n",
                include_str!("../../notary/mod.rs"),
            ),
        ),
        (
            "Treasury",
            concat!(
                include_str!("../../treasury/metadata.rs"),
                "\n",
                include_str!("../../treasury/invoke.rs"),
                "\n",
                include_str!("../../treasury/mod.rs")
            ),
        ),
    ]
}

fn standard_contract_dispatch_sources()
-> [(&'static str, &'static str); STANDARD_NATIVE_CONTRACT_COUNT] {
    [
        (
            "ContractManagement",
            concat!(
                include_str!("../../contract_management/metadata.rs"),
                "\n",
                include_str!("../../contract_management/invoke.rs"),
            ),
        ),
        (
            "StdLib",
            concat!(
                include_str!("../../std_lib/metadata.rs"),
                "\n",
                include_str!("../../std_lib/invoke.rs"),
            ),
        ),
        (
            "CryptoLib",
            concat!(
                include_str!("../../crypto_lib/bls.rs"),
                "\n",
                include_str!("../../crypto_lib/hashing.rs"),
                "\n",
                include_str!("../../crypto_lib/metadata.rs"),
                "\n",
                include_str!("../../crypto_lib/invoke.rs"),
                "\n",
                include_str!("../../crypto_lib/signatures.rs"),
            ),
        ),
        (
            "LedgerContract",
            concat!(
                include_str!("../../ledger_contract/metadata.rs"),
                "\n",
                include_str!("../../ledger_contract/invoke.rs"),
            ),
        ),
        (
            "AtipicialCoin",
            concat!(
                include_str!("../../atipicial_token/metadata.rs"),
                "\n",
                include_str!("../../atipicial_token/invoke.rs"),
            ),
        ),
        (
            "AtipicialDollar",
            concat!(
                include_str!("../../atipicial_dollar/metadata.rs"),
                "\n",
                include_str!("../../atipicial_dollar/invoke.rs"),
            ),
        ),
        (
            "PolicyContract",
            concat!(
                include_str!("../../policy_contract/metadata.rs"),
                "\n",
                include_str!("../../policy_contract/invoke.rs"),
            ),
        ),
        (
            "RoleManagement",
            concat!(
                include_str!("../../role_management/metadata.rs"),
                "\n",
                include_str!("../../role_management/invoke.rs"),
            ),
        ),
        (
            "OracleContract",
            concat!(
                include_str!("../../oracle_contract/metadata.rs"),
                "\n",
                include_str!("../../oracle_contract/invoke.rs"),
            ),
        ),
        (
            "Notary",
            concat!(
                include_str!("../../notary/metadata.rs"),
                "\n",
                include_str!("../../notary/invoke.rs"),
            ),
        ),
        (
            "Treasury",
            concat!(
                include_str!("../../treasury/metadata.rs"),
                "\n",
                include_str!("../../treasury/invoke.rs"),
            ),
        ),
    ]
}

fn dispatch_source_mentions_method(source: &str, method_name: &str) -> bool {
    source.contains(&format!("\"{method_name}\""))
        || (method_name == "symbol" && source.contains("aep17_symbol_method"))
        || (method_name == "decimals" && source.contains("aep17_decimals_method"))
        || (method_name == "totalSupply" && source.contains("aep17_total_supply_method"))
        || (method_name == "balanceOf" && source.contains("aep17_balance_of_method"))
        || (method_name == "transfer" && source.contains("aep17_transfer_method"))
        || (method_name == AEP17_PAYMENT_METHOD
            && (source.contains("AEP17_PAYMENT_METHOD") || source.contains("aep17_payment_method")))
        || (method_name == AEP11_PAYMENT_METHOD
            && (source.contains("AEP11_PAYMENT_METHOD") || source.contains("aep11_payment_method")))
}

#[test]
fn native_contract_style_sources_follow_canonical_catalog_order() {
    let style_names = standard_contract_sources().map(|(name, _)| name);
    let catalog_names = crate::standard_native_contract_specs().map(|spec| spec.name);

    assert_eq!(style_names, catalog_names);
}

#[test]
fn native_contract_handles_use_uniform_macros() {
    for (name, source) in standard_contract_sources() {
        let production = source;
        assert!(
            production.contains("native_contract_handle!("),
            "{name} should declare its handle via native_contract_handle!"
        );
        assert!(
            production.contains(&format!("native_contract_identity!({name});")),
            "{name} should implement NativeContract identity via native_contract_identity!"
        );
        assert!(
            !production.contains("fn id(&self)"),
            "{name} should not hand-write NativeContract::id"
        );
        assert!(
            !production.contains("fn hash(&self)"),
            "{name} should not hand-write NativeContract::hash"
        );
        assert!(
            !production.contains("fn name(&self)"),
            "{name} should not hand-write NativeContract::name"
        );
        assert!(
            !production.contains("fn as_any(&self)"),
            "{name} should not hand-write NativeContract::as_any"
        );
    }
}

#[test]
fn native_contract_invocation_boundaries_use_invoke_modules() {
    for (name, source) in standard_contract_sources() {
        let production = source;
        assert!(
            production.contains("mod invoke;"),
            "{name} should keep native method handlers in an invoke module"
        );
        assert!(
            production.contains("native_contract_dispatch!("),
            "{name} should declare native method dispatch through native_contract_dispatch!"
        );
        assert!(
            !production.contains("self.invoke_native(engine, method, args)"),
            "{name} root NativeContract::invoke should not hand-write dispatch shims"
        );
        assert!(
            !production.contains("pub(super) fn invoke_native("),
            "{name} invoke module should contain method handlers, not duplicate dispatch wrappers"
        );
        assert!(
            !production.contains("mod dispatch;"),
            "{name} should not use a second dispatch module name"
        );
    }
}

#[test]
fn native_contract_provider_uses_typed_catalog_not_trait_objects() {
    let catalog = include_str!("../../registry/catalog.rs");
    let provider = include_str!("../../registry/provider.rs");
    let standard = include_str!("../../registry/standard.rs");
    let macros = include_str!("../../support/macros.rs");

    assert!(
        !catalog.contains("dyn NativeContract"),
        "standard native catalog should use typed handles, not NativeContract trait objects"
    );
    assert!(
        !provider.contains("dyn NativeContract"),
        "standard native provider should use typed handles, not NativeContract trait objects"
    );
    assert!(
        standard.contains("pub enum StandardNativeContract"),
        "standard native dispatch should stay on the closed typed enum"
    );
    assert!(
        !standard.contains("dyn NativeContract"),
        "standard native enum should delegate typed handles, not erased trait objects"
    );
    assert!(
        macros.contains("ApplicationEngine<P, D, B>"),
        "native dispatch bindings should stay generic over diagnostics and cache backing"
    );
}

#[test]
fn native_contract_manifest_methods_are_represented_in_dispatch_sources() {
    for ((source_name, source), contract) in standard_contract_dispatch_sources()
        .into_iter()
        .zip(crate::standard_native_contracts())
    {
        assert_eq!(source_name, contract.name());
        for method in contract.methods() {
            assert!(
                dispatch_source_mentions_method(source, &method.name),
                "{} manifest method '{}' should be represented in dispatch source",
                contract.name(),
                method.name
            );
        }
    }
}

#[test]
fn native_contract_dispatch_uses_binding_tables_not_raw_method_switches() {
    for (name, source) in standard_contract_dispatch_sources() {
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        assert!(
            !production.contains("match method"),
            "{name} should dispatch native ABI methods through NativeMethodBinding tables, not raw method-name matches"
        );
        assert!(
            production.contains("NativeMethodBinding::new("),
            "{name} dispatch source should bind ABI metadata to concrete native handlers"
        );
    }
}

#[test]
fn native_contract_metadata_tables_use_handle_name_prefixes() {
    let expected = [
        ("ContractManagement", "CONTRACT_MANAGEMENT"),
        ("StdLib", "STD_LIB"),
        ("CryptoLib", "CRYPTO_LIB"),
        ("LedgerContract", "LEDGER_CONTRACT"),
        ("AtipicialCoin", "ATC_TOKEN"),
        ("AtipicialDollar", "GAS_TOKEN"),
        ("PolicyContract", "POLICY_CONTRACT"),
        ("RoleManagement", "ROLE_MANAGEMENT"),
        ("OracleContract", "ORACLE_CONTRACT"),
        ("Notary", "NOTARY"),
        ("Treasury", "TREASURY"),
    ];

    for ((name, source), (expected_name, prefix)) in
        standard_contract_sources().into_iter().zip(expected)
    {
        assert_eq!(name, expected_name);
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        assert!(
            production.contains(&format!("static {prefix}_METHODS")),
            "{name} metadata methods should use the full handle prefix {prefix}_METHODS"
        );
        let has_events = production.contains("fn event_descriptors(&self)");
        if has_events {
            assert!(
                production.contains(&format!("static {prefix}_EVENTS")),
                "{name} metadata events should use the full handle prefix {prefix}_EVENTS"
            );
        }
    }
}

#[test]
fn native_contracts_reference_metadata_tables_through_module_namespace() {
    for (name, source) in standard_contract_sources() {
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        assert!(
            !production.contains("use metadata::"),
            "{name} should reference metadata tables as metadata::CONTRACT_TABLE for consistent native-contract style"
        );
    }
}

#[test]
fn native_contract_storage_keys_use_shared_builders() {
    for (name, source) in standard_contract_sources() {
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        assert!(
            !production.contains("StorageKey::create("),
            "{name} should route single-prefix native storage keys through crate::keys"
        );
        assert!(
            !production.contains("StorageKey::create_with_bytes("),
            "{name} should route raw-suffix native storage keys through crate::keys"
        );
        assert!(
            !production.contains("StorageKey::create_with_"),
            "{name} should route typed native storage keys through crate::keys"
        );
        assert!(
            !production.contains("StorageKey::new(Self::ID"),
            "{name} should not hand-wrap raw suffixes with StorageKey::new(Self::ID, ...)"
        );
        assert!(
            !production.contains("StorageKey::new(\n            Self::ID"),
            "{name} should not hand-wrap raw suffixes with StorageKey::new(Self::ID, ...)"
        );
        assert!(
            !production.contains("StorageKey::new(\n                Self::ID"),
            "{name} should not hand-wrap raw suffixes with StorageKey::new(Self::ID, ...)"
        );
    }
}

#[test]
fn native_contract_supported_standards_use_shared_helper() {
    for (name, source) in standard_contract_sources() {
        for standard in [
            AEP17_STANDARD,
            AEP26_STANDARD,
            AEP27_STANDARD,
            AEP30_STANDARD,
        ] {
            assert!(
                !source.contains(&format!("\"{standard}\".to_string()")),
                "{name} should build manifest supported standards through native_supported_standards"
            );
        }
    }
}

#[test]
fn aep17_transfer_notifications_use_shared_event_name() {
    assert_eq!(AEP17_TRANSFER_EVENT, "Transfer");

    for (name, source) in standard_contract_sources() {
        assert!(
            !source.contains("\"Transfer\".to_string()"),
            "{name} should emit AEP-17 Transfer notifications via AEP17_TRANSFER_EVENT"
        );
    }
}

#[test]
fn aep17_transfer_notifications_use_shared_payload_helper() {
    for name in ["AtipicialDollar", "AtipicialCoin"] {
        let source = standard_contract_sources()
            .into_iter()
            .find(|(contract_name, _)| *contract_name == name)
            .map(|(_, source)| source)
            .expect("AEP-17 token source should be available");

        assert!(
            source.contains("aep17_transfer_notification_state"),
            "{name} should build Transfer notification payloads through aep17_transfer_notification_state"
        );
        assert!(
            !source.contains("AEP17_TRANSFER_EVENT.to_owned(),\n                vec!["),
            "{name} should not hand-write Transfer notification payload vectors"
        );
    }
}

#[test]
fn aep17_token_method_tables_use_shared_abi_helpers() {
    for name in ["AtipicialDollar", "AtipicialCoin"] {
        let source = standard_contract_sources()
            .into_iter()
            .find(|(contract_name, _)| *contract_name == name)
            .map(|(_, source)| source)
            .expect("AEP-17 token source should be available");

        for helper in [
            "aep17_symbol_method",
            "aep17_decimals_method",
            "aep17_total_supply_method",
            "aep17_balance_of_method",
            "aep17_transfer_method",
        ] {
            assert!(
                source.contains(helper),
                "{name} should build AEP-17 ABI method descriptors through {helper}"
            );
        }

        for conversion in [".into()", ".to_string()", ".to_owned()"] {
            assert!(
                !source.contains(&format!(
                    "NativeMethod::new(\n            \"transfer\"{conversion}"
                )),
                "{name} should not hand-write the AEP-17 transfer method descriptor"
            );
        }
    }
}

#[test]
fn aep17_payment_callbacks_use_shared_method_and_payload_helper() {
    for name in ["AtipicialDollar", "AtipicialCoin"] {
        let source = standard_contract_sources()
            .into_iter()
            .find(|(contract_name, _)| *contract_name == name)
            .map(|(_, source)| source)
            .expect("AEP-17 token source should be available");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(
            production.contains("AEP17_PAYMENT_METHOD"),
            "{name} should queue AEP-17 payment callbacks via AEP17_PAYMENT_METHOD"
        );
        assert!(
            production.contains("aep17_payment_callback_args"),
            "{name} should build AEP-17 payment callback args through aep17_payment_callback_args"
        );
        assert!(
            !production.contains("\"onAEP17Payment\",\n            vec!["),
            "{name} should not hand-write onAEP17Payment callback payload vectors"
        );
        assert!(
            production.contains("aep17_payment_data_item"),
            "{name} should decode transfer `data` through aep17_payment_data_item"
        );
        assert!(
            !production.contains(
                "BinarySerializer::deserialize(data, &ExecutionEngineLimits::default(), None)"
            ),
            "{name} should not hand-roll AEP-17 transfer data deserialization"
        );
    }
}

#[test]
fn aep17_amount_args_use_shared_raw_integer_parser() {
    for name in ["AtipicialDollar", "AtipicialCoin", "Notary"] {
        let source = standard_contract_sources()
            .into_iter()
            .find(|(contract_name, _)| *contract_name == name)
            .map(|(_, source)| source)
            .expect("AEP-17 amount source should be available");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(
            production.contains("crate::args::raw_required_integer_arg"),
            "{name} should decode AEP-17 amount args through raw_required_integer_arg"
        );
        assert!(
            !production.contains("BigInt::from_signed_bytes_le(args"),
            "{name} should not hand-roll raw AEP-17 amount integer decoding"
        );
    }
}

#[test]
fn aep_payment_method_descriptors_use_shared_helpers() {
    for name in ["AtipicialCoin", "Notary"] {
        let source = standard_contract_sources()
            .into_iter()
            .find(|(contract_name, _)| *contract_name == name)
            .map(|(_, source)| source)
            .expect("AEP-17 payment source should be available");
        assert!(
            source.contains("aep17_payment_method"),
            "{name} should describe onAEP17Payment through aep17_payment_method"
        );
    }

    let treasury = standard_contract_sources()
        .into_iter()
        .find(|(name, _)| *name == "Treasury")
        .map(|(_, source)| source)
        .expect("Treasury source should be available");
    assert!(
        treasury.contains("aep17_payment_method"),
        "Treasury should describe onAEP17Payment through aep17_payment_method"
    );
    assert!(
        treasury.contains("aep11_payment_method"),
        "Treasury should describe onAEP11Payment through aep11_payment_method"
    );
}
