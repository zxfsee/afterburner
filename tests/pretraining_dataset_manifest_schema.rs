use afterburner::data::{
    PRETRAINING_DATASET_MANIFEST_REQUIRED_FIELDS, PRETRAINING_DATASET_MANIFEST_SCHEMA_VERSION,
    PRETRAINING_SAMPLE_METADATA_CHECKSUM_PATTERN, PRETRAINING_SAMPLE_METADATA_SPLIT_VALUES,
};
use std::collections::BTreeSet;
use std::fs;

mod support;

use support::fixture_path;

#[test]
fn pretraining_dataset_manifest_schema_fixture_has_required_contract_fields() {
    let schema_text = fs::read_to_string(fixture_path("pretraining_dataset_manifest.schema.json"))
        .expect("read pretraining dataset manifest schema fixture");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$schema").and_then(|v| v.as_str()),
        Some("https://json-schema.org/draft/2020-12/schema")
    );
    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/pretraining-dataset-manifest/v1")
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
    let expected = PRETRAINING_DATASET_MANIFEST_REQUIRED_FIELDS
        .into_iter()
        .map(|field| field.to_string())
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
        Some(PRETRAINING_DATASET_MANIFEST_SCHEMA_VERSION)
    );
    assert_eq!(
        properties
            .get("checksum_rollup_sha256")
            .and_then(|v| v.get("pattern"))
            .and_then(|v| v.as_str()),
        Some(PRETRAINING_SAMPLE_METADATA_CHECKSUM_PATTERN)
    );
    assert_eq!(
        properties
            .get("shards")
            .and_then(|v| v.get("items"))
            .and_then(|v| v.get("properties"))
            .and_then(|v| v.get("split"))
            .and_then(|v| v.get("enum"))
            .and_then(|v| v.as_array())
            .map(|values| {
                values
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .expect("schema shard split enum items must be strings")
                            .to_string()
                    })
                    .collect::<BTreeSet<_>>()
            }),
        Some(
            PRETRAINING_SAMPLE_METADATA_SPLIT_VALUES
                .into_iter()
                .map(|value| value.to_string())
                .collect::<BTreeSet<_>>()
        )
    );
}
