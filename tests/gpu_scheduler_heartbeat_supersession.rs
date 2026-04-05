use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::fixture_path;

#[test]
fn gpu_scheduler_heartbeat_supersession_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_heartbeat_supersession.schema.json",
    ))
    .expect("read heartbeat supersession schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse heartbeat supersession schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/gpu-scheduler-heartbeat-supersession/v1")
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
fn gpu_scheduler_heartbeat_supersession_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let previous = tmp.path().join("previous.json");
    let next = tmp.path().join("next.json");
    fs::write(
        &previous,
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
    .expect("write previous heartbeat");
    fs::write(
        &next,
        r#"{
  "schema_version": "1",
  "job_id": "job-0001",
  "lease_id": "lease-job-0001",
  "worker_id": "worker-0",
  "state": "checkpointed",
  "observed_at_unix_ms": 1735689960000,
  "progress_marker": "checkpoint-group-0004/step-256"
}"#,
    )
    .expect("write next heartbeat");
    let out = tmp.path().join("gpu_scheduler_heartbeat_supersession.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("scheduler-heartbeat")
        .arg("supersede")
        .arg("--previous-heartbeat")
        .arg(&previous)
        .arg("--next-heartbeat")
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
            "previous_heartbeat_path".to_string(),
            Value::from("<previous>"),
        );
    supersession
        .as_object_mut()
        .expect("supersession must be object")
        .insert("next_heartbeat_path".to_string(), Value::from("<next>"));

    let expected_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_heartbeat_supersession.example.json",
    ))
    .expect("read heartbeat supersession example");
    let mut expected: Value =
        serde_json::from_str(&expected_text).expect("parse heartbeat supersession example");
    expected
        .as_object_mut()
        .expect("expected supersession must be object")
        .insert(
            "previous_heartbeat_path".to_string(),
            Value::from("<previous>"),
        );
    expected
        .as_object_mut()
        .expect("expected supersession must be object")
        .insert("next_heartbeat_path".to_string(), Value::from("<next>"));
    assert_eq!(supersession, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str) == Some("gpu_scheduler_heartbeat_superseded")
        })
        .unwrap_or_else(|| {
            panic!("missing gpu_scheduler_heartbeat_superseded event in stderr: {stderr}")
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
        "previous_heartbeat_path".to_string(),
        Value::from("<previous>"),
    );
    supersession_obj.insert("next_heartbeat_path".to_string(), Value::from("<next>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "gpu_scheduler_heartbeat_superseded",
            "fields": {
                "supersession_path": "<out>",
                "supersession": expected
            }
        })
    );
}
