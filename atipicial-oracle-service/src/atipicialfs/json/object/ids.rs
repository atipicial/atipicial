use super::super::super::proto::atipicialfs_v2;

pub fn atipicialfs_json_object_id(id: &atipicialfs_v2::refs::ObjectId) -> Option<String> {
    use base64::Engine as _;
    let value = base64::engine::general_purpose::STANDARD.encode(&id.value);
    Some(format!("{{ \"value\": \"{value}\" }}"))
}

pub fn atipicialfs_json_container_id(id: &atipicialfs_v2::refs::ContainerId) -> Option<String> {
    use base64::Engine as _;
    let value = base64::engine::general_purpose::STANDARD.encode(&id.value);
    Some(format!("{{ \"value\": \"{value}\" }}"))
}

pub fn atipicialfs_json_owner_id(id: &atipicialfs_v2::refs::OwnerId) -> Option<String> {
    use base64::Engine as _;
    let value = base64::engine::general_purpose::STANDARD.encode(&id.value);
    Some(format!("{{ \"value\": \"{value}\" }}"))
}
