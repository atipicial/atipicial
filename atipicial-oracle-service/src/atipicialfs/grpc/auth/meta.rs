use super::super::super::AtipicialFsAuth;
use super::super::super::proto::atipicialfs_v2;
use atipicial_error::{CoreError, CoreResult};
use atipicial_primitives::hex_util;

const ATCFS_SDK_VERSION_MAJOR: u32 = 2;
const ATCFS_SDK_VERSION_MINOR: u32 = 11;

impl AtipicialFsAuth {
    pub(crate) fn build_atipicialfs_meta_header(
        &self,
    ) -> CoreResult<atipicialfs_v2::session::RequestMetaHeader> {
        let mut meta = atipicialfs_v2::session::RequestMetaHeader {
            version: Some(atipicialfs_v2::refs::Version {
                major: ATCFS_SDK_VERSION_MAJOR,
                minor: ATCFS_SDK_VERSION_MINOR,
            }),
            ttl: 2,
            ..Default::default()
        };
        if let Some(token) = build_atipicialfs_bearer_token(self)? {
            meta.bearer_token = Some(token);
        }
        Ok(meta)
    }
}

fn build_atipicialfs_bearer_token(auth: &AtipicialFsAuth) -> CoreResult<Option<atipicialfs_v2::acl::BearerToken>> {
    let token = auth
        .token
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());
    let Some(token) = token else {
        return Ok(None);
    };
    let data = base64::engine::general_purpose::STANDARD
        .decode(strip_bearer_prefix(token))
        .map_err(|_| CoreError::other("invalid bearer token"))?;
    if data.is_empty() {
        return Ok(None);
    }

    let signature = auth
        .signature
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());
    let signature_key = auth
        .signature_key
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());

    match (signature, signature_key) {
        (Some(signature), Some(signature_key)) => {
            let body = atipicialfs_v2::acl::bearer_token::Body::decode(data.as_slice())
                .map_err(|_| CoreError::other("invalid bearer token body"))?;
            let signature_bytes = decode_atipicialfs_signature_bytes(signature)?;
            let key_bytes = decode_atipicialfs_signature_bytes(signature_key)?;
            let scheme = if auth.wallet_connect {
                atipicialfs_v2::refs::SignatureScheme::EcdsaRfc6979Sha256WalletConnect as i32
            } else {
                atipicialfs_v2::refs::SignatureScheme::EcdsaSha512 as i32
            };
            Ok(Some(atipicialfs_v2::acl::BearerToken {
                body: Some(body),
                signature: Some(atipicialfs_v2::refs::Signature {
                    key: key_bytes,
                    sign: signature_bytes,
                    scheme,
                }),
            }))
        }
        (None, None) => {
            let token = atipicialfs_v2::acl::BearerToken::decode(data.as_slice())
                .map_err(|_| CoreError::other("invalid bearer token"))?;
            Ok(Some(token))
        }
        _ => Err(CoreError::other("missing bearer signature or key")),
    }
}

fn decode_atipicialfs_signature_bytes(value: &str) -> CoreResult<Vec<u8>> {
    let trimmed = value.trim();
    let normalized = normalize_atipicialfs_hex_header(trimmed);
    if let Ok(decoded) = hex_util::decode_hex(&normalized) {
        return Ok(decoded);
    }
    base64::engine::general_purpose::STANDARD
        .decode(trimmed)
        .map_err(|_| CoreError::other("invalid atipicialfs signature"))
}

use super::super::super::auth::strip_bearer_prefix;
use super::super::super::json::normalize_atipicialfs_hex_header;
use base64::Engine as _;
use prost::Message;
