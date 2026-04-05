use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

#[path = "support/fixture.rs"]
mod fixture_test_support;

use fixture_test_support::fixture_path;

#[test]
fn gpu_scheduler_heartbeat_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path("gpu_scheduler_heartbeat.schema.json"))
        .expect("read heartbeat schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse heartbeat schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/gpu-scheduler-heartbeat/v1")
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
fn gpu_scheduler_heartbeat_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("gpu_scheduler_heartbeat.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("scheduler-heartbeat")
        .arg("--job-id")
        .arg("job-0001")
        .arg("--lease-id")
        .arg("lease-job-0001")
        .arg("--worker-id")
        .arg("worker-0")
        .arg("--state")
        .arg("running")
        .arg("--observed-at-unix-ms")
        .arg("1735689900000")
        .arg("--progress-marker")
        .arg("checkpoint-group-0003/step-128")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read heartbeat");
    let heartbeat: Value = serde_json::from_str(&text).expect("parse heartbeat");
    let expected_text = fs::read_to_string(fixture_path("gpu_scheduler_heartbeat.example.json"))
        .expect("read heartbeat example");
    let expected: Value = serde_json::from_str(&expected_text).expect("parse heartbeat example");
    assert_eq!(heartbeat, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str) == Some("gpu_scheduler_heartbeat_written")
        })
        .unwrap_or_else(|| {
            panic!("missing gpu_scheduler_heartbeat_written event in stderr: {stderr}")
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
    fields.insert("heartbeat_path".to_string(), Value::from("<heartbeat>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "gpu_scheduler_heartbeat_written",
            "fields": {
                "heartbeat_path": "<heartbeat>",
                "heartbeat": expected
            }
        })
    );
}
