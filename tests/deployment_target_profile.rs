use std::fs;
use std::path::PathBuf;

use serde_json::Value;

#[path = "support/fixture.rs"]
mod fixture_test_support;

use fixture_test_support::fixture_path;

#[test]
fn deployment_target_profile_schema_and_adr_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path("deployment_target_profile.schema.json"))
        .expect("read deployment target profile schema fixture");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse schema fixture");
    let schema = schema
        .as_object()
        .expect("deployment target profile schema must be a json object");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/deployment-target-profile/v1")
    );
    assert_eq!(schema.get("type").and_then(Value::as_str), Some("object"));

    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be an array");
    for key in [
        "schema_version",
        "profile_name",
        "deploy_hostname",
        "ssh_user",
        "system",
        "artifact_root",
        "activation_strategy",
    ] {
        assert!(
            required.iter().any(|value| value.as_str() == Some(key)),
            "schema.required must include `{key}`"
        );
    }

    let properties = schema
        .get("properties")
        .and_then(Value::as_object)
        .expect("schema.properties must be an object");
    assert_eq!(
        properties
            .get("schema_version")
            .and_then(|value| value.get("const"))
            .and_then(Value::as_str),
        Some("1")
    );

    let sample_text = fs::read_to_string(fixture_path("deployment_target_profile.example.json"))
        .expect("read deployment target profile example fixture");
    let sample: Value = serde_json::from_str(&sample_text)
        .expect("parse deployment target profile example fixture");

    let sample = sample.as_object().expect("sample must be object");
    for key in [
        "schema_version",
        "profile_name",
        "deploy_hostname",
        "ssh_user",
        "system",
        "artifact_root",
        "activation_strategy",
    ] {
        assert!(
            sample.contains_key(key),
            "sample deployment target profile must include `{key}`"
        );
    }

    let architecture =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ARCHITECTURE.md"))
            .expect("read architecture");
    let adr = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/adr/008-deployment-target-profile.md"),
    )
    .expect("read deployment target profile ADR");

    assert!(
        architecture.contains("ADR-008"),
        "architecture decisions index must link ADR-008"
    );
    assert!(
        adr.contains("deployment target profile"),
        "ADR must describe the deployment target profile contract"
    );
    assert!(
        adr.contains("deploy-rs"),
        "ADR must explain the deploy-rs relationship"
    );
}
