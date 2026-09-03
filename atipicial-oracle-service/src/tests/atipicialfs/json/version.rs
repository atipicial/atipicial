#[cfg(feature = "atipicialfs-grpc")]
use super::super::super::json::{atipicialfs_json_object_id, atipicialfs_json_version};
#[cfg(feature = "atipicialfs-grpc")]
use super::super::super::proto::atipicialfs_v2;

#[cfg(feature = "atipicialfs-grpc")]
#[test]
fn atipicialfs_json_empty_message_formats_as_empty_object() {
    let version = atipicialfs_v2::refs::Version { major: 0, minor: 0 };
    let json = atipicialfs_json_version(&version);
    assert_eq!(json, r#"{ "major": 0, "minor": 0 }"#);

    let empty_id = atipicialfs_v2::refs::ObjectId { value: Vec::new() };
    let json = atipicialfs_json_object_id(&empty_id);
    assert_eq!(json, Some(r#"{ "value": "" }"#.to_string()));
}
