use super::*;
use atipicial_io::SerializableExtensions;

#[test]
fn magic_constant_matches_atipicial_spec() {
    // 0x3346454E = 'N','E','F',3 in little-endian
    assert_eq!(AefFile::MAGIC, 0x3346_454E);
}

#[test]
fn new_computes_checksum() {
    let aef = AefFile::new("atipicial-core-v0.0.0".to_string(), vec![0x40]); // RET
    assert_ne!(aef.checksum, 0);
}

#[test]
fn try_compute_checksum_matches_compatibility_checksum() {
    let aef = AefFile::new("atipicial-core-v0.0.0".to_string(), vec![0x40]); // RET
    assert_eq!(
        AefFile::try_compute_checksum(&aef).expect("checksum"),
        AefFile::compute_checksum(&aef)
    );
}

#[test]
fn default_has_zero_checksum() {
    let aef = AefFile::default();
    assert_eq!(aef.checksum, 0);
}

#[test]
fn new_constructor_stores_fields() {
    let aef = AefFile::new("compiler".to_string(), vec![1, 2, 3, 4]);
    assert_eq!(aef.compiler, "compiler");
    assert_eq!(aef.script, vec![1, 2, 3, 4]);
    assert!(aef.tokens.is_empty());
    assert!(aef.source.is_empty());
}

#[test]
fn size_includes_all_fields() {
    let aef = AefFile::new("c".to_string(), vec![0; 100]);
    // 4 (magic) + 64 (compiler fixed) + 1 (source var int) + 1 (reserved)
    // + 1 (tokens var int) + 2 (reserved u16) + 2 (script var int) + 4 (checksum)
    // + the actual bytes
    let size = aef.size();
    assert!(size > 4 + 64 + 1 + 1 + 1 + 2 + 2 + 4);
}

#[test]
fn try_to_bytes_matches_existing_bytes_and_serializable_wire() {
    let aef = AefFile::new("atipicial-core-v0.0.0".to_string(), vec![0x40]); // RET
    assert_eq!(aef.try_to_bytes().expect("fallible bytes"), aef.to_bytes());
    assert_eq!(
        aef.try_to_bytes().expect("fallible bytes"),
        aef.to_array().expect("serializable bytes")
    );
}

#[test]
fn from_bytes_rejects_bad_magic() {
    let bytes = vec![0xFF; 100];
    let result = AefFile::from_bytes(&bytes);
    assert!(result.is_err());
}
