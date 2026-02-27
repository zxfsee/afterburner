use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

#[test]
fn serving_rollout_budget_schema_fixture_has_required_contract_fields() {
    let schema_text =
        fs::read_to_string(fixture_path("serving_rollout_budget_metadata.schema.json"))
            .expect("read serving rollout budget schema fixture");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$schema").and_then(|v| v.as_str()),
        Some("https://json-schema.org/draft/2020-12/schema")
    );
    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/serving-rollout-budget-metadata/v1")
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
    let expected = [
        "schema_version",
        "service",
        "latency_budget_ms_p99",
        "error_budget_ratio",
        "window",
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
            .and_then(|v| v.get("const"))
            .and_then(|v| v.as_str()),
        Some("1")
    );
    assert_eq!(
        properties
            .get("service")
            .and_then(|v| v.get("type"))
            .and_then(|v| v.as_str()),
        Some("string")
    );
    assert_eq!(
        properties
            .get("latency_budget_ms_p99")
            .and_then(|v| v.get("type"))
            .and_then(|v| v.as_str()),
        Some("integer")
    );
    assert_eq!(
        properties
            .get("error_budget_ratio")
            .and_then(|v| v.get("type"))
            .and_then(|v| v.as_str()),
        Some("number")
    );
    assert_eq!(
        properties
            .get("window")
            .and_then(|v| v.get("type"))
            .and_then(|v| v.as_str()),
        Some("string")
    );
}
