use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

#[path = "support/fixture.rs"]
mod fixture_test_support;

use fixture_test_support::fixture_path;

#[test]
fn gpu_scheduler_heartbeat_pointer_supersession_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_heartbeat_pointer_supersession.schema.json",
    ))
    .expect("read heartbeat pointer supersession schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse heartbeat pointer supersession schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/gpu-scheduler-heartbeat-pointer-supersession/v1")
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
        "previous_pointer_path",
        "next_pointer_path",
        "previous_heartbeat_path",
        "next_heartbeat_path",
        "job_id",
        "lease_id",
        "worker_id",
        "previous_state",
        "next_state",
        "previous_observed_at_unix_ms",
        "next_observed_at_unix_ms",
        "superseded_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn gpu_scheduler_heartbeat_pointer_supersession_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let previous = tmp.path().join("previous-pointer.json");
    let next = tmp.path().join("next-pointer.json");
    fs::write(
        &previous,
        r#"{
  "schema_version": "1",
  "heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.previous.json",
  "job_id": "job-0001",
  "lease_id": "lease-job-0001",
  "worker_id": "worker-0",
  "state": "running",
  "observed_at_unix_ms": 1735689900000,
  "progress_marker": "checkpoint-group-0003/step-128"
}"#,
    )
    .expect("write previous pointer");
    fs::write(
        &next,
        r#"{
  "schema_version": "1",
  "heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.json",
  "job_id": "job-0001",
  "lease_id": "lease-job-0001",
  "worker_id": "worker-0",
  "state": "checkpointed",
  "observed_at_unix_ms": 1735689960000,
  "progress_marker": "checkpoint-group-0004/step-256"
}"#,
    )
    .expect("write next pointer");
    let out = tmp
        .path()
        .join("gpu_scheduler_heartbeat_pointer_supersession.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("scheduler-heartbeat")
        .arg("pointer")
        .arg("supersede")
        .arg("--previous-pointer")
        .arg(&previous)
        .arg("--next-pointer")
        .arg(&next)
        .arg("--superseded-at-unix-ms")
        .arg("1735689999000")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read supersession");
    let mut supersession: Value = serde_json::from_str(&text).expect("parse supersession");
    supersession
        .as_object_mut()
        .expect("supersession must be object")
        .insert(
            "previous_pointer_path".to_string(),
            Value::from("<previous>"),
        );
    supersession
        .as_object_mut()
        .expect("supersession must be object")
        .insert("next_pointer_path".to_string(), Value::from("<next>"));

    let expected_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_heartbeat_pointer_supersession.example.json",
    ))
    .expect("read heartbeat pointer supersession example");
    let mut expected: Value =
        serde_json::from_str(&expected_text).expect("parse heartbeat pointer supersession example");
    expected
        .as_object_mut()
        .expect("expected supersession must be object")
        .insert(
            "previous_pointer_path".to_string(),
            Value::from("<previous>"),
        );
    expected
        .as_object_mut()
        .expect("expected supersession must be object")
        .insert("next_pointer_path".to_string(), Value::from("<next>"));
    assert_eq!(supersession, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("gpu_scheduler_heartbeat_pointer_superseded")
        })
        .unwrap_or_else(|| {
            panic!("missing heartbeat pointer supersession event in stderr: {stderr}")
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
        "previous_pointer_path".to_string(),
        Value::from("<previous>"),
    );
    supersession_obj.insert("next_pointer_path".to_string(), Value::from("<next>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "gpu_scheduler_heartbeat_pointer_superseded",
            "fields": {
                "supersession_path": "<out>",
                "supersession": expected
            }
        })
    );
}
