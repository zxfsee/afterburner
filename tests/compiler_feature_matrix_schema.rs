use std::collections::BTreeSet;
use std::fs;

mod support;

use support::fixture_path;

#[test]
fn compiler_feature_matrix_schema_fixture_has_required_contract_fields() {
    let schema_text = fs::read_to_string(fixture_path("compiler_feature_matrix.schema.json"))
        .expect("read compiler feature matrix schema fixture");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$schema").and_then(|v| v.as_str()),
        Some("https://json-schema.org/draft/2020-12/schema")
    );
    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/compiler-feature-matrix/v1")
    );
    assert_eq!(schema.get("type").and_then(|v| v.as_str()), Some("object"));
    assert_eq!(
        schema.get("additionalProperties").and_then(|v| v.as_bool()),
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
    let expected = ["schema_version", "backend_feature_matrix"]
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

    let matrix = schema
        .get("properties")
        .and_then(|v| v.get("backend_feature_matrix"))
        .expect("schema.properties.backend_feature_matrix must exist");
    assert_eq!(matrix.get("type").and_then(|v| v.as_str()), Some("array"));
    assert_eq!(matrix.get("minItems").and_then(|v| v.as_u64()), Some(1));

    let item_schema = matrix
        .get("items")
        .expect("schema.properties.backend_feature_matrix.items must exist");
    assert_eq!(
        item_schema.get("type").and_then(|v| v.as_str()),
        Some("object")
    );
    assert_eq!(
        item_schema
            .get("additionalProperties")
            .and_then(|v| v.as_bool()),
        Some(false)
    );

    let item_required = item_schema
        .get("required")
        .and_then(|v| v.as_array())
        .expect("matrix item required must be an array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("matrix item required values must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected_item_required = ["backend", "enabled_features"]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(item_required, expected_item_required);

    let item_properties = item_schema
        .get("properties")
        .expect("matrix item properties must exist");
    assert_eq!(
        item_properties
            .get("backend")
            .and_then(|v| v.get("type"))
            .and_then(|v| v.as_str()),
        Some("string")
    );
    assert_eq!(
        item_properties
            .get("enabled_features")
            .and_then(|v| v.get("type"))
            .and_then(|v| v.as_str()),
        Some("array")
    );
    assert_eq!(
        item_properties
            .get("enabled_features")
            .and_then(|v| v.get("minItems"))
            .and_then(|v| v.as_u64()),
        Some(1)
    );

    let valid_matrix = serde_json::json!({
        "schema_version": "1",
        "backend_feature_matrix": [
            {
                "backend": "cpu",
                "enabled_features": ["f32", "avx2"]
            },
            {
                "backend": "cuda",
                "enabled_features": ["f16", "tensor-cores"]
            }
        ]
    });
    let matrix_obj = valid_matrix
        .as_object()
        .expect("valid_matrix must be an object");
    for key in &expected {
        assert!(
            matrix_obj.contains_key(key),
            "required field {key} missing from sample matrix"
        );
    }
}
