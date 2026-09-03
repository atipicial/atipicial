#[cfg(feature = "atipicialfs-grpc")]
mod header;
#[cfg(feature = "atipicialfs-grpc")]
mod ids;
#[cfg(feature = "atipicialfs-grpc")]
mod payload;
#[cfg(feature = "atipicialfs-grpc")]
mod signature;
#[cfg(feature = "atipicialfs-grpc")]
mod version;

#[cfg(feature = "atipicialfs-grpc")]
pub(crate) use header::atipicialfs_json_header;
#[cfg(feature = "atipicialfs-grpc")]
pub(crate) use ids::{atipicialfs_json_container_id, atipicialfs_json_object_id, atipicialfs_json_owner_id};
#[cfg(feature = "atipicialfs-grpc")]
pub(crate) use payload::build_atipicialfs_object_payload;
#[cfg(feature = "atipicialfs-grpc")]
pub(crate) use signature::atipicialfs_json_signature;
#[cfg(feature = "atipicialfs-grpc")]
pub(crate) use version::atipicialfs_json_version;
