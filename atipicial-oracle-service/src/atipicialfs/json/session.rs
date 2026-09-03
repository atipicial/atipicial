#![allow(unused_imports)]

#[cfg(feature = "atipicialfs-grpc")]
mod v1;
#[cfg(feature = "atipicialfs-grpc")]
mod v2;

#[cfg(feature = "atipicialfs-grpc")]
pub(crate) use v1::atipicialfs_json_session_token;
#[cfg(feature = "atipicialfs-grpc")]
pub(crate) use v2::atipicialfs_json_session_token_v2;
