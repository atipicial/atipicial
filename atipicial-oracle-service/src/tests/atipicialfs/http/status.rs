use super::super::super::http::map_atipicialfs_status;
use atipicial_payloads::OracleResponseCode;
use reqwest::StatusCode;

#[test]
fn map_atipicialfs_status_codes() {
    assert!(map_atipicialfs_status(StatusCode::OK).is_none());
    assert!(map_atipicialfs_status(StatusCode::PARTIAL_CONTENT).is_none());
    assert_eq!(
        map_atipicialfs_status(StatusCode::NOT_FOUND),
        Some(OracleResponseCode::NotFound)
    );
    assert_eq!(
        map_atipicialfs_status(StatusCode::FORBIDDEN),
        Some(OracleResponseCode::Forbidden)
    );
    assert_eq!(
        map_atipicialfs_status(StatusCode::REQUEST_TIMEOUT),
        Some(OracleResponseCode::Timeout)
    );
    assert_eq!(
        map_atipicialfs_status(StatusCode::GATEWAY_TIMEOUT),
        Some(OracleResponseCode::Timeout)
    );
    assert_eq!(
        map_atipicialfs_status(StatusCode::BAD_REQUEST),
        Some(OracleResponseCode::Error)
    );
}
