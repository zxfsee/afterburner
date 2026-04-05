use std::collections::BTreeSet;
use std::fs;

#[path = "support/fixture.rs"]
mod fixture_test_support;

use fixture_test_support::fixture_path;

#[test]
fn framework_adapter_registry_schema_fixture_has_required_contract_fields() {
    let schema_text = fs::read_to_string(fixture_path("framework_adapter_registry.schema.json"))
        .expect("read framework adapter registry schema fixture");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$schema").and_then(|v| v.as_str()),
        Some("https://json-schema.org/draft/2020-12/schema")
    );
    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/framework-adapter-registry/v1")
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
    let expected = ["schema_version", "adapters"]
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

    let supported_pairs = schema
        .get("x-supported-adapter-version-pairs")
        .and_then(|v| v.as_array())
        .expect("schema.x-supported-adapter-version-pairs must be an array");
    assert!(
        !supported_pairs.is_empty(),
        "schema.x-supported-adapter-version-pairs must include at least one pair"
    );

    let mut seen_pairs = BTreeSet::new();
    for pair in supported_pairs {
        let pair = pair
            .as_object()
            .expect("supported adapter/version pair entries must be objects");
        let pair_fields = pair.keys().cloned().collect::<BTreeSet<_>>();
        let expected_pair_fields = ["adapter", "version"]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            pair_fields, expected_pair_fields,
            "supported pair entries must contain only adapter/version fields"
        );

        let adapter = pair
            .get("adapter")
            .and_then(|v| v.as_str())
            .expect("supported pair adapter must be a string");
        assert!(
            !adapter.is_empty(),
            "supported pair adapter values must not be empty"
        );
        let version = pair
            .get("version")
            .and_then(|v| v.as_str())
            .expect("supported pair version must be a string");
        assert!(
            !version.is_empty(),
            "supported pair version values must not be empty"
        );
        assert!(
            seen_pairs.insert((adapter.to_string(), version.to_string())),
            "duplicate supported adapter/version pair: {adapter}:{version}"
        );
    }

    let adapters = schema
        .get("properties")
        .and_then(|v| v.get("adapters"))
        .expect("schema.properties.adapters must exist");
    assert_eq!(adapters.get("type").and_then(|v| v.as_str()), Some("array"));
    assert_eq!(adapters.get("minItems").and_then(|v| v.as_u64()), Some(1));

    let item_schema = adapters
        .get("items")
        .expect("schema.properties.adapters.items must exist");
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
        .expect("adapter item required must be an array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("adapter item required values must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected_item_required = ["adapter", "version"]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(item_required, expected_item_required);

    let item_properties = item_schema
        .get("properties")
        .expect("adapter item properties must exist");
    assert_eq!(
        item_properties
            .get("adapter")
            .and_then(|v| v.get("type"))
            .and_then(|v| v.as_str()),
        Some("string")
    );
    assert_eq!(
        item_properties
            .get("version")
            .and_then(|v| v.get("type"))
            .and_then(|v| v.as_str()),
        Some("string")
    );
}
