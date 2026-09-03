use super::super::super::json::build_atipicialfs_object_payload;
use super::super::super::proto::atipicialfs_v2;
use super::super::super::{AtipicialFsAuth, OracleAtipicialFsProtocol};
use super::super::auth::build_atipicialfs_request_verification_header;
use super::super::verify::{validate_atipicialfs_response, verify_atipicialfs_signature_bytes};
use atipicial_error::{CoreError, CoreResult};
use atipicial_payloads::OracleResponseCode;
use atipicial_wallets::KeyPair;
use prost::Message;
use tonic::transport::Channel;

impl OracleAtipicialFsProtocol {
    pub(in super::super::super) async fn fetch_header_grpc(
        &self,
        client: &mut atipicialfs_v2::object::object_service_client::ObjectServiceClient<Channel>,
        address: &atipicialfs_v2::refs::Address,
        auth: &AtipicialFsAuth,
        oracle_key: &KeyPair,
    ) -> (OracleResponseCode, String) {
        let object = match self
            .fetch_header_object_grpc(client, address, auth, oracle_key)
            .await
        {
            Ok(object) => object,
            Err(msg) => return (OracleResponseCode::Error, msg.to_string()),
        };
        let header = match object.header.as_ref() {
            Some(h) => h,
            None => {
                return (
                    OracleResponseCode::Error,
                    "object has no header".to_string(),
                );
            }
        };
        let payload = build_atipicialfs_object_payload(header, &object.payload);
        use base64::Engine as _;
        let payload_b64 = base64::engine::general_purpose::STANDARD.encode(payload.encode_to_vec());
        (OracleResponseCode::Success, payload_b64)
    }

    pub(in super::super::super) async fn fetch_header_object_grpc(
        &self,
        client: &mut atipicialfs_v2::object::object_service_client::ObjectServiceClient<Channel>,
        address: &atipicialfs_v2::refs::Address,
        auth: &AtipicialFsAuth,
        oracle_key: &KeyPair,
    ) -> CoreResult<atipicialfs_v2::object::Object> {
        let meta = auth.build_atipicialfs_meta_header()?;
        let body = atipicialfs_v2::object::head_request::Body {
            address: Some(address.clone()),
            main_only: false,
            raw: false,
        };
        let verify = build_atipicialfs_request_verification_header(&body, &meta, oracle_key)?;
        let request = atipicialfs_v2::object::HeadRequest {
            body: Some(body),
            meta_header: Some(meta),
            verify_header: Some(verify),
        };

        let response = client
            .head(request)
            .await
            .map_err(|_| CoreError::other("request failed"))?;
        let response = response.into_inner();
        let body = response
            .body
            .ok_or_else(|| CoreError::other("missing response body"))?;
        validate_atipicialfs_response(
            &body,
            response.meta_header.as_ref(),
            response.verify_header.as_ref(),
        )?;

        match body.head {
            Some(atipicialfs_v2::object::head_response::body::Head::Header(header_with_sig)) => {
                let header = header_with_sig
                    .header
                    .ok_or_else(|| CoreError::other("missing object header"))?;
                let signature = header_with_sig
                    .signature
                    .ok_or_else(|| CoreError::other("missing object signature"))?;
                let object_id = address
                    .object_id
                    .as_ref()
                    .ok_or_else(|| CoreError::other("missing object id"))?;
                if !verify_atipicialfs_signature_bytes(&signature, &object_id.encode_to_vec()) {
                    return Err(CoreError::other("invalid object signature"));
                }
                Ok(atipicialfs_v2::object::Object {
                    object_id: Some(object_id.clone()),
                    signature: Some(signature),
                    header: Some(header),
                    payload: Vec::new(),
                })
            }
            Some(atipicialfs_v2::object::head_response::body::Head::ShortHeader(_)) => {
                Err(CoreError::other("unexpected short header"))
            }
            Some(atipicialfs_v2::object::head_response::body::Head::SplitInfo(_)) => {
                Err(CoreError::other("split header response"))
            }
            None => Err(CoreError::other("missing header response")),
        }
    }
}
