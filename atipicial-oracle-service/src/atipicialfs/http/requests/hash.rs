use super::super::super::{AtipicialFsAuth, AtipicialFsRange, OracleAtipicialFsProtocol};
use super::super::utils::{hash_response_body, map_atipicialfs_status};
use atipicial_payloads::OracleResponseCode;

impl OracleAtipicialFsProtocol {
    pub(super) async fn fetch_hash(
        &self,
        auth: &AtipicialFsAuth,
        object_url: &str,
        range: Option<AtipicialFsRange>,
    ) -> (OracleResponseCode, String) {
        let builder = if let Some(range) = range {
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
            builder.header(
                reqwest::header::RANGE,
                format!("bytes={}-{}", range.offset, end),
            )
        } else {
            match self.request_builder(reqwest::Method::GET, object_url, auth) {
                Ok(builder) => builder,
                Err(_) => return (OracleResponseCode::Error, String::new()),
            }
        };

        let response = match builder.send().await {
            Ok(response) => response,
            Err(_) => return (OracleResponseCode::Timeout, String::new()),
        };

        if let Some(code) = map_atipicialfs_status(response.status()) {
            return (code, String::new());
        }

        let hash = match hash_response_body(response).await {
            Ok(hash) => hash,
            Err(code) => return (code, String::new()),
        };
        (OracleResponseCode::Success, format!("\"{}\"", hash))
    }
}
