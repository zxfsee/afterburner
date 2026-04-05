use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::fixture_path;

#[test]
fn gpu_scheduler_heartbeat_pointer_schema_and_workflow_are_explicit() {
    let schema_text =
        fs::read_to_string(fixture_path("gpu_scheduler_heartbeat_pointer.schema.json"))
            .expect("read heartbeat pointer schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse heartbeat pointer schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/gpu-scheduler-heartbeat-pointer/v1")
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
        "heartbeat_path",
        "job_id",
        "lease_id",
        "worker_id",
        "state",
        "observed_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
    assert!(
        schema
            .get("properties")
            .and_then(|value| value.get("progress_marker"))
            .is_some(),
        "schema must expose optional progress_marker"
    );
}

#[test]
fn gpu_scheduler_heartbeat_pointer_writes_pointer_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let heartbeat = tmp.path().join("gpu_scheduler_heartbeat.json");
    fs::write(
        &heartbeat,
        r#"{
  "schema_version": "1",
  "job_id": "job-0001",
  "lease_id": "lease-job-0001",
  "worker_id": "worker-0",
  "state": "running",
  "observed_at_unix_ms": 1735689900000,
  "progress_marker": "checkpoint-group-0003/step-128"
}"#,
    )
    .expect("write heartbeat");

    let out = tmp.path().join("gpu_scheduler_heartbeat_pointer.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("scheduler-heartbeat")
        .arg("point")
        .arg("--heartbeat")
        .arg(&heartbeat)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read heartbeat pointer");
    let mut pointer: Value = serde_json::from_str(&text).expect("parse heartbeat pointer");
    pointer
        .as_object_mut()
        .expect("pointer must be object")
        .insert("heartbeat_path".to_string(), Value::from("<heartbeat>"));
    let expected_text =
        fs::read_to_string(fixture_path("gpu_scheduler_heartbeat_pointer.example.json"))
            .expect("read heartbeat pointer example");
    let mut expected: Value =
        serde_json::from_str(&expected_text).expect("parse heartbeat pointer example");
    expected
        .as_object_mut()
        .expect("expected pointer must be object")
        .insert("heartbeat_path".to_string(), Value::from("<heartbeat>"));
    assert_eq!(pointer, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("gpu_scheduler_heartbeat_pointer_written")
        })
        .unwrap_or_else(|| {
            panic!("missing gpu scheduler heartbeat pointer event in stderr: {stderr}")
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
    fields.insert("pointer_path".to_string(), Value::from("<pointer>"));
    let pointer_obj = fields
        .get_mut("pointer")
        .and_then(Value::as_object_mut)
        .expect("event pointer must be an object");
    pointer_obj.insert("heartbeat_path".to_string(), Value::from("<heartbeat>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "gpu_scheduler_heartbeat_pointer_written",
            "fields": {
                "pointer_path": "<pointer>",
                "pointer": expected
            }
        })
    );
}
