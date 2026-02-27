use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

#[test]
fn calibration_artifact_schema_fixture_parses_and_validates_contract() {
    let schema_text = fs::read_to_string(fixture_path("calibration_artifact_metadata.schema.json"))
        .expect("read calibration schema fixture");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$schema").and_then(|v| v.as_str()),
        Some("https://json-schema.org/draft/2020-12/schema")
    );
    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/calibration-artifact-metadata/v1")
    );
    assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("object"));
    assert_eq!(
        schema
            .get("additionalProperties")
            .and_then(|v| v.as_bool()),
        Some(false)
    );

    let required = schema
        .get("required")
        .and_then(|v| v.as_array())
        .expect("schema.required must be an array");
    let required = required
        .iter()
        .map(|v| {
            v.as_str()
                .expect("schema.required items must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "calibration_artifact",
        "artifact_version",
        "method",
        "created_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    assert_eq!(
        schema
            .get("properties")
            .and_then(|v| v.get("schema_version"))
            .and_then(|v| v.get("const"))
            .and_then(|v| v.as_str()),
        Some("1")
    );

    let valid_metadata = serde_json::json!({
        "schema_version": "1",
        "calibration_artifact": "artifacts/calibration/default.calib",
        "artifact_version": "v1",
        "method": "temperature-scaling",
        "created_at_unix_ms": 1_735_689_600_000_i64
    });
    let metadata = valid_metadata
        .as_object()
        .expect("valid_metadata must be an object");

    for key in &expected {
        assert!(
            metadata.contains_key(key),
            "required field {key} missing from sample metadata"
        );
    }

    let schema_version = metadata
        .get("schema_version")
        .and_then(|v| v.as_str())
        .expect("schema_version must be a string");
    assert_eq!(schema_version, "1");

    for string_key in ["calibration_artifact", "artifact_version", "method"] {
        let value = metadata
            .get(string_key)
            .and_then(|v| v.as_str())
            .expect("string fields must be strings");
        assert!(
            !value.is_empty(),
            "string field {string_key} must be non-empty"
        );
    }

    let created_at = metadata
        .get("created_at_unix_ms")
        .and_then(|v| v.as_i64())
        .expect("created_at_unix_ms must be an integer");
    assert!(created_at >= 0, "created_at_unix_ms must be non-negative");
}
