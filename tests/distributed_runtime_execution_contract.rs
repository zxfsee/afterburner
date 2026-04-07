use std::collections::BTreeSet;
use std::fs;

use serde_json::Value;

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use fixture_test_support::fixture_path;
use repo_test_support::repo_file;

#[test]
fn distributed_runtime_execution_contract_is_documented() {
    let schema_text = fs::read_to_string(fixture_path("distributed_runtime_execution.schema.json"))
        .expect("read execution schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse schema json");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/distributed-runtime-execution/v1")
    );

    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("required values must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "artifact_version",
        "backend",
        "runtime_mode",
        "node_count",
        "world_size",
        "device_group",
        "participant_count",
        "participant_devices",
        "participants",
        "batch_size",
        "num_epochs",
        "train_items_total",
        "valid_items_total",
        "optimizer_strategy",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let architecture = repo_file("ARCHITECTURE.md");
    for needle in [
        "guarded explicit single-node DP train path",
        "distributed_runtime_execution.json",
        "`world_size`, ranks, `device_group`, and",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the DP runtime capability"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "distributed_runtime_execution.json",
        "guarded explicit single-node DP train path",
        "CPU explicit DP",
        "participant-device refs",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must mention `{needle}` as part of the DP runtime capability"
        );
    }
}
