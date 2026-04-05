use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::fixture_path;

#[test]
fn gpu_scheduler_heartbeat_pointer_rollback_schema_is_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_heartbeat_pointer_rollback.schema.json",
    ))
    .expect("read pointer rollback schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse pointer rollback schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/gpu-scheduler-heartbeat-pointer-rollback/v1")
    );
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("required items must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "current_pointer_path",
        "restored_pointer_path",
        "previous_heartbeat_path",
        "restored_heartbeat_path",
        "job_id",
        "lease_id",
        "worker_id",
        "previous_state",
        "restored_state",
        "previous_observed_at_unix_ms",
        "restored_observed_at_unix_ms",
        "rolled_back_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn gpu_scheduler_heartbeat_pointer_rollback_writes_pointer_and_rollback_record() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let current_pointer = tmp.path().join("current-pointer.json");
    let restored_pointer = tmp.path().join("restored-pointer.json");
    fs::write(
        &current_pointer,
        r#"{
  "schema_version": "1",
  "heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.json",
  "job_id": "job-0001",
  "lease_id": "lease-job-0001",
  "worker_id": "worker-0",
  "state": "running",
  "observed_at_unix_ms": 1735689960000,
  "progress_marker": "checkpoint-group-0004/step-256"
}"#,
    )
    .expect("write current pointer");
    fs::write(
        &restored_pointer,
        r#"{
  "schema_version": "1",
  "heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.previous.json",
  "job_id": "job-0001",
  "lease_id": "lease-job-0001",
  "worker_id": "worker-0",
  "state": "checkpointed",
  "observed_at_unix_ms": 1735689900000,
  "progress_marker": "checkpoint-group-0003/step-128"
}"#,
    )
    .expect("write restored pointer");

    let out_pointer = tmp.path().join("pointer-out.json");
    let out_record = tmp.path().join("rollback.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("scheduler-heartbeat")
        .arg("pointer")
        .arg("rollback")
        .arg("--current-pointer")
        .arg(&current_pointer)
        .arg("--restored-pointer")
        .arg(&restored_pointer)
        .arg("--rolled-back-at-unix-ms")
        .arg("1735690100000")
        .arg("--out-pointer")
        .arg(&out_pointer)
        .arg("--out-record")
        .arg(&out_record);
    let assert = cmd.assert().success();

    let pointer_text = fs::read_to_string(&out_pointer).expect("read restored pointer");
    let pointer: Value = serde_json::from_str(&pointer_text).expect("parse restored pointer");
    assert_eq!(
        pointer,
        serde_json::json!({
            "schema_version": "1",
            "heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.previous.json",
            "job_id": "job-0001",
            "lease_id": "lease-job-0001",
            "worker_id": "worker-0",
            "state": "checkpointed",
            "observed_at_unix_ms": 1735689900000_u64,
            "progress_marker": "checkpoint-group-0003/step-128"
        })
    );

    let rollback_text = fs::read_to_string(&out_record).expect("read rollback record");
    let mut rollback: Value = serde_json::from_str(&rollback_text).expect("parse rollback record");
    let object = rollback.as_object_mut().expect("rollback must be object");
    object.insert("current_pointer_path".to_string(), Value::from("<current>"));
    object.insert(
        "restored_pointer_path".to_string(),
        Value::from("<restored>"),
    );
    assert_eq!(
        rollback,
        serde_json::json!({
            "schema_version": "1",
            "current_pointer_path": "<current>",
            "restored_pointer_path": "<restored>",
            "previous_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.json",
            "restored_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.previous.json",
            "job_id": "job-0001",
            "lease_id": "lease-job-0001",
            "worker_id": "worker-0",
            "previous_state": "running",
            "restored_state": "checkpointed",
            "previous_observed_at_unix_ms": 1735689960000_u64,
            "restored_observed_at_unix_ms": 1735689900000_u64,
            "previous_progress_marker": "checkpoint-group-0004/step-256",
            "restored_progress_marker": "checkpoint-group-0003/step-128",
            "rolled_back_at_unix_ms": 1735690100000_u64
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("gpu_scheduler_heartbeat_pointer_rolled_back")
        })
        .unwrap_or_else(|| panic!("missing pointer rollback event in stderr: {stderr}"));
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be object");
    fields.insert("pointer_path".to_string(), Value::from("<pointer-out>"));
    fields.insert("rollback_path".to_string(), Value::from("<rollback-out>"));
    let rollback_obj = fields
        .get_mut("rollback")
        .and_then(Value::as_object_mut)
        .expect("event rollback must be object");
    rollback_obj.insert("current_pointer_path".to_string(), Value::from("<current>"));
    rollback_obj.insert(
        "restored_pointer_path".to_string(),
        Value::from("<restored>"),
    );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "gpu_scheduler_heartbeat_pointer_rolled_back",
            "fields": {
                "pointer_path": "<pointer-out>",
                "rollback_path": "<rollback-out>",
                "pointer": pointer,
                "rollback": rollback
            }
        })
    );
}
