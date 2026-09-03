use super::super::super::{AtipicialFsAuth, AtipicialFsCommand, AtipicialFsRequest, OracleAtipicialFsProtocol};
use atipicial_payloads::OracleResponseCode;
use atipicial_payloads::oracle_response::MAX_RESULT_SIZE;

impl OracleAtipicialFsProtocol {
    pub(in super::super::super) async fn execute_request(
        &self,
        endpoint: &str,
        request: AtipicialFsRequest,
        auth: &AtipicialFsAuth,
    ) -> (OracleResponseCode, String) {
        let base = endpoint.trim_end_matches('/');
        let object_url = format!(
            "{}/v1/objects/{}/by_id/{}",
            base, request.container, request.object
        );

        match request.command {
            AtipicialFsCommand::Payload => self.fetch_payload(auth, &object_url).await,
            AtipicialFsCommand::Header => self.fetch_header(auth, &object_url).await,
            AtipicialFsCommand::Range(range) => {
                if range.length > MAX_RESULT_SIZE as u64 {
                    return (OracleResponseCode::ResponseTooLarge, String::new());
                }
                self.fetch_range(auth, &object_url, range).await
            }
            AtipicialFsCommand::Hash(range) => self.fetch_hash(auth, &object_url, range).await,
        }
    }
}
