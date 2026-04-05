use std::collections::BTreeSet;
use std::fs;

#[path = "support/fixture.rs"]
mod fixture_test_support;

use fixture_test_support::fixture_path;

#[test]
fn distributed_shard_metadata_schema_fixture_has_required_contract_fields() {
    let schema_text = fs::read_to_string(fixture_path("distributed_shard_metadata.schema.json"))
        .expect("read distributed shard metadata schema fixture");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$schema").and_then(|v| v.as_str()),
        Some("https://json-schema.org/draft/2020-12/schema")
    );
    assert_eq!(
        schema.get("$id").and_then(|v| v.as_str()),
        Some("https://afterburner.local/schemas/distributed-shard-metadata/v1")
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
        "shard_id",
        "shard_index",
        "shard_count",
        "owner_worker_id",
        "owner_worker_rank",
        "checksum_algorithm",
        "checksum_sha256",
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

    for key in ["shard_id", "owner_worker_id"] {
        assert_eq!(
            properties
                .get(key)
                .and_then(|v| v.get("type"))
                .and_then(|v| v.as_str()),
            Some("string")
        );
        assert_eq!(
            properties
                .get(key)
                .and_then(|v| v.get("minLength"))
                .and_then(|v| v.as_u64()),
            Some(1)
        );
    }

    for (key, minimum) in [
        ("shard_index", 0_u64),
        ("shard_count", 1_u64),
        ("owner_worker_rank", 0_u64),
    ] {
        assert_eq!(
            properties
                .get(key)
                .and_then(|v| v.get("type"))
                .and_then(|v| v.as_str()),
            Some("integer")
        );
        assert_eq!(
            properties
                .get(key)
                .and_then(|v| v.get("minimum"))
                .and_then(|v| v.as_u64()),
            Some(minimum)
        );
    }

    assert_eq!(
        properties
            .get("checksum_algorithm")
            .and_then(|v| v.get("const"))
            .and_then(|v| v.as_str()),
        Some("sha256")
    );
    assert_eq!(
        properties
            .get("checksum_sha256")
            .and_then(|v| v.get("pattern"))
            .and_then(|v| v.as_str()),
        Some("^[a-f0-9]{64}$")
    );

    let valid_metadata = serde_json::json!({
        "schema_version": "1",
        "shard_id": "train-shard-0003",
        "shard_index": 3,
        "shard_count": 16,
        "owner_worker_id": "worker-07",
        "owner_worker_rank": 7,
        "checksum_algorithm": "sha256",
        "checksum_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
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

    let shard_index = metadata
        .get("shard_index")
        .and_then(|v| v.as_i64())
        .expect("shard_index must be an integer");
    let shard_count = metadata
        .get("shard_count")
        .and_then(|v| v.as_i64())
        .expect("shard_count must be an integer");
    assert!(shard_index >= 0, "shard_index must be non-negative");
    assert!(shard_count > 0, "shard_count must be positive");
    assert!(
        shard_index < shard_count,
        "shard_index must be less than shard_count"
    );
}
