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
fn infer_output_drift_baseline_pointer_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "infer_output_drift_baseline_pointer.schema.json",
    ))
    .expect("read infer drift baseline pointer schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse infer drift baseline pointer");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/infer-output-drift-baseline-pointer/v1")
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
        justfile.contains("drift-point-approved-baseline approval:"),
        "justfile must expose the drift-point-approved-baseline workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("infer_output_drift_baseline_pointer.json"),
        "workflow reference must mention the baseline pointer artifact"
    );
    assert!(
        workflows.contains("just drift-point-approved-baseline"),
        "workflow reference must mention the baseline pointer workflow"
    );
}

#[test]
fn drift_point_approved_baseline_writes_pointer_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let approval = tmp.path().join("approval.json");
    fs::write(
        &approval,
        r#"{"schema_version":"1","baseline_path":"baseline.json","artifact":"artifacts/inference/candidate/model.mpk","artifact_version":"0.2.0","policy_profile":"promotion-default","approved_by":"ops-review","approval_ticket":"CHG-4242","approved_at_unix_ms":1735689600000}"#,
    )
    .expect("write approval");

    let out = tmp.path().join("pointer.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("drift")
        .arg("point-approved-baseline")
        .arg("--approval")
        .arg(&approval)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let pointer_text = fs::read_to_string(&out).expect("read pointer");
    let mut pointer: Value = serde_json::from_str(&pointer_text).expect("parse pointer");
    pointer
        .as_object_mut()
        .expect("pointer must be a top-level object")
        .insert("approval_path".to_string(), Value::from("<approval>"));
    assert_eq!(
        pointer,
        serde_json::json!({
            "schema_version": "1",
            "approval_path": "<approval>",
            "artifact": "artifacts/inference/candidate/model.mpk",
            "artifact_version": "0.2.0",
            "policy_profile": "promotion-default"
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "infer_output_drift_baseline_pointer_written");
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
    pointer_obj.insert("approval_path".to_string(), Value::from("<approval>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "infer_cli",
            "event": "infer_output_drift_baseline_pointer_written",
            "fields": {
                "pointer_path": "<pointer>",
                "pointer": pointer
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
