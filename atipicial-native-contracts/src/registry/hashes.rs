//! Well-known native contract script hashes.
//!
//! The 11 standard native contracts have hard-coded script hashes that
//! are the same across the entire Atipicial network (computed from the
//! native contract's manifest using `Helper::get_contract_hash(zero,
//! 0, name)`).
//!
//! Values below are taken from the Atipicial N3 MainNet/TestNet reference
//! implementation and verified by the
//! `print_canonical_native_hashes` integration test. They are exposed
//! as [`std::sync::LazyLock`] values (rather than `const`) because
//! `UInt160` does not yet expose a const byte-array constructor.

use atipicial_primitives::UInt160;
use std::sync::LazyLock;

fn native_hash(bytes: [u8; UInt160::LENGTH]) -> UInt160 {
    UInt160::from_array(bytes)
}

/// ContractManagement contract hash.
pub static CONTRACT_MANAGEMENT_HASH: LazyLock<UInt160> = LazyLock::new(|| {
    native_hash([
        0xfd, 0xa3, 0xfa, 0x43, 0x46, 0xea, 0x53, 0x2a, 0x25, 0x8f, 0xc4, 0x97, 0xdd, 0xad, 0xdb,
        0x64, 0x37, 0xc9, 0xfd, 0xff,
    ])
});

/// StdLib contract hash.
pub static STDLIB_HASH: LazyLock<UInt160> = LazyLock::new(|| {
    native_hash([
        0xc0, 0xef, 0x39, 0xce, 0xe0, 0xe4, 0xe9, 0x25, 0xc6, 0xc2, 0xa0, 0x6a, 0x79, 0xe1, 0x44,
        0x0d, 0xd8, 0x6f, 0xce, 0xac,
    ])
});

/// CryptoLib contract hash (BLS12-381).
pub static CRYPTO_LIB_HASH: LazyLock<UInt160> = LazyLock::new(|| {
    native_hash([
        0x1b, 0xf5, 0x75, 0xab, 0x11, 0x89, 0x68, 0x84, 0x13, 0x61, 0x0a, 0x35, 0xa1, 0x28, 0x86,
        0xcd, 0xe0, 0xb6, 0x6c, 0x72,
    ])
});

/// Ledger contract hash.
pub static LEDGER_CONTRACT_HASH: LazyLock<UInt160> = LazyLock::new(|| {
    native_hash([
        0xbe, 0xf2, 0x04, 0x31, 0x40, 0x36, 0x2a, 0x77, 0xc1, 0x50, 0x99, 0xc7, 0xe6, 0x4c, 0x12,
        0xf7, 0x00, 0xb6, 0x65, 0xda,
    ])
});

/// ATC token contract hash.
///
/// Derived from the renamed contract via
/// `Helper::get_contract_hash(UInt160::ZERO, 0, "AtipicialCoin")` (regenerated
/// with `cargo run -p atipicial-native-contracts --example print_native_hashes`
/// after the project-wide rename).
pub static ATC_TOKEN_HASH: LazyLock<UInt160> = LazyLock::new(|| {
    native_hash([
        0x97, 0x40, 0xae, 0xae, 0xa7, 0x52, 0x73, 0x62, 0x3a, 0xfa, 0x91, 0xad, 0x81, 0xbd, 0xd2,
        0xcf, 0x0f, 0x27, 0xf8, 0x12,
    ])
});

/// ATD token contract hash.
///
/// Derived from the renamed contract via
/// `Helper::get_contract_hash(UInt160::ZERO, 0, "AtipicialDollar")` (regenerated
/// with `cargo run -p atipicial-native-contracts --example print_native_hashes`
/// after the project-wide rename).
pub static GAS_TOKEN_HASH: LazyLock<UInt160> = LazyLock::new(|| {
    native_hash([
        0x4c, 0xc1, 0x0b, 0x0e, 0xec, 0x04, 0x3a, 0xdd, 0x22, 0x2d, 0x57, 0x58, 0x01, 0x5c, 0xe4,
        0x34, 0x02, 0x16, 0x7c, 0x48,
    ])
});

/// Policy contract hash.
pub static POLICY_CONTRACT_HASH: LazyLock<UInt160> = LazyLock::new(|| {
    native_hash([
        0x7b, 0xc6, 0x81, 0xc0, 0xa1, 0xf7, 0x1d, 0x54, 0x34, 0x57, 0xb6, 0x8b, 0xba, 0x8d, 0x5f,
        0x9f, 0xdd, 0x4e, 0x5e, 0xcc,
    ])
});

/// RoleManagement contract hash.
pub static ROLE_MANAGEMENT_HASH: LazyLock<UInt160> = LazyLock::new(|| {
    native_hash([
        0xe2, 0x95, 0xe3, 0x91, 0x54, 0x4c, 0x17, 0x8a, 0xd9, 0x4f, 0x03, 0xec, 0x4d, 0xcd, 0xff,
        0x78, 0x53, 0x4e, 0xcf, 0x49,
    ])
});

/// Oracle contract hash.
pub static ORACLE_CONTRACT_HASH: LazyLock<UInt160> = LazyLock::new(|| {
    native_hash([
        0x58, 0x87, 0x17, 0x11, 0x7e, 0x0a, 0xa8, 0x10, 0x72, 0xaf, 0xab, 0x71, 0xd2, 0xdd, 0x89,
        0xfe, 0x7c, 0x4b, 0x92, 0xfe,
    ])
});

/// Notary contract hash.
pub static NOTARY_HASH: LazyLock<UInt160> = LazyLock::new(|| {
    native_hash([
        0x3b, 0xec, 0x35, 0x31, 0x11, 0x9b, 0xba, 0xd7, 0x6d, 0xd0, 0x44, 0x92, 0x0b, 0x0d, 0xe6,
        0xc3, 0x19, 0x4f, 0xe1, 0xc1,
    ])
});

/// Treasury contract hash (activated by the `HF_Faun` hardfork).
///
/// Computed identically to the other native contracts as
/// `get_contract_hash(UInt160::ZERO, 0, "Treasury")`. Verified against the
/// C# Atipicial v3.10.1 reference (`UT_NativeContract.cs`, which pins
/// `"hash":"0x156326f25b1b5d839a4d326aeaa75383c9563ac1"` for Treasury).
pub static TREASURY_HASH: LazyLock<UInt160> = LazyLock::new(|| {
    native_hash([
        0xc1, 0x3a, 0x56, 0xc9, 0x83, 0x53, 0xa7, 0xea, 0x6a, 0x32, 0x4d, 0x9a, 0x83, 0x5d, 0x1b,
        0x5b, 0xf2, 0x26, 0x63, 0x15,
    ])
});

#[cfg(test)]
#[path = "../tests/registry/hashes.rs"]
mod tests;
