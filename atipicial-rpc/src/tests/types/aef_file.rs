use super::*;
use crate::types::test_fixtures::rpc_case_result;
use atipicial_manifest::MethodToken;

fn sample_aef() -> AefFile {
    AefFile {
        compiler: "atipicial".into(),
        source: "src".into(),
        tokens: vec![MethodToken::default()],
        script: vec![1, 2, 3],
        checksum: 999,
    }
}

#[test]
fn rpc_aef_file_roundtrip() {
    let aef = sample_aef();
    let rpc = RpcAefFile::new(aef.clone());
    let json = rpc.to_json();
    let parsed = RpcAefFile::from_json(&json).expect("aef");
    assert_eq!(parsed.nef_file.compiler, aef.compiler);
    assert_eq!(parsed.nef_file.tokens.len(), aef.tokens.len());
    assert_eq!(parsed.nef_file.script, aef.script);
    assert_eq!(parsed.nef_file.checksum, aef.checksum);
}

#[test]
fn rpc_aef_file_rejects_missing_script() {
    let mut json = JObject::new();
    json.insert("compiler".to_string(), JToken::String("atipicial".into()));
    json.insert("source".to_string(), JToken::String("src".into()));
    json.insert(
        "tokens".to_string(),
        JToken::Array(atipicial_serialization::json::JArray::new()),
    );
    json.insert("checksum".to_string(), JToken::Number(1f64));

    assert!(RpcAefFile::from_json(&json).is_err());
}

#[test]
fn aef_to_json_matches_rpc_test_case() {
    let Some(result) = rpc_case_result("getcontractstateasync") else {
        return;
    };
    let expected = result
        .get("aef")
        .and_then(JToken::as_object)
        .expect("aef result");
    let parsed = RpcAefFile::from_json(expected).expect("parse");
    let actual = parsed.to_json();
    assert_eq!(expected.to_string(), actual.to_string());
}
