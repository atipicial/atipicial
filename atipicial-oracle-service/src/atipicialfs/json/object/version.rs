use super::super::super::proto::atipicialfs_v2;

pub fn atipicialfs_json_version(version: &atipicialfs_v2::refs::Version) -> String {
    format!(
        "{{ \"major\": {}, \"minor\": {} }}",
        version.major, version.minor
    )
}
