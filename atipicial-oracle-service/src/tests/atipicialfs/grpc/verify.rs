use super::{atipicialfs_v2, verify_atipicialfs_signature_bytes};
use crate::atipicialfs::auth::AtipicialFsBearerSigner;
use atipicial_crypto::Secp256r1Crypto;
use atipicial_wallets::KeyPair;

#[test]
fn verifies_atipicialfs_signature_from_core_signing_path() {
    let private_key = Secp256r1Crypto::generate_private_key();
    let key = KeyPair::from_private_key(&private_key).expect("test key");
    let data = b"atipicialfs response body";
    let signature = atipicialfs_v2::refs::Signature {
        key: key.compressed_public_key(),
        sign: AtipicialFsBearerSigner::sign_atipicialfs_sha512(data, &key).expect("atipicialfs signature"),
        scheme: atipicialfs_v2::refs::SignatureScheme::EcdsaSha512 as i32,
    };

    assert!(verify_atipicialfs_signature_bytes(&signature, data));
    assert!(!verify_atipicialfs_signature_bytes(&signature, b"mutated"));
}
