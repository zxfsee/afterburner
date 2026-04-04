use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

#[test]
fn gpu_scheduler_heartbeat_pointer_history_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_heartbeat_pointer_history.schema.json",
    ))
    .expect("read heartbeat pointer history schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse heartbeat pointer history schema");

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
        "heartbeat_path",
        "job_id",
        "lease_id",
        "worker_id",
        "state",
        "observed_at_unix_ms",
        "recorded_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
    assert!(
        entries
            .get("properties")
            .and_then(|value| value.get("progress_marker"))
            .is_some(),
        "history entries must expose optional progress_marker"
    );
}

#[test]
fn gpu_scheduler_heartbeat_pointer_history_writes_history_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pointer = tmp.path().join("gpu_scheduler_heartbeat_pointer.json");
    fs::write(
        &pointer,
        r#"{
  "schema_version": "1",
  "heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.json",
  "job_id": "job-0001",
  "lease_id": "lease-job-0001",
  "worker_id": "worker-0",
  "state": "running",
  "observed_at_unix_ms": 1735689900000,
  "progress_marker": "checkpoint-group-0003/step-128"
}"#,
    )
    .expect("write heartbeat pointer");
    let out = tmp
        .path()
        .join("gpu_scheduler_heartbeat_pointer_history.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("scheduler-heartbeat")
        .arg("pointer")
        .arg("history")
        .arg("--pointer")
        .arg(&pointer)
        .arg("--event")
        .arg("pointed")
        .arg("--recorded-at-unix-ms")
        .arg("1735689950000")
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
                    "heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.json",
                    "job_id": "job-0001",
                    "lease_id": "lease-job-0001",
                    "worker_id": "worker-0",
                    "state": "running",
                    "observed_at_unix_ms": 1735689900000_u64,
                    "progress_marker": "checkpoint-group-0003/step-128",
                    "recorded_at_unix_ms": 1735689950000_u64
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
                == Some("gpu_scheduler_heartbeat_pointer_history_written")
        })
        .unwrap_or_else(|| panic!("missing heartbeat pointer history event in stderr: {stderr}"));
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
            "source": "deploy_cli",
            "event": "gpu_scheduler_heartbeat_pointer_history_written",
            "fields": {
                "history_path": "<history>",
                "history": history
            }
        })
    );
}
