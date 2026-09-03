use super::super::super::AtipicialFsRange;

#[test]
fn parse_atipicialfs_range_rejects_invalid_format() {
    assert!(AtipicialFsRange::parse_atipicialfs_range("not-a-range").is_err());
    assert!(AtipicialFsRange::parse_atipicialfs_range("10|").is_err());
    assert!(AtipicialFsRange::parse_atipicialfs_range("|10").is_err());
}
