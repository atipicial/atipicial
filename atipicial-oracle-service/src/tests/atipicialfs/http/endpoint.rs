use super::super::super::http::normalize_atipicialfs_endpoint;

#[test]
fn normalize_atipicialfs_endpoint_requires_value() {
    assert!(normalize_atipicialfs_endpoint("").is_err());
    assert!(normalize_atipicialfs_endpoint("   ").is_err());
}

#[test]
fn normalize_atipicialfs_endpoint_adds_scheme() {
    let normalized = normalize_atipicialfs_endpoint("127.0.0.1:8080").expect("normalize endpoint");
    assert_eq!(normalized, "http://127.0.0.1:8080");
}

#[test]
fn normalize_atipicialfs_endpoint_preserves_scheme() {
    let normalized = normalize_atipicialfs_endpoint("https://atipicialfs.example").expect("normalize endpoint");
    assert_eq!(normalized, "https://atipicialfs.example");
}
