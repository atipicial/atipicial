use super::super::super::{AtipicialFsAuth, AtipicialFsRange, OracleAtipicialFsProtocol};
use super::super::utils::{map_atipicialfs_status, read_limited_body};
use atipicial_payloads::OracleResponseCode;
use atipicial_payloads::oracle_response::MAX_RESULT_SIZE;

impl OracleAtipicialFsProtocol {
    pub(super) async fn fetch_range(
        &self,
        auth: &AtipicialFsAuth,
        object_url: &str,
        range: AtipicialFsRange,
    ) -> (OracleResponseCode, String) {
        if range.length == 0 {
            return (
                OracleResponseCode::Error,
                "object range is invalid (expected 'Offset|Length')".to_string(),
            );
        }
        let Some(end) = range.offset.checked_add(range.length.saturating_sub(1)) else {
            return (
                OracleResponseCode::Error,
                "object range is invalid (expected 'Offset|Length')".to_string(),
            );
        };
        let builder = match self.request_builder(reqwest::Method::GET, object_url, auth) {
            Ok(builder) => builder,
            Err(_) => return (OracleResponseCode::Error, String::new()),
        };
        let response = match builder
            .header(
                reqwest::header::RANGE,
                format!("bytes={}-{}", range.offset, end),
            )
            .send()
            .await
        {
            Ok(response) => response,
            Err(_) => return (OracleResponseCode::Timeout, String::new()),
        };

        if let Some(code) = map_atipicialfs_status(response.status()) {
            return (code, String::new());
        }

        let body = match read_limited_body(response, MAX_RESULT_SIZE).await {
            Ok(body) => body,
            Err(code) => return (code, String::new()),
        };
        match String::from_utf8(body) {
            Ok(text) => (OracleResponseCode::Success, text),
            Err(_) => (OracleResponseCode::Error, String::new()),
        }
    }
}
