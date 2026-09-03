#[cfg(feature = "atipicialfs-grpc")]
// Rationale: this module wraps tonic/prost generated AtipicialFs protobuf code, so
// generated naming, enum shape, documentation, and unused-field lints are
// accepted at the boundary instead of rewritten locally.
#[allow(
    clippy::doc_overindented_list_items,
    clippy::doc_lazy_continuation,
    clippy::large_enum_variant,
    clippy::enum_variant_names,
    clippy::module_inception,
    dead_code
)]
pub(super) mod atipicialfs_proto {
    pub mod atipicial {
        pub mod fs {
            pub mod v2 {
                pub mod object {
                    tonic::include_proto!("atipicial.fs.v2.object");
                }
                pub mod refs {
                    tonic::include_proto!("atipicial.fs.v2.refs");
                }
                pub mod session {
                    tonic::include_proto!("atipicial.fs.v2.session");
                }
                pub mod acl {
                    tonic::include_proto!("atipicial.fs.v2.acl");
                }
                pub mod status {
                    tonic::include_proto!("atipicial.fs.v2.status");
                }
            }
        }
    }
}

#[cfg(feature = "atipicialfs-grpc")]
pub(super) use self::atipicialfs_proto::atipicial::fs::v2 as atipicialfs_v2;
