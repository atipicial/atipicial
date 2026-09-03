use super::super::super::proto::atipicialfs_v2;

pub fn build_atipicialfs_object_payload(
    header: &atipicialfs_v2::object::Header,
    data: &[u8],
) -> atipicialfs_v2::object::Object {
    atipicialfs_v2::object::Object {
        object_id: None,
        signature: None,
        header: Some(header.clone()),
        payload: data.to_vec(),
    }
}
