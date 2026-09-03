use super::super::super::AtipicialFsCommand;
use super::super::super::AtipicialFsRequest;

fn sample_atipicialfs_id(byte: u8) -> String {
    bs58::encode([byte; 32]).into_string()
}

fn error_message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[test]
fn parse_atipicialfs_request_payload() {
    let container = sample_atipicialfs_id(1);
    let object = sample_atipicialfs_id(2);
    let request = AtipicialFsRequest::parse_atipicialfs_request(&format!("atipicialfs:{}/{}", container, object))
        .expect("parse payload");
    assert_eq!(request.container, container);
    assert_eq!(request.object, object);
    assert!(matches!(request.command, AtipicialFsCommand::Payload));
}

#[test]
fn parse_atipicialfs_request_rejects_authority_urls() {
    let container = sample_atipicialfs_id(1);
    let object = sample_atipicialfs_id(2);
    let err = AtipicialFsRequest::parse_atipicialfs_request(&format!("atipicialfs://{}/{}", container, object))
        .expect_err("authority-style URL should fail");
    assert!(error_message(err).contains("Invalid atipicialfs url"));
}

#[test]
fn parse_atipicialfs_request_rejects_double_slash() {
    let container = sample_atipicialfs_id(1);
    let object = sample_atipicialfs_id(2);
    let err = AtipicialFsRequest::parse_atipicialfs_request(&format!("atipicialfs:{container}//{object}"))
        .expect_err("double slash should fail");
    assert!(error_message(err).contains("Invalid atipicialfs url"));
}

#[test]
fn parse_atipicialfs_request_range() {
    let container = sample_atipicialfs_id(1);
    let object = sample_atipicialfs_id(2);
    let request =
        AtipicialFsRequest::parse_atipicialfs_request(&format!("atipicialfs:{}/{}/range/10|20", container, object))
            .expect("parse range");
    assert_eq!(request.container, container);
    assert_eq!(request.object, object);
    match request.command {
        AtipicialFsCommand::Range(range) => {
            assert_eq!(range.offset, 10);
            assert_eq!(range.length, 20);
        }
        _ => panic!("expected range command"),
    }
}

#[test]
fn parse_atipicialfs_request_range_percent_decoded() {
    let container = sample_atipicialfs_id(1);
    let object = sample_atipicialfs_id(2);
    let request =
        AtipicialFsRequest::parse_atipicialfs_request(&format!("atipicialfs:{}/{}/range/10%7C20", container, object))
            .expect("parse range");
    match request.command {
        AtipicialFsCommand::Range(range) => {
            assert_eq!(range.offset, 10);
            assert_eq!(range.length, 20);
        }
        _ => panic!("expected range command"),
    }
}

#[test]
fn parse_atipicialfs_request_header_and_hash() {
    let container = sample_atipicialfs_id(1);
    let object = sample_atipicialfs_id(2);
    let header =
        AtipicialFsRequest::parse_atipicialfs_request(&format!("atipicialfs:{}/{}/header", container, object))
            .expect("parse header");
    assert!(matches!(header.command, AtipicialFsCommand::Header));

    let hash = AtipicialFsRequest::parse_atipicialfs_request(&format!("atipicialfs:{}/{}/hash", container, object))
        .expect("parse hash");
    match hash.command {
        AtipicialFsCommand::Hash(None) => {}
        _ => panic!("expected hash without range"),
    }

    let hash_range =
        AtipicialFsRequest::parse_atipicialfs_request(&format!("atipicialfs:{}/{}/hash/5|7", container, object))
            .expect("parse hash range");
    match hash_range.command {
        AtipicialFsCommand::Hash(Some(range)) => {
            assert_eq!(range.offset, 5);
            assert_eq!(range.length, 7);
        }
        _ => panic!("expected hash with range"),
    }
}

#[test]
fn parse_atipicialfs_request_ignores_query_fragment() {
    let container = sample_atipicialfs_id(1);
    let object = sample_atipicialfs_id(2);
    let request = AtipicialFsRequest::parse_atipicialfs_request(&format!(
        "atipicialfs:{}/{}/header?foo=1#bar",
        container, object
    ))
    .expect("parse header with query");
    assert!(matches!(request.command, AtipicialFsCommand::Header));
    assert_eq!(request.container, container);
    assert_eq!(request.object, object);
}

#[test]
fn parse_atipicialfs_request_missing_range_errors() {
    let container = sample_atipicialfs_id(1);
    let object = sample_atipicialfs_id(2);
    let err = AtipicialFsRequest::parse_atipicialfs_request(&format!("atipicialfs:{}/{}/range", container, object))
        .expect_err("range should error");
    let err = error_message(err);
    assert!(
        err.contains("missing object range"),
        "unexpected error: {err}"
    );
}

#[test]
fn parse_atipicialfs_request_rejects_invalid_ids() {
    let object = sample_atipicialfs_id(2);
    let err =
        AtipicialFsRequest::parse_atipicialfs_request(&format!("atipicialfs:0/{}", object)).expect_err("invalid id");
    let err = error_message(err);
    assert!(
        err.contains("invalid atipicialfs container id"),
        "unexpected error: {err}"
    );
}
