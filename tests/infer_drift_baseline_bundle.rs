use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn infer_output_drift_baseline_bundle_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "infer_output_drift_baseline_bundle.schema.json",
    ))
    .expect("read infer drift baseline bundle schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse infer drift baseline bundle");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/infer-output-drift-baseline-bundle/v1")
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
        "baseline_path",
        "artifact",
        "artifact_version",
        "policy_profile",
        "history_entry_count",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("infer_output_drift_baseline_bundle.json"),
        "workflow reference must mention the baseline bundle artifact"
    );
}

#[test]
fn drift_export_baseline_bundle_writes_bundle_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pointer = tmp.path().join("pointer.json");
    let history = tmp.path().join("history.json");
    let approval = tmp.path().join("approval.json");
    let baseline = tmp.path().join("baseline.json");
    fs::write(
        &pointer,
        format!(
            "{{\"schema_version\":\"1\",\"approval_path\":\"{}\",\"artifact\":\"artifacts/inference/0.2.0/model.mpk\",\"artifact_version\":\"0.2.0\",\"policy_profile\":\"promotion-default\"}}",
            approval.display()
        ),
    )
    .expect("write pointer");
    fs::write(
        &history,
        format!(
            "{{\"schema_version\":\"1\",\"entries\":[{{\"event\":\"pointed\",\"pointer_path\":\"{}\",\"approval_path\":\"{}\",\"artifact\":\"artifacts/inference/0.2.0/model.mpk\",\"artifact_version\":\"0.2.0\",\"policy_profile\":\"promotion-default\",\"recorded_at_unix_ms\":1735689800000}}]}}",
            pointer.display(),
            approval.display()
        ),
    )
    .expect("write history");
    fs::write(
        &approval,
        format!(
            "{{\"schema_version\":\"1\",\"baseline_path\":\"{}\",\"artifact\":\"artifacts/inference/0.2.0/model.mpk\",\"artifact_version\":\"0.2.0\",\"policy_profile\":\"promotion-default\",\"approved_by\":\"ops-review\",\"approval_ticket\":\"CHG-4242\",\"approved_at_unix_ms\":1735689600000}}",
            baseline.display()
        ),
    )
    .expect("write approval");
    fs::write(
        &baseline,
        r#"{"schema_version":"1","summary_path":"summary.json","receipt_path":"receipt.json","policy_path":"policy.json","policy_profile":"promotion-default","artifact":"artifacts/inference/0.2.0/model.mpk","artifact_version":"0.2.0","backend":"cpu","predicted_class":3,"top_probability":0.91,"margin_to_second":0.44,"logits_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","probabilities_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}"#,
    )
    .expect("write baseline");

    let out = tmp.path().join("bundle.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("drift")
        .arg("export-baseline-bundle")
        .arg("--pointer")
        .arg(&pointer)
        .arg("--history")
        .arg(&history)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let bundle_text = fs::read_to_string(&out).expect("read bundle");
    let mut bundle: Value = serde_json::from_str(&bundle_text).expect("parse bundle");
    let object = bundle.as_object_mut().expect("bundle must be object");
    object.insert("pointer_path".to_string(), Value::from("<pointer>"));
    object.insert("history_path".to_string(), Value::from("<history>"));
    object.insert("approval_path".to_string(), Value::from("<approval>"));
    object.insert("baseline_path".to_string(), Value::from("<baseline>"));
    assert_eq!(
        bundle,
        serde_json::json!({
            "schema_version": "1",
            "pointer_path": "<pointer>",
            "history_path": "<history>",
            "approval_path": "<approval>",
            "baseline_path": "<baseline>",
            "artifact": "artifacts/inference/0.2.0/model.mpk",
            "artifact_version": "0.2.0",
            "policy_profile": "promotion-default",
            "history_entry_count": 1
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "infer_output_drift_baseline_bundle_written");
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be object");
    fields.insert("bundle_path".to_string(), Value::from("<bundle>"));
    let bundle_obj = fields
        .get_mut("bundle")
        .and_then(Value::as_object_mut)
        .expect("event bundle must be object");
    bundle_obj.insert("pointer_path".to_string(), Value::from("<pointer>"));
    bundle_obj.insert("history_path".to_string(), Value::from("<history>"));
    bundle_obj.insert("approval_path".to_string(), Value::from("<approval>"));
    bundle_obj.insert("baseline_path".to_string(), Value::from("<baseline>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "infer_cli",
            "event": "infer_output_drift_baseline_bundle_written",
            "fields": {
                "bundle_path": "<bundle>",
                "bundle": bundle
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
