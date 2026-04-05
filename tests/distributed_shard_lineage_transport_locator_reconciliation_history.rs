use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn distributed_shard_lineage_transport_locator_reconciliation_history_schema_and_workflow_are_explicit()
 {
    let schema_text = fs::read_to_string(fixture_path(
        "distributed_shard_lineage_transport_locator_reconciliation_history.schema.json",
    ))
    .expect("read lineage transport locator reconciliation history schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse lineage transport locator reconciliation history schema");

    let entries = schema
        .get("properties")
        .and_then(|value| value.get("entries"))
        .and_then(|value| value.get("items"))
        .and_then(Value::as_object)
        .expect("entries.items must be an object");
    let required = entries
        .get("required")
        .and_then(Value::as_array)
        .expect("entries.items.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("schema.required items must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "event",
        "reconciliation_path",
        "handoff_path",
        "locator_path",
        "reconciliation_status",
        "desired",
        "current",
        "recorded_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains(
            "distributed-shard-lineage-transport-locator-history reconciliation event recorded_at_unix_ms:"
        ),
        "justfile must expose the grouped distributed-shard-lineage-transport-locator history workflow"
    );
}

#[test]
fn distributed_shard_lineage_transport_locator_reconciliation_history_writes_history_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let reconciliation = tmp
        .path()
        .join("distributed_shard_lineage_transport_locator_reconciliation.json");
    fs::write(
        &reconciliation,
        r#"{
  "schema_version": "1",
  "handoff_path": "artifacts/train/distributed_shard_lineage_evidence_handoff.json",
  "locator_path": "artifacts/train/distributed_shard_lineage_transport_locator.json",
  "desired": {
    "shard_id": "train-shard-0003",
    "source": "fineweb-edu/slice",
    "source_revision": "2026-03-22-macbook-100m-tokens",
    "checkpoint_group": "ckpt-group-0007"
  },
  "current": {
    "handoff_path": "artifacts/train/distributed_shard_lineage_evidence_handoff.json",
    "shard_id": "train-shard-0003",
    "source": "fineweb-edu/slice",
    "source_revision": "2026-03-22-macbook-100m-tokens",
    "checkpoint_group": "ckpt-group-0007"
  },
  "reconciliation_status": "aligned"
}"#,
    )
    .expect("write reconciliation");
    let out = tmp
        .path()
        .join("distributed_shard_lineage_transport_locator_reconciliation_history.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("lineage")
        .arg("locator")
        .arg("transport")
        .arg("history")
        .arg("--reconciliation")
        .arg(&reconciliation)
        .arg("--event")
        .arg("reconciled")
        .arg("--recorded-at-unix-ms")
        .arg("1735689800000")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let history_text = fs::read_to_string(&out).expect("read history");
    let mut history: Value = serde_json::from_str(&history_text).expect("parse history");
    let entries = history
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .expect("history entries must be array");
    let entry = entries[0].as_object_mut().expect("entry must be object");
    entry.insert(
        "reconciliation_path".to_string(),
        Value::from("<reconciliation>"),
    );
    assert_eq!(
        history,
        serde_json::json!({
            "schema_version": "1",
            "entries": [
                {
                    "event": "reconciled",
                    "reconciliation_path": "<reconciliation>",
                    "handoff_path": "artifacts/train/distributed_shard_lineage_evidence_handoff.json",
                    "locator_path": "artifacts/train/distributed_shard_lineage_transport_locator.json",
                    "reconciliation_status": "aligned",
                    "desired": {
                        "shard_id": "train-shard-0003",
                        "source": "fineweb-edu/slice",
                        "source_revision": "2026-03-22-macbook-100m-tokens",
                        "checkpoint_group": "ckpt-group-0007"
                    },
                    "current": {
                        "handoff_path": "artifacts/train/distributed_shard_lineage_evidence_handoff.json",
                        "shard_id": "train-shard-0003",
                        "source": "fineweb-edu/slice",
                        "source_revision": "2026-03-22-macbook-100m-tokens",
                        "checkpoint_group": "ckpt-group-0007"
                    },
                    "recorded_at_unix_ms": 1735689800000_u64
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
                == Some(
                    "distributed_shard_lineage_transport_locator_reconciliation_history_written",
                )
        })
        .unwrap_or_else(|| {
            panic!(
                "missing lineage transport locator reconciliation history event in stderr: {stderr}"
            )
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
    fields.insert("history_path".to_string(), Value::from("<history>"));
    let history_obj = fields
        .get_mut("history")
        .and_then(Value::as_object_mut)
        .expect("event history must be an object");
    let entries = history_obj
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .expect("event history entries must be array");
    entries[0]
        .as_object_mut()
        .expect("event history entry must be object")
        .insert(
            "reconciliation_path".to_string(),
            Value::from("<reconciliation>"),
        );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "train_cli",
            "event": "distributed_shard_lineage_transport_locator_reconciliation_history_written",
            "fields": {
                "history_path": "<history>",
                "history": history
            }
        })
    );
}
