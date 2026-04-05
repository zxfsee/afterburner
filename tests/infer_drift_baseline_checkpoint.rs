use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn infer_output_drift_baseline_checkpoint_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "infer_output_drift_baseline_checkpoint.schema.json",
    ))
    .expect("read infer drift baseline checkpoint schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse infer drift baseline checkpoint");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/infer-output-drift-baseline-checkpoint/v1")
    );
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("schema.required items must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "pointer_path",
        "history_path",
        "approval_path",
        "artifact",
        "artifact_version",
        "policy_profile",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("drift-checkpoint-baseline pointer history:"),
        "justfile must expose the drift-checkpoint-baseline workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("infer_output_drift_baseline_checkpoint.json"),
        "workflow reference must mention the baseline checkpoint artifact"
    );
    assert!(
        workflows.contains("just drift-checkpoint-baseline"),
        "workflow reference must mention the baseline checkpoint workflow"
    );
}

#[test]
fn drift_checkpoint_baseline_writes_checkpoint_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pointer = tmp.path().join("pointer.json");
    let history = tmp.path().join("history.json");
    fs::write(
        &pointer,
        r#"{"schema_version":"1","approval_path":"approval.json","artifact":"artifacts/inference/0.2.0/model.mpk","artifact_version":"0.2.0","policy_profile":"promotion-default"}"#,
    )
    .expect("write pointer");
    fs::write(
        &history,
        r#"{"schema_version":"1","entries":[{"event":"pointed","pointer_path":"pointer.json","approval_path":"approval.json","artifact":"artifacts/inference/0.2.0/model.mpk","artifact_version":"0.2.0","policy_profile":"promotion-default","recorded_at_unix_ms":1735689800000}]}"#,
    )
    .expect("write history");

    let out = tmp.path().join("checkpoint.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("drift")
        .arg("checkpoint-baseline")
        .arg("--pointer")
        .arg(&pointer)
        .arg("--history")
        .arg(&history)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read checkpoint");
    let mut checkpoint: Value = serde_json::from_str(&text).expect("parse checkpoint");
    let object = checkpoint
        .as_object_mut()
        .expect("checkpoint must be object");
    object.insert("pointer_path".to_string(), Value::from("<pointer>"));
    object.insert("history_path".to_string(), Value::from("<history>"));
    assert_eq!(
        checkpoint,
        serde_json::json!({
            "schema_version": "1",
            "pointer_path": "<pointer>",
            "history_path": "<history>",
            "approval_path": "approval.json",
            "artifact": "artifacts/inference/0.2.0/model.mpk",
            "artifact_version": "0.2.0",
            "policy_profile": "promotion-default"
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "infer_output_drift_baseline_checkpoint_written");
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be object");
    fields.insert("checkpoint_path".to_string(), Value::from("<checkpoint>"));
    let checkpoint_obj = fields
        .get_mut("checkpoint")
        .and_then(Value::as_object_mut)
        .expect("event checkpoint must be object");
    checkpoint_obj.insert("pointer_path".to_string(), Value::from("<pointer>"));
    checkpoint_obj.insert("history_path".to_string(), Value::from("<history>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "infer_cli",
            "event": "infer_output_drift_baseline_checkpoint_written",
            "fields": {
                "checkpoint_path": "<checkpoint>",
                "checkpoint": checkpoint
            }
        })
    );
}

fn stderr_event(stderr: &str, event_name: &str) -> Value {
    stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| event.get("event").and_then(Value::as_str) == Some(event_name))
        .unwrap_or_else(|| panic!("missing event `{event_name}` in stderr: {stderr}"))
}
