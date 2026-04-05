use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

#[path = "support/fixture.rs"]
mod fixture_test_support;

use fixture_test_support::fixture_path;

#[test]
fn gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation_schema_is_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation.schema.json",
    ))
    .expect("read heartbeat pointer rollback supersession reconciliation schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse heartbeat pointer rollback supersession reconciliation schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/gpu-scheduler-heartbeat-pointer-rollback-supersession-reconciliation/v1"
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
        "supersession_path",
        "current_supersession_path",
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
fn gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation_writes_artifact_and_event()
{
    let tmp = tempfile::tempdir().expect("tempdir");
    let supersession = tmp
        .path()
        .join("gpu_scheduler_heartbeat_pointer_rollback_supersession.json");
    let current = tmp
        .path()
        .join("gpu_scheduler_heartbeat_pointer_rollback_supersession.current.json");
    let supersession_text = r#"{
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
}"#;
    fs::write(&supersession, supersession_text).expect("write desired supersession");
    fs::write(&current, supersession_text).expect("write current supersession");
    let out = tmp
        .path()
        .join("gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("scheduler-heartbeat")
        .arg("pointer")
        .arg("rollback")
        .arg("supersession")
        .arg("reconcile")
        .arg("--supersession")
        .arg(&supersession)
        .arg("--current-supersession")
        .arg(&current)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read reconciliation");
    let mut reconciliation: Value = serde_json::from_str(&text).expect("parse reconciliation");
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert(
            "supersession_path".to_string(),
            Value::from("<supersession>"),
        );
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert(
            "current_supersession_path".to_string(),
            Value::from("<current>"),
        );
    let current_obj = reconciliation
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("current object");
    current_obj.insert(
        "current_supersession_path".to_string(),
        Value::from("<current>"),
    );

    let expected_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation.example.json",
    ))
    .expect("read heartbeat pointer rollback supersession reconciliation example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse heartbeat pointer rollback supersession reconciliation example");
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert(
            "supersession_path".to_string(),
            Value::from("<supersession>"),
        );
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert(
            "current_supersession_path".to_string(),
            Value::from("<current>"),
        );
    let expected_current = expected
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("expected current object");
    expected_current.insert(
        "current_supersession_path".to_string(),
        Value::from("<current>"),
    );
    assert_eq!(reconciliation, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some(
                    "gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation_written",
                )
        })
        .unwrap_or_else(|| {
            panic!("missing pointer rollback supersession reconciliation event in stderr: {stderr}")
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
    reconciliation_obj.insert(
        "supersession_path".to_string(),
        Value::from("<supersession>"),
    );
    reconciliation_obj.insert(
        "current_supersession_path".to_string(),
        Value::from("<current>"),
    );
    let event_current = reconciliation_obj
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("event current object");
    event_current.insert(
        "current_supersession_path".to_string(),
        Value::from("<current>"),
    );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation_written",
            "fields": {
                "reconciliation_path": "<reconciliation>",
                "reconciliation": expected
            }
        })
    );
}
