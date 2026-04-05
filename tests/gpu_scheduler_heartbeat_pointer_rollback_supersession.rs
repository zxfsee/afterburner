use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::fixture_path;

#[test]
fn gpu_scheduler_heartbeat_pointer_rollback_supersession_schema_is_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_heartbeat_pointer_rollback_supersession.schema.json",
    ))
    .expect("read heartbeat pointer rollback supersession schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse heartbeat pointer rollback supersession schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/gpu-scheduler-heartbeat-pointer-rollback-supersession/v1"
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
        "previous_rollback_path",
        "next_rollback_path",
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
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn gpu_scheduler_heartbeat_pointer_rollback_supersession_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let previous = tmp.path().join("previous-rollback.json");
    let next = tmp.path().join("next-rollback.json");
    fs::write(
        &previous,
        r#"{
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
}"#,
    )
    .expect("write previous rollback");
    fs::write(
        &next,
        r#"{
  "schema_version": "1",
  "current_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.json",
  "restored_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.older.json",
  "previous_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.json",
  "restored_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.older.json",
  "job_id": "job-0001",
  "lease_id": "lease-job-0001",
  "worker_id": "worker-0",
  "previous_state": "running",
  "restored_state": "rolled-back",
  "previous_observed_at_unix_ms": 1735689960000,
  "restored_observed_at_unix_ms": 1735689840000,
  "previous_progress_marker": "checkpoint-group-0004/step-256",
  "restored_progress_marker": "checkpoint-group-0002/step-64",
  "rolled_back_at_unix_ms": 1735690150000
}"#,
    )
    .expect("write next rollback");
    let out = tmp
        .path()
        .join("gpu_scheduler_heartbeat_pointer_rollback_supersession.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("scheduler-heartbeat")
        .arg("pointer")
        .arg("rollback")
        .arg("supersede")
        .arg("--previous-rollback")
        .arg(&previous)
        .arg("--next-rollback")
        .arg(&next)
        .arg("--superseded-at-unix-ms")
        .arg("1735690200000")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read supersession");
    let mut supersession: Value = serde_json::from_str(&text).expect("parse rollback supersession");
    supersession
        .as_object_mut()
        .expect("supersession must be object")
        .insert(
            "previous_rollback_path".to_string(),
            Value::from("<previous>"),
        );
    supersession
        .as_object_mut()
        .expect("supersession must be object")
        .insert("next_rollback_path".to_string(), Value::from("<next>"));

    let expected_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_heartbeat_pointer_rollback_supersession.example.json",
    ))
    .expect("read heartbeat pointer rollback supersession example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse heartbeat pointer rollback supersession example");
    expected
        .as_object_mut()
        .expect("expected supersession must be object")
        .insert(
            "previous_rollback_path".to_string(),
            Value::from("<previous>"),
        );
    expected
        .as_object_mut()
        .expect("expected supersession must be object")
        .insert("next_rollback_path".to_string(), Value::from("<next>"));
    assert_eq!(supersession, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("gpu_scheduler_heartbeat_pointer_rollback_superseded")
        })
        .unwrap_or_else(|| {
            panic!("missing heartbeat pointer rollback supersession event in stderr: {stderr}")
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
    fields.insert("supersession_path".to_string(), Value::from("<out>"));
    let supersession_obj = fields
        .get_mut("supersession")
        .and_then(Value::as_object_mut)
        .expect("event supersession must be object");
    supersession_obj.insert(
        "previous_rollback_path".to_string(),
        Value::from("<previous>"),
    );
    supersession_obj.insert("next_rollback_path".to_string(), Value::from("<next>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "gpu_scheduler_heartbeat_pointer_rollback_superseded",
            "fields": {
                "supersession_path": "<out>",
                "supersession": expected
            }
        })
    );
}
