use super::super::super::proto::atipicialfs_v2;

pub fn atipicialfs_json_signature(sig: &atipicialfs_v2::refs::Signature) -> Option<String> {
    use base64::Engine as _;
    use std::convert::TryFrom;
    let key = base64::engine::general_purpose::STANDARD.encode(&sig.key);
    let sign = base64::engine::general_purpose::STANDARD.encode(&sig.sign);
    let scheme = match atipicialfs_v2::refs::SignatureScheme::try_from(sig.scheme) {
        Ok(atipicialfs_v2::refs::SignatureScheme::EcdsaSha512) => "ECDSA_SHA512",
        Ok(atipicialfs_v2::refs::SignatureScheme::EcdsaRfc6979Sha256) => "ECDSA",
        Ok(atipicialfs_v2::refs::SignatureScheme::EcdsaRfc6979Sha256WalletConnect) => "WALLET_CONNECT",
        _ => "UNKNOWN",
    };
    Some(format!(
        "{{ \"key\": \"{key}\", \"signature\": \"{sign}\", \"scheme\": \"{scheme}\" }}"
    ))
}
