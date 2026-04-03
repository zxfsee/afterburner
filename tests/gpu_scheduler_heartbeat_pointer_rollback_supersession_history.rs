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
fn gpu_scheduler_heartbeat_pointer_rollback_supersession_history_schema_is_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_heartbeat_pointer_rollback_supersession_history.schema.json",
    ))
    .expect("read heartbeat pointer rollback supersession history schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse heartbeat pointer rollback supersession history schema");

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
        "supersession_path",
        "previous_restored_pointer_path",
        "next_restored_pointer_path",
        "previous_restored_heartbeat_path",
        "next_restored_heartbeat_path",
        "job_id",
        "lease_id",
        "worker_id",
        "previous_restored_state",
        "next_restored_state",
        "previous_restored_observed_at_unix_ms",
        "next_restored_observed_at_unix_ms",
        "superseded_at_unix_ms",
        "recorded_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn gpu_scheduler_heartbeat_pointer_rollback_supersession_history_writes_history_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let supersession = tmp
        .path()
        .join("gpu_scheduler_heartbeat_pointer_rollback_supersession.json");
    fs::write(
        &supersession,
        r#"{
  "schema_version": "1",
  "previous_rollback_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback.json",
  "next_rollback_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback.next.json",
  "previous_restored_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.previous.json",
  "next_restored_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.older.json",
  "previous_restored_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.previous.json",
  "next_restored_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.older.json",
  "job_id": "job-0001",
  "lease_id": "lease-job-0001",
  "worker_id": "worker-0",
  "previous_restored_state": "checkpointed",
  "next_restored_state": "rolled-back",
  "previous_restored_observed_at_unix_ms": 1735689900000,
  "next_restored_observed_at_unix_ms": 1735689840000,
  "previous_restored_progress_marker": "checkpoint-group-0003/step-128",
  "next_restored_progress_marker": "checkpoint-group-0002/step-64",
  "superseded_at_unix_ms": 1735690200000
}"#,
    )
    .expect("write supersession");
    let out = tmp
        .path()
        .join("gpu_scheduler_heartbeat_pointer_rollback_supersession_history.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("scheduler-heartbeat")
        .arg("point-record-rollback-supersession-history")
        .arg("--supersession")
        .arg(&supersession)
        .arg("--event")
        .arg("superseded")
        .arg("--recorded-at-unix-ms")
        .arg("1735690210000")
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
        "supersession_path".to_string(),
        Value::from("<supersession>"),
    );
    assert_eq!(
        history,
        serde_json::json!({
            "schema_version": "1",
            "entries": [
                {
                    "event": "superseded",
                    "supersession_path": "<supersession>",
                    "previous_restored_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.previous.json",
                    "next_restored_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.older.json",
                    "previous_restored_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.previous.json",
                    "next_restored_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.older.json",
                    "job_id": "job-0001",
                    "lease_id": "lease-job-0001",
                    "worker_id": "worker-0",
                    "previous_restored_state": "checkpointed",
                    "next_restored_state": "rolled-back",
                    "previous_restored_observed_at_unix_ms": 1735689900000_u64,
                    "next_restored_observed_at_unix_ms": 1735689840000_u64,
                    "previous_restored_progress_marker": "checkpoint-group-0003/step-128",
                    "next_restored_progress_marker": "checkpoint-group-0002/step-64",
                    "superseded_at_unix_ms": 1735690200000_u64,
                    "recorded_at_unix_ms": 1735690210000_u64
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
                == Some("gpu_scheduler_heartbeat_pointer_rollback_supersession_history_written")
        })
        .unwrap_or_else(|| {
            panic!("missing pointer rollback supersession history event in stderr: {stderr}")
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
        .insert(
            "supersession_path".to_string(),
            Value::from("<supersession>"),
        );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "gpu_scheduler_heartbeat_pointer_rollback_supersession_history_written",
            "fields": {
                "history_path": "<history>",
                "history": history
            }
        })
    );
}
