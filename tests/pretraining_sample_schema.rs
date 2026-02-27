use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

#[test]
fn pretraining_sample_schema_fixture_has_required_contract_fields() {
    let schema_text = fs::read_to_string(fixture_path("pretraining_sample_metadata.schema.json"))
        .expect("read pretraining sample schema fixture");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$schema").and_then(|v| v.as_str()),
        Some("https://json-schema.org/draft/2020-12/schema")
    );
    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/pretraining-sample-metadata/v1")
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
    let expected = ["schema_version", "source", "split", "checksum"]
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
            .get("source")
            .and_then(|v| v.get("type"))
            .and_then(|v| v.as_str()),
        Some("string")
    );
    assert_eq!(
        properties
            .get("source")
            .and_then(|v| v.get("minLength"))
            .and_then(|v| v.as_u64()),
        Some(1)
    );
    assert_eq!(
        properties
            .get("split")
            .and_then(|v| v.get("enum"))
            .and_then(|v| v.as_array())
            .map(|values| {
                values
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .expect("schema.properties.split.enum items must be strings")
                            .to_string()
                    })
                    .collect::<BTreeSet<_>>()
            }),
        Some(
            ["train", "validation", "test"]
                .into_iter()
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        )
    );
    assert_eq!(
        properties
            .get("checksum")
            .and_then(|v| v.get("pattern"))
            .and_then(|v| v.as_str()),
        Some("^[a-f0-9]{64}$")
    );
}
