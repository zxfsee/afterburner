use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use fixture_test_support::fixture_path;
use repo_test_support::repo_file;

#[test]
fn distributed_shard_lineage_evidence_bundle_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "distributed_shard_lineage_evidence_bundle.schema.json",
    ))
    .expect("read lineage evidence bundle schema fixture");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse lineage evidence bundle schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/distributed-shard-lineage-evidence-bundle/v1")
    );
    let required = schema
        .get("required")
        .and_then(Value::as_array)
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
        "receipt_path",
        "shard_id",
        "source",
        "source_revision",
        "checkpoint_group",
        "evidence_count",
        "evidence_sources",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("distributed-shard-lineage-bundle receipt:"),
        "justfile must expose the grouped distributed-shard-lineage-bundle workflow"
    );
}

#[test]
fn distributed_shard_lineage_evidence_bundle_writes_bundle_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let receipt = tmp.path().join("distributed_shard_lineage_receipt.json");
    fs::write(
        &receipt,
        r#"{
  "schema_version": "1",
  "shard_id": "train-shard-0003",
  "source": "fineweb-edu/slice",
  "source_revision": "2026-03-22-macbook-100m-tokens",
  "checkpoint_group": "ckpt-group-0007",
  "checked_at_unix_ms": 1735689600000,
  "evidence_sources": [
    {
      "metadata_path": "fixtures/distributed_shard_metadata.example.json",
      "checkpoint_root": "artifacts/train/checkpoints/group-0007",
      "observed_at_unix_ms": 1735689600000
    }
  ]
}"#,
    )
    .expect("write lineage receipt");

    let out = tmp
        .path()
        .join("distributed_shard_lineage_evidence_bundle.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("lineage")
        .arg("bundle")
        .arg("--receipt")
        .arg(&receipt)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read lineage evidence bundle");
    let mut bundle: Value = serde_json::from_str(&text).expect("parse lineage evidence bundle");
    bundle
        .as_object_mut()
        .expect("bundle must be object")
        .insert("receipt_path".to_string(), Value::from("<receipt>"));
    assert_eq!(
        bundle,
        serde_json::json!({
            "schema_version": "1",
            "receipt_path": "<receipt>",
            "shard_id": "train-shard-0003",
            "source": "fineweb-edu/slice",
            "source_revision": "2026-03-22-macbook-100m-tokens",
            "checkpoint_group": "ckpt-group-0007",
            "evidence_count": 1,
            "evidence_sources": [
                {
                    "metadata_path": "fixtures/distributed_shard_metadata.example.json",
                    "checkpoint_root": "artifacts/train/checkpoints/group-0007",
                    "observed_at_unix_ms": 1735689600000i64
                }
            ]
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("distributed_shard_lineage_evidence_bundle_written")
        })
        .unwrap_or_else(|| panic!("missing lineage evidence bundle event in stderr: {stderr}"));
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("bundle_path".to_string(), Value::from("<bundle>"));
    let bundle_obj = fields
        .get_mut("bundle")
        .and_then(Value::as_object_mut)
        .expect("event bundle must be an object");
    bundle_obj.insert("receipt_path".to_string(), Value::from("<receipt>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "train_cli",
            "event": "distributed_shard_lineage_evidence_bundle_written",
            "fields": {
                "bundle_path": "<bundle>",
                "bundle": bundle
            }
        })
    );
}
