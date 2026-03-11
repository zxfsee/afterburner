use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde_json::json;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

#[test]
fn artifact_rollout_ownership_schema_and_adr_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path("artifact_rollout_ownership.schema.json"))
        .expect("read artifact rollout ownership schema fixture");
    let schema: serde_json::Value =
        serde_json::from_str(&schema_text).expect("parse schema fixture");

    assert_eq!(
        schema.get("$schema").and_then(|value| value.as_str()),
        Some("https://json-schema.org/draft/2020-12/schema")
    );
    assert_eq!(
        schema.get("$id").and_then(|value| value.as_str()),
        Some("https://afterburner.local/schemas/artifact-rollout-ownership/v1")
    );
    assert_eq!(
        schema.get("type").and_then(|value| value.as_str()),
        Some("object")
    );
    assert_eq!(
        schema
            .get("additionalProperties")
            .and_then(|value| value.as_bool()),
        Some(false)
    );

    let required = schema
        .get("required")
        .and_then(|value| value.as_array())
        .expect("schema.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("schema.required items must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "artifact_version",
        "artifact_manifest",
        "provenance_source",
        "rollout_owner",
        "approved_by",
        "approved_operations",
        "approval_ticket",
        "approved_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let properties = schema
        .get("properties")
        .expect("schema.properties must be present");
    assert_eq!(
        properties
            .get("schema_version")
            .and_then(|value| value.get("const"))
            .and_then(|value| value.as_str()),
        Some("1")
    );
    assert_eq!(
        properties
            .get("approved_operations")
            .and_then(|value| value.get("type"))
            .and_then(|value| value.as_str()),
        Some("array")
    );
    let approved_operations_enum = properties
        .get("approved_operations")
        .and_then(|value| value.get("items"))
        .and_then(|value| value.get("enum"))
        .and_then(|value| value.as_array())
        .expect("approved_operations items must define an enum");
    let approved_operations_enum = approved_operations_enum
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("approved_operations enum values must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected_operations = ["upload", "promote", "deploy"]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(approved_operations_enum, expected_operations);

    let sample = json!({
        "schema_version": "1",
        "artifact_version": "0.1.0",
        "artifact_manifest": "artifacts/inference/0.1.0/manifest.toml",
        "provenance_source": "artifacts/train/train_done.jsonl",
        "rollout_owner": "ml-release",
        "approved_by": "ops-review",
        "approved_operations": ["upload", "promote", "deploy"],
        "approval_ticket": "CHG-4242",
        "approved_at_unix_ms": 1735689600000_u64
    });
    let sample = sample.as_object().expect("sample must be a json object");
    for key in expected {
        assert!(
            sample.contains_key(key.as_str()),
            "sample artifact rollout ownership must include `{key}`"
        );
    }

    let architecture =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ARCHITECTURE.md"))
            .expect("read architecture");
    let adr = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("docs/adr/009-artifact-rollout-ownership.md"),
    )
    .expect("read artifact rollout ownership ADR");

    assert!(
        architecture.contains("ADR-009"),
        "architecture decisions index must link ADR-009"
    );
    assert!(
        adr.contains("artifact rollout ownership"),
        "ADR must describe the artifact rollout ownership contract"
    );
    assert!(
        adr.contains("approved_operations"),
        "ADR must document the shared approval scope field"
    );
}
