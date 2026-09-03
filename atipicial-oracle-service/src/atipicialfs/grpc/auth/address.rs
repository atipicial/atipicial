use super::super::super::AtipicialFsRequest;
use super::super::super::decode_raw_base58;
use super::super::super::proto::atipicialfs_v2;
use atipicial_error::{CoreError, CoreResult};

impl AtipicialFsRequest {
    pub(crate) fn build_atipicialfs_grpc_address(&self) -> CoreResult<atipicialfs_v2::refs::Address> {
        let container = decode_atipicialfs_id_bytes(&self.container, 32)?;
        let object = decode_atipicialfs_id_bytes(&self.object, 32)?;
        Ok(atipicialfs_v2::refs::Address {
            container_id: Some(atipicialfs_v2::refs::ContainerId { value: container }),
            object_id: Some(atipicialfs_v2::refs::ObjectId { value: object }),
        })
    }
}

fn decode_atipicialfs_id_bytes(value: &str, expected_len: usize) -> CoreResult<Vec<u8>> {
    decode_raw_base58(value, Some(expected_len)).ok_or_else(|| CoreError::other("invalid atipicialfs id"))
}
