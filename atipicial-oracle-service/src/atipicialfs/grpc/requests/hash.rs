use super::super::super::proto::atipicialfs_v2;
use super::super::super::{AtipicialFsAuth, AtipicialFsRange, OracleAtipicialFsProtocol};
use super::super::auth::build_atipicialfs_request_verification_header;
use super::super::verify::validate_atipicialfs_response;
use atipicial_payloads::OracleResponseCode;
use atipicial_primitives::UInt256;
use atipicial_wallets::KeyPair;
use tonic::transport::Channel;

impl OracleAtipicialFsProtocol {
    pub(in super::super::super) async fn fetch_hash_grpc(
        &self,
        client: &mut atipicialfs_v2::object::object_service_client::ObjectServiceClient<Channel>,
        address: &atipicialfs_v2::refs::Address,
        range: Option<AtipicialFsRange>,
        auth: &AtipicialFsAuth,
        oracle_key: &KeyPair,
    ) -> (OracleResponseCode, String) {
        match range {
            None => {
                let object = match self
                    .fetch_header_object_grpc(client, address, auth, oracle_key)
                    .await
                {
                    Ok(object) => object,
                    Err(_) => return (OracleResponseCode::Error, String::new()),
                };
                let header = match object.header.as_ref() {
                    Some(header) => header,
                    None => return (OracleResponseCode::Error, String::new()),
                };
                let checksum = match header.payload_hash.as_ref() {
                    Some(checksum) => checksum,
                    None => return (OracleResponseCode::Error, String::new()),
                };
                let hash =
                    UInt256::from_bytes(&checksum.sum).map_err(|_| OracleResponseCode::Error);
                match hash {
                    Ok(hash) => (OracleResponseCode::Success, format!("\"{}\"", hash)),
                    Err(code) => (code, String::new()),
                }
            }
            Some(range) => {
                let meta = match auth.build_atipicialfs_meta_header() {
                    Ok(meta) => meta,
                    Err(_) => return (OracleResponseCode::Error, String::new()),
                };
                let body = atipicialfs_v2::object::get_range_hash_request::Body {
                    address: Some(address.clone()),
                    ranges: vec![atipicialfs_v2::object::Range {
                        offset: range.offset,
                        length: range.length,
                    }],
                    salt: Vec::new(),
                    r#type: atipicialfs_v2::refs::ChecksumType::Sha256 as i32,
                };
                let verify = match build_atipicialfs_request_verification_header(&body, &meta, oracle_key)
                {
                    Ok(verify) => verify,
                    Err(_) => return (OracleResponseCode::Error, String::new()),
                };
                let request = atipicialfs_v2::object::GetRangeHashRequest {
                    body: Some(body),
                    meta_header: Some(meta),
                    verify_header: Some(verify),
                };

                let response = match client.get_range_hash(request).await {
                    Ok(response) => response.into_inner(),
                    Err(_) => return (OracleResponseCode::Error, String::new()),
                };
                let body = match response.body.as_ref() {
                    Some(body) => body,
                    None => return (OracleResponseCode::Error, String::new()),
                };
                if validate_atipicialfs_response(
                    body,
                    response.meta_header.as_ref(),
                    response.verify_header.as_ref(),
                )
                .is_err()
                {
                    return (OracleResponseCode::Error, String::new());
                }
                if body.hash_list.is_empty() {
                    return (OracleResponseCode::Error, String::new());
                }
                let hash =
                    UInt256::from_bytes(&body.hash_list[0]).map_err(|_| OracleResponseCode::Error);
                match hash {
                    Ok(hash) => (OracleResponseCode::Success, format!("\"{}\"", hash)),
                    Err(code) => (code, String::new()),
                }
            }
        }
    }
}
