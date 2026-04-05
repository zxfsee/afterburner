use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn infer_output_drift_baseline_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path("infer_output_drift_baseline.schema.json"))
        .expect("read infer drift baseline schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse infer drift baseline");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/infer-output-drift-baseline/v1")
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
        "summary_path",
        "receipt_path",
        "policy_path",
        "policy_profile",
        "artifact",
        "artifact_version",
        "backend",
        "predicted_class",
        "top_probability",
        "margin_to_second",
        "logits_sha256",
        "probabilities_sha256",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("drift-baseline summary receipt:"),
        "justfile must expose the drift-baseline workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("infer_output_drift_baseline.json"),
        "workflow reference must mention the infer drift baseline artifact"
    );
    assert!(
        workflows.contains("just drift-baseline"),
        "workflow reference must mention the drift baseline workflow"
    );
}

#[test]
fn drift_baseline_writes_baseline_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let summary = tmp.path().join("summary.json");
    let receipt = tmp.path().join("receipt.json");
    fs::write(
        &summary,
        r#"{"schema_version":"1","artifact":"artifacts/inference/candidate/model.mpk","artifact_version":"0.2.0","backend":"cpu","predicted_class":3,"top_probability":0.91,"margin_to_second":0.44,"logits_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","probabilities_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}"#,
    )
    .expect("write summary");
    fs::write(
        &receipt,
        format!(
            "{{\"schema_version\":\"1\",\"candidate_summary_path\":\"{}\",\"current_summary_path\":\"current.json\",\"policy_path\":\"policy.json\",\"policy_profile\":\"promotion-default\",\"candidate_artifact\":\"artifacts/inference/candidate/model.mpk\",\"current_artifact\":\"artifacts/inference/current/model.mpk\",\"candidate_artifact_version\":\"0.2.0\",\"current_artifact_version\":\"0.1.0\",\"candidate_predicted_class\":3,\"current_predicted_class\":3,\"top_probability_delta\":0.02,\"margin_to_second_delta\":0.04,\"max_top_probability_delta\":0.05,\"max_margin_to_second_delta\":0.05,\"passed\":true}}",
            summary.display()
        ),
    )
    .expect("write receipt");

    let out = tmp.path().join("baseline.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("drift")
        .arg("baseline")
        .arg("--summary")
        .arg(&summary)
        .arg("--receipt")
        .arg(&receipt)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let baseline_text = fs::read_to_string(&out).expect("read baseline");
    let mut baseline: Value = serde_json::from_str(&baseline_text).expect("parse baseline");
    normalize_baseline(&mut baseline);
    assert_eq!(
        baseline,
        serde_json::json!({
            "schema_version": "1",
            "summary_path": "<summary>",
            "receipt_path": "<receipt>",
            "policy_path": "policy.json",
            "policy_profile": "promotion-default",
            "artifact": "artifacts/inference/candidate/model.mpk",
            "artifact_version": "0.2.0",
            "backend": "cpu",
            "predicted_class": 3,
            "top_probability": 0.91,
            "margin_to_second": 0.44,
            "logits_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "probabilities_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "infer_output_drift_baseline_written");
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("baseline_path".to_string(), Value::from("<baseline>"));
    let baseline_obj = fields
        .get_mut("baseline")
        .and_then(Value::as_object_mut)
        .expect("event baseline must be an object");
    baseline_obj.insert("summary_path".to_string(), Value::from("<summary>"));
    baseline_obj.insert("receipt_path".to_string(), Value::from("<receipt>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "infer_cli",
            "event": "infer_output_drift_baseline_written",
            "fields": {
                "baseline_path": "<baseline>",
                "baseline": baseline
            }
        })
    );
}

#[test]
fn drift_baseline_rejects_failed_receipt() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let summary = tmp.path().join("summary.json");
    let receipt = tmp.path().join("receipt.json");
    fs::write(
        &summary,
        r#"{"schema_version":"1","artifact":"artifacts/inference/candidate/model.mpk","artifact_version":"0.2.0","backend":"cpu","predicted_class":3,"top_probability":0.91,"margin_to_second":0.44,"logits_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","probabilities_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}"#,
    )
    .expect("write summary");
    fs::write(
        &receipt,
        format!(
            "{{\"schema_version\":\"1\",\"candidate_summary_path\":\"{}\",\"current_summary_path\":\"current.json\",\"policy_path\":\"policy.json\",\"policy_profile\":\"promotion-default\",\"candidate_artifact\":\"artifacts/inference/candidate/model.mpk\",\"current_artifact\":\"artifacts/inference/current/model.mpk\",\"candidate_artifact_version\":\"0.2.0\",\"current_artifact_version\":\"0.1.0\",\"candidate_predicted_class\":3,\"current_predicted_class\":4,\"top_probability_delta\":0.2,\"margin_to_second_delta\":0.4,\"max_top_probability_delta\":0.05,\"max_margin_to_second_delta\":0.05,\"passed\":false}}",
            summary.display()
        ),
    )
    .expect("write failed receipt");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("drift")
        .arg("baseline")
        .arg("--summary")
        .arg(&summary)
        .arg("--receipt")
        .arg(&receipt);
    cmd.assert().failure().code(2);
}

fn stderr_event(stderr: &str, event_name: &str) -> Value {
    stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| event.get("event").and_then(Value::as_str) == Some(event_name))
        .unwrap_or_else(|| panic!("missing event `{event_name}` in stderr: {stderr}"))
}

fn normalize_baseline(baseline: &mut Value) {
    let object = baseline
        .as_object_mut()
        .expect("baseline must be a top-level object");
    object.insert("summary_path".to_string(), Value::from("<summary>"));
    object.insert("receipt_path".to_string(), Value::from("<receipt>"));
}
