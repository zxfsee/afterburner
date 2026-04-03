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

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn gpu_scheduler_heartbeat_supersession_reconciliation_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "gpu_scheduler_heartbeat_supersession_reconciliation.schema.json",
    ))
    .expect("read heartbeat supersession reconciliation schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse heartbeat supersession reconciliation schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/gpu-scheduler-heartbeat-supersession-reconciliation/v1"
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

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains(
            "scheduler-heartbeat-supersession-reconcile supersession current_supersession:"
        ),
        "justfile must expose the scheduler-heartbeat-supersession-reconcile workflow"
    );

    let workflows = repo_file("docs/workflows.md");
    assert!(
        workflows.contains("gpu_scheduler_heartbeat_supersession_reconciliation.json"),
        "workflow reference must mention the scheduler heartbeat supersession reconciliation artifact"
    );
    assert!(
        workflows.contains("just scheduler-heartbeat-supersession-reconcile"),
        "workflow reference must mention the scheduler heartbeat supersession reconciliation workflow"
    );

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("gpu_scheduler_heartbeat_supersession_reconciliation.json"),
        "reference index must mention the scheduler heartbeat supersession reconciliation artifact"
    );
}

#[test]
fn gpu_scheduler_heartbeat_supersession_reconciliation_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let supersession = tmp.path().join("gpu_scheduler_heartbeat_supersession.json");
    let current = tmp
        .path()
        .join("gpu_scheduler_heartbeat_supersession.current.json");
    fs::write(
        &supersession,
        r#"{
  "schema_version": "1",
  "previous_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.previous.json",
  "next_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.json",
  "job_id": "job-0001",
  "lease_id": "lease-job-0001",
  "worker_id": "worker-0",
  "previous_state": "running",
  "next_state": "checkpointed",
  "previous_observed_at_unix_ms": 1735689900000,
  "next_observed_at_unix_ms": 1735689960000,
  "previous_progress_marker": "checkpoint-group-0003/step-128",
  "next_progress_marker": "checkpoint-group-0004/step-256",
  "superseded_at_unix_ms": 1735689999000
}"#,
    )
    .expect("write desired supersession");
    fs::write(
        &current,
        r#"{
  "schema_version": "1",
  "previous_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.previous.json",
  "next_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.json",
  "job_id": "job-0001",
  "lease_id": "lease-job-0001",
  "worker_id": "worker-0",
  "previous_state": "running",
  "next_state": "checkpointed",
  "previous_observed_at_unix_ms": 1735689900000,
  "next_observed_at_unix_ms": 1735689960000,
  "previous_progress_marker": "checkpoint-group-0003/step-128",
  "next_progress_marker": "checkpoint-group-0004/step-256",
  "superseded_at_unix_ms": 1735689999000
}"#,
    )
    .expect("write current supersession");
    let out = tmp
        .path()
        .join("gpu_scheduler_heartbeat_supersession_reconciliation.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("scheduler-heartbeat")
        .arg("reconcile-supersession")
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
        "gpu_scheduler_heartbeat_supersession_reconciliation.example.json",
    ))
    .expect("read heartbeat supersession reconciliation example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse heartbeat supersession reconciliation example");
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
                == Some("gpu_scheduler_heartbeat_supersession_reconciliation_written")
        })
        .unwrap_or_else(|| {
            panic!(
                "missing gpu_scheduler_heartbeat_supersession_reconciliation_written event in stderr: {stderr}"
            )
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
            "event": "gpu_scheduler_heartbeat_supersession_reconciliation_written",
            "fields": {
                "reconciliation_path": "<reconciliation>",
                "reconciliation": expected
            }
        })
    );
}
