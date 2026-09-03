use super::super::super::proto::atipicialfs_v2;
use super::super::super::{AtipicialFsAuth, AtipicialFsRange, OracleAtipicialFsProtocol};
use super::super::auth::build_atipicialfs_request_verification_header;
use super::super::verify::validate_atipicialfs_response;
use atipicial_payloads::OracleResponseCode;
use atipicial_wallets::KeyPair;
use tonic::transport::Channel;

impl OracleAtipicialFsProtocol {
    pub(in super::super::super) async fn fetch_range_grpc(
        &self,
        client: &mut atipicialfs_v2::object::object_service_client::ObjectServiceClient<Channel>,
        address: &atipicialfs_v2::refs::Address,
        range: AtipicialFsRange,
        auth: &AtipicialFsAuth,
        oracle_key: &KeyPair,
    ) -> (OracleResponseCode, String) {
        let meta = match auth.build_atipicialfs_meta_header() {
            Ok(meta) => meta,
            Err(_) => return (OracleResponseCode::Error, String::new()),
        };
        let body = atipicialfs_v2::object::get_range_request::Body {
            address: Some(address.clone()),
            range: Some(atipicialfs_v2::object::Range {
                offset: range.offset,
                length: range.length,
            }),
            raw: false,
        };
        let verify = match build_atipicialfs_request_verification_header(&body, &meta, oracle_key) {
            Ok(verify) => verify,
            Err(_) => return (OracleResponseCode::Error, String::new()),
        };
        let request = atipicialfs_v2::object::GetRangeRequest {
            body: Some(body),
            meta_header: Some(meta),
            verify_header: Some(verify),
        };

        let response = match client.get_range(request).await {
            Ok(response) => response,
            Err(_) => return (OracleResponseCode::Error, String::new()),
        };
        let mut stream = response.into_inner();
        let total = range.length as usize;
        let mut payload = vec![0u8; total];
        let mut offset = 0usize;

        loop {
            match stream.message().await {
                Ok(Some(mut item)) => {
                    let body = match item.body.take() {
                        Some(body) => body,
                        None => return (OracleResponseCode::Error, String::new()),
                    };
                    if validate_atipicialfs_response(
                        &body,
                        item.meta_header.as_ref(),
                        item.verify_header.as_ref(),
                    )
                    .is_err()
                    {
                        return (OracleResponseCode::Error, String::new());
                    }
                    match body.range_part {
                        Some(atipicialfs_v2::object::get_range_response::body::RangePart::Chunk(
                            chunk,
                        )) => {
                            if offset > total || offset + chunk.len() > total {
                                return (OracleResponseCode::Error, String::new());
                            }
                            payload[offset..offset + chunk.len()].copy_from_slice(&chunk);
                            offset += chunk.len();
                        }
                        Some(atipicialfs_v2::object::get_range_response::body::RangePart::SplitInfo(
                            _,
                        )) => {
                            return (OracleResponseCode::Error, String::new());
                        }
                        None => return (OracleResponseCode::Error, String::new()),
                    }
                }
                Ok(None) => break,
                Err(_) => return (OracleResponseCode::Error, String::new()),
            }
        }

        match String::from_utf8(payload) {
            Ok(text) => (OracleResponseCode::Success, text),
            Err(_) => (OracleResponseCode::Error, String::new()),
        }
    }
}
