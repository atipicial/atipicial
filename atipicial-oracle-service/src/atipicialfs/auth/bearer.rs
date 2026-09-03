use super::super::{AtipicialFsAuth, OracleServiceSettings};
use super::signing::AtipicialFsBearerSigner;
use atipicial_primitives::hex_util;
use atipicial_wallets::KeyPair;

impl AtipicialFsAuth {
    pub(crate) fn build_atipicialfs_auth(
        settings: &OracleServiceSettings,
        oracle_key: Option<&KeyPair>,
    ) -> AtipicialFsAuth {
        let token = settings
            .atipicialfs_bearer_token
            .as_ref()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        let mut signature = settings
            .atipicialfs_bearer_signature
            .as_ref()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        let mut signature_key = settings
            .atipicialfs_bearer_signature_key
            .as_ref()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());

        if settings.atipicialfs_auto_sign_bearer
            && signature.is_none()
            && signature_key.is_none()
            && token.as_deref().map(strip_bearer_prefix).is_some()
        {
            if let (Some(token_value), Some(key)) = (token.as_deref(), oracle_key) {
                if let Some((sig, key_bytes)) = AtipicialFsBearerSigner::sign_atipicialfs_bearer(
                    strip_bearer_prefix(token_value),
                    key,
                    settings.atipicialfs_wallet_connect,
                ) {
                    signature = Some(hex_util::encode_hex(&sig));
                    signature_key = Some(hex_util::encode_hex(&key_bytes));
                }
            }
        }

        AtipicialFsAuth {
            token,
            signature,
            signature_key,
            wallet_connect: settings.atipicialfs_wallet_connect,
        }
    }
}

pub(crate) fn strip_bearer_prefix(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.len() >= 7 && trimmed[..7].eq_ignore_ascii_case("bearer ") {
        &trimmed[7..]
    } else {
        trimmed
    }
}
