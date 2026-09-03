use super::super::super::json::build_atipicialfs_header_payload;
use super::super::super::{AtipicialFsAuth, OracleAtipicialFsProtocol};
use super::super::utils::map_atipicialfs_status;
use atipicial_payloads::OracleResponseCode;

impl OracleAtipicialFsProtocol {
    pub(super) async fn fetch_header(
        &self,
        auth: &AtipicialFsAuth,
        object_url: &str,
    ) -> (OracleResponseCode, String) {
        let builder = match self.request_builder(reqwest::Method::HEAD, object_url, auth) {
            Ok(builder) => builder,
            Err(_) => return (OracleResponseCode::Error, String::new()),
        };
        let response = match builder.send().await {
            Ok(response) => response,
            Err(_) => return (OracleResponseCode::Timeout, String::new()),
        };

        if let Some(code) = map_atipicialfs_status(response.status()) {
            return (code, String::new());
        }

        let payload = build_atipicialfs_header_payload(response.headers());
        (OracleResponseCode::Success, payload)
    }
}
