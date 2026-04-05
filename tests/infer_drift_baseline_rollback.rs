use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn infer_output_drift_baseline_rollback_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "infer_output_drift_baseline_rollback.schema.json",
    ))
    .expect("read infer drift baseline rollback schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse infer drift baseline rollback");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/infer-output-drift-baseline-rollback/v1")
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
        "current_pointer_path",
        "restored_approval_path",
        "previous_artifact_version",
        "restored_artifact_version",
        "policy_profile",
        "rolled_back_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("infer_output_drift_baseline_rollback.json"),
        "workflow reference must mention the baseline rollback artifact"
    );
}

#[test]
fn drift_rollback_approved_baseline_writes_pointer_and_rollback_record() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let current_pointer = tmp.path().join("pointer.json");
    let restored_approval = tmp.path().join("approval.json");
    fs::write(
        &current_pointer,
        r#"{"schema_version":"1","approval_path":"approval-new.json","artifact":"artifacts/inference/0.2.0/model.mpk","artifact_version":"0.2.0","policy_profile":"promotion-default"}"#,
    )
    .expect("write current pointer");
    fs::write(
        &restored_approval,
        r#"{"schema_version":"1","baseline_path":"baseline-old.json","artifact":"artifacts/inference/0.1.0/model.mpk","artifact_version":"0.1.0","policy_profile":"promotion-default","approved_by":"ops-review","approval_ticket":"CHG-1000","approved_at_unix_ms":1735689500000}"#,
    )
    .expect("write restored approval");

    let out_pointer = tmp.path().join("pointer-restored.json");
    let out_record = tmp.path().join("rollback.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("drift")
        .arg("rollback-approved-baseline")
        .arg("--current-pointer")
        .arg(&current_pointer)
        .arg("--restored-approval")
        .arg(&restored_approval)
        .arg("--rolled-back-at-unix-ms")
        .arg("1735689800000")
        .arg("--out-pointer")
        .arg(&out_pointer)
        .arg("--out-record")
        .arg(&out_record);
    let assert = cmd.assert().success();

    let pointer_text = fs::read_to_string(&out_pointer).expect("read restored pointer");
    let mut pointer: Value = serde_json::from_str(&pointer_text).expect("parse restored pointer");
    pointer
        .as_object_mut()
        .expect("pointer must be object")
        .insert("approval_path".to_string(), Value::from("<approval>"));
    assert_eq!(
        pointer,
        serde_json::json!({
            "schema_version": "1",
            "approval_path": "<approval>",
            "artifact": "artifacts/inference/0.1.0/model.mpk",
            "artifact_version": "0.1.0",
            "policy_profile": "promotion-default"
        })
    );

    let rollback_text = fs::read_to_string(&out_record).expect("read rollback record");
    let mut rollback: Value = serde_json::from_str(&rollback_text).expect("parse rollback record");
    let object = rollback.as_object_mut().expect("rollback must be object");
    object.insert("current_pointer_path".to_string(), Value::from("<pointer>"));
    object.insert(
        "restored_approval_path".to_string(),
        Value::from("<approval>"),
    );
    assert_eq!(
        rollback,
        serde_json::json!({
            "schema_version": "1",
            "current_pointer_path": "<pointer>",
            "restored_approval_path": "<approval>",
            "previous_artifact_version": "0.2.0",
            "restored_artifact_version": "0.1.0",
            "policy_profile": "promotion-default",
            "rolled_back_at_unix_ms": 1735689800000_u64
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "infer_output_drift_baseline_rolled_back");
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
    let pointer_obj = fields
        .get_mut("pointer")
        .and_then(Value::as_object_mut)
        .expect("event pointer must be object");
    pointer_obj.insert("approval_path".to_string(), Value::from("<approval>"));
    let rollback_obj = fields
        .get_mut("rollback")
        .and_then(Value::as_object_mut)
        .expect("event rollback must be object");
    rollback_obj.insert("current_pointer_path".to_string(), Value::from("<pointer>"));
    rollback_obj.insert(
        "restored_approval_path".to_string(),
        Value::from("<approval>"),
    );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "infer_cli",
            "event": "infer_output_drift_baseline_rolled_back",
            "fields": {
                "pointer_path": "<pointer-out>",
                "rollback_path": "<rollback-out>",
                "pointer": pointer,
                "rollback": rollback
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
