use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn distributed_shard_lineage_transport_locator_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "distributed_shard_lineage_transport_locator.schema.json",
    ))
    .expect("read distributed shard lineage transport locator schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse distributed shard lineage transport locator schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/distributed-shard-lineage-transport-locator/v1")
    );
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("required entries must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "handoff_path",
        "shard_id",
        "source",
        "source_revision",
        "checkpoint_group",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("distributed-shard-lineage-transport-locator handoff:"),
        "justfile must expose the grouped distributed-shard-lineage-transport-locator workflow"
    );
}

#[test]
fn distributed_shard_lineage_transport_locator_writes_locator_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handoff = tmp
        .path()
        .join("distributed_shard_lineage_evidence_handoff.json");
    fs::write(
        &handoff,
        r#"{
  "schema_version": "1",
  "bundle_path": "artifacts/train/distributed_shard_lineage_evidence_bundle.json",
  "shard_id": "train-shard-0003",
  "source": "fineweb-edu/slice",
  "source_revision": "2026-03-22-macbook-100m-tokens",
  "checkpoint_group": "ckpt-group-0007",
  "evidence_count": 1
}"#,
    )
    .expect("write distributed shard lineage evidence handoff");

    let out = tmp
        .path()
        .join("distributed_shard_lineage_transport_locator.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("lineage")
        .arg("locator")
        .arg("transport")
        .arg("point")
        .arg("--handoff")
        .arg(&handoff)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let locator_text = fs::read_to_string(&out).expect("read lineage transport locator");
    let mut locator: Value =
        serde_json::from_str(&locator_text).expect("parse lineage transport locator");
    locator
        .as_object_mut()
        .expect("locator must be a top-level object")
        .insert("handoff_path".to_string(), Value::from("<handoff>"));
    let expected_text = fs::read_to_string(fixture_path(
        "distributed_shard_lineage_transport_locator.example.json",
    ))
    .expect("read distributed shard lineage transport locator example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse distributed shard lineage transport locator example");
    expected
        .as_object_mut()
        .expect("expected locator must be a top-level object")
        .insert("handoff_path".to_string(), Value::from("<handoff>"));
    assert_eq!(locator, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("distributed_shard_lineage_transport_locator_written")
        })
        .unwrap_or_else(|| {
            panic!("missing distributed shard lineage transport locator event in stderr: {stderr}")
        });
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("locator_path".to_string(), Value::from("<locator>"));
    let locator_obj = fields
        .get_mut("locator")
        .and_then(Value::as_object_mut)
        .expect("event locator must be an object");
    locator_obj.insert("handoff_path".to_string(), Value::from("<handoff>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "train_cli",
            "event": "distributed_shard_lineage_transport_locator_written",
            "fields": {
                "locator_path": "<locator>",
                "locator": expected
            }
        })
    );
}
