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
fn distributed_shard_lineage_locator_history_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "distributed_shard_lineage_locator_history.schema.json",
    ))
    .expect("read distributed shard lineage locator history schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse distributed shard lineage locator history schema");

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
        "pointer_path",
        "locator_path",
        "shard_id",
        "source",
        "source_revision",
        "checkpoint_group",
        "recorded_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains(
            "distributed-shard-lineage-locator-history pointer event recorded_at_unix_ms:"
        ),
        "justfile must expose the grouped distributed-shard-lineage-locator history workflow"
    );
}

#[test]
fn distributed_shard_lineage_locator_history_writes_history_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pointer = tmp
        .path()
        .join("distributed_shard_lineage_locator_pointer.json");
    fs::write(
        &pointer,
        r#"{"schema_version":"1","locator_path":"artifacts/train/distributed_shard_lineage_transport_locator.json","shard_id":"shard-0001","source":"fineweb-edu/slice","source_revision":"sample-2026-03-01","checkpoint_group":"group-a"}"#,
    )
    .expect("write pointer");
    let out = tmp
        .path()
        .join("distributed_shard_lineage_locator_history.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("lineage")
        .arg("locator")
        .arg("history")
        .arg("--pointer")
        .arg(&pointer)
        .arg("--event")
        .arg("pointed")
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
    entry.insert("pointer_path".to_string(), Value::from("<pointer>"));
    assert_eq!(
        history,
        serde_json::json!({
            "schema_version": "1",
            "entries": [
                {
                    "event": "pointed",
                    "pointer_path": "<pointer>",
                    "locator_path": "artifacts/train/distributed_shard_lineage_transport_locator.json",
                    "shard_id": "shard-0001",
                    "source": "fineweb-edu/slice",
                    "source_revision": "sample-2026-03-01",
                    "checkpoint_group": "group-a",
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
                == Some("distributed_shard_lineage_locator_history_written")
        })
        .unwrap_or_else(|| {
            panic!("missing distributed shard lineage locator history event in stderr: {stderr}")
        });
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be object");
    fields.insert("history_path".to_string(), Value::from("<history>"));
    let history_obj = fields
        .get_mut("history")
        .and_then(Value::as_object_mut)
        .expect("event history must be object");
    let entries = history_obj
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .expect("event history entries must be array");
    entries[0]
        .as_object_mut()
        .expect("event history entry must be object")
        .insert("pointer_path".to_string(), Value::from("<pointer>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "train_cli",
            "event": "distributed_shard_lineage_locator_history_written",
            "fields": {
                "history_path": "<history>",
                "history": history
            }
        })
    );
}
