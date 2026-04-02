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
fn gpu_scheduler_heartbeat_pointer_rollback_reconciliation_schema_is_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_heartbeat_pointer_rollback_reconciliation.schema.json",
    ))
    .expect("read pointer rollback reconciliation schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse pointer rollback reconciliation schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/gpu-scheduler-heartbeat-pointer-rollback-reconciliation/v1"
        )
    );
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("required entries must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "rollback_path",
        "current_rollback_path",
        "desired",
        "current",
        "reconciliation_status",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn gpu_scheduler_heartbeat_pointer_rollback_reconciliation_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let rollback = tmp
        .path()
        .join("gpu_scheduler_heartbeat_pointer_rollback.json");
    let current = tmp
        .path()
        .join("gpu_scheduler_heartbeat_pointer_rollback.current.json");
    let rollback_text = r#"{
  "schema_version": "1",
  "current_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.json",
  "restored_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.previous.json",
  "previous_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.json",
  "restored_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.previous.json",
  "job_id": "job-0001",
  "lease_id": "lease-job-0001",
  "worker_id": "worker-0",
  "previous_state": "running",
  "restored_state": "checkpointed",
  "previous_observed_at_unix_ms": 1735689960000,
  "restored_observed_at_unix_ms": 1735689900000,
  "previous_progress_marker": "checkpoint-group-0004/step-256",
  "restored_progress_marker": "checkpoint-group-0003/step-128",
  "rolled_back_at_unix_ms": 1735690100000
}"#;
    fs::write(&rollback, rollback_text).expect("write desired rollback");
    fs::write(&current, rollback_text).expect("write current rollback");
    let out = tmp
        .path()
        .join("gpu_scheduler_heartbeat_pointer_rollback_reconciliation.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("reconcile-scheduler-heartbeat-pointer-rollback")
        .arg("--rollback")
        .arg(&rollback)
        .arg("--current-rollback")
        .arg(&current)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read reconciliation");
    let mut reconciliation: Value = serde_json::from_str(&text).expect("parse reconciliation");
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("rollback_path".to_string(), Value::from("<rollback>"));
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert(
            "current_rollback_path".to_string(),
            Value::from("<current>"),
        );
    let current_obj = reconciliation
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("current object");
    current_obj.insert(
        "current_rollback_path".to_string(),
        Value::from("<current>"),
    );

    let expected_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_heartbeat_pointer_rollback_reconciliation.example.json",
    ))
    .expect("read pointer rollback reconciliation example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse pointer rollback reconciliation example");
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("rollback_path".to_string(), Value::from("<rollback>"));
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert(
            "current_rollback_path".to_string(),
            Value::from("<current>"),
        );
    let expected_current = expected
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("expected current object");
    expected_current.insert(
        "current_rollback_path".to_string(),
        Value::from("<current>"),
    );
    assert_eq!(reconciliation, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("gpu_scheduler_heartbeat_pointer_rollback_reconciliation_written")
        })
        .unwrap_or_else(|| {
            panic!("missing pointer rollback reconciliation event in stderr: {stderr}")
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
    fields.insert(
        "reconciliation_path".to_string(),
        Value::from("<reconciliation>"),
    );
    let reconciliation_obj = fields
        .get_mut("reconciliation")
        .and_then(Value::as_object_mut)
        .expect("event reconciliation must be an object");
    reconciliation_obj.insert("rollback_path".to_string(), Value::from("<rollback>"));
    reconciliation_obj.insert(
        "current_rollback_path".to_string(),
        Value::from("<current>"),
    );
    let event_current = reconciliation_obj
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("event current object");
    event_current.insert(
        "current_rollback_path".to_string(),
        Value::from("<current>"),
    );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "gpu_scheduler_heartbeat_pointer_rollback_reconciliation_written",
            "fields": {
                "reconciliation_path": "<reconciliation>",
                "reconciliation": expected
            }
        })
    );
}
