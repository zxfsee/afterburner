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
fn infer_output_drift_receipt_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path("infer_output_drift_receipt.schema.json"))
        .expect("read infer drift receipt schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse infer drift receipt");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/infer-output-drift-receipt/v1")
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
        "candidate_summary_path",
        "current_summary_path",
        "policy_path",
        "policy_profile",
        "candidate_artifact",
        "current_artifact",
        "candidate_artifact_version",
        "current_artifact_version",
        "candidate_predicted_class",
        "current_predicted_class",
        "top_probability_delta",
        "margin_to_second_delta",
        "max_top_probability_delta",
        "max_margin_to_second_delta",
        "passed",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("drift-receipt candidate_summary current_summary policy:"),
        "justfile must expose the drift-receipt workflow"
    );

    let readme = repo_file("README.md");
    assert!(
        readme.contains("infer_output_drift_receipt.json"),
        "README must mention the infer drift receipt artifact"
    );
    assert!(
        readme.contains("just drift-receipt"),
        "README must mention the drift receipt workflow"
    );
    let policy_text = fs::read_to_string(fixture_path("infer_output_drift_policy.schema.json"))
        .expect("read infer drift policy schema");
    let policy: Value = serde_json::from_str(&policy_text).expect("parse infer drift policy");
    assert_eq!(
        policy.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/infer-output-drift-policy/v1")
    );
}

#[test]
fn drift_receipt_writes_receipt_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let candidate = tmp.path().join("candidate.json");
    let current = tmp.path().join("current.json");
    let policy = tmp.path().join("policy.json");
    fs::write(
        &candidate,
        r#"{"schema_version":"1","artifact":"artifacts/inference/candidate/model.mpk","artifact_version":"0.2.0","backend":"cpu","predicted_class":3,"top_probability":0.91,"margin_to_second":0.44,"logits_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","probabilities_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}"#,
    )
    .expect("write candidate summary");
    fs::write(
        &current,
        r#"{"schema_version":"1","artifact":"artifacts/inference/current/model.mpk","artifact_version":"0.1.0","backend":"cpu","predicted_class":3,"top_probability":0.89,"margin_to_second":0.40,"logits_sha256":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","probabilities_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"}"#,
    )
    .expect("write current summary");
    fs::write(
        &policy,
        r#"{"schema_version":"1","profile_name":"promotion-default","max_top_probability_delta":0.05,"max_margin_to_second_delta":0.05}"#,
    )
    .expect("write drift policy");

    let out = tmp.path().join("receipt.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("drift-receipt")
        .arg("--candidate")
        .arg(&candidate)
        .arg("--current")
        .arg(&current)
        .arg("--policy")
        .arg(&policy)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let receipt_text = fs::read_to_string(&out).expect("read drift receipt");
    let mut receipt: Value = serde_json::from_str(&receipt_text).expect("parse drift receipt");
    normalize_receipt(&mut receipt);
    assert_eq!(
        receipt,
        serde_json::json!({
            "schema_version": "1",
            "candidate_summary_path": "<candidate>",
            "current_summary_path": "<current>",
            "policy_path": "<policy>",
            "policy_profile": "promotion-default",
            "candidate_artifact": "artifacts/inference/candidate/model.mpk",
            "current_artifact": "artifacts/inference/current/model.mpk",
            "candidate_artifact_version": "0.2.0",
            "current_artifact_version": "0.1.0",
            "candidate_predicted_class": 3,
            "current_predicted_class": 3,
            "top_probability_delta": 0.02,
            "margin_to_second_delta": 0.04,
            "max_top_probability_delta": 0.05,
            "max_margin_to_second_delta": 0.05,
            "passed": true
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "infer_output_drift_receipt_written");
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("receipt_path".to_string(), Value::from("<receipt>"));
    let receipt_obj = fields
        .get_mut("receipt")
        .and_then(Value::as_object_mut)
        .expect("event receipt must be an object");
    receipt_obj.insert(
        "candidate_summary_path".to_string(),
        Value::from("<candidate>"),
    );
    receipt_obj.insert("current_summary_path".to_string(), Value::from("<current>"));
    receipt_obj.insert("policy_path".to_string(), Value::from("<policy>"));
    let top_probability_delta = receipt_obj
        .get("top_probability_delta")
        .and_then(Value::as_f64)
        .expect("event receipt top_probability_delta must be numeric");
    receipt_obj.insert(
        "top_probability_delta".to_string(),
        serde_json::json!((top_probability_delta * 1_000_000.0).round() / 1_000_000.0),
    );
    let margin_to_second_delta = receipt_obj
        .get("margin_to_second_delta")
        .and_then(Value::as_f64)
        .expect("event receipt margin_to_second_delta must be numeric");
    receipt_obj.insert(
        "margin_to_second_delta".to_string(),
        serde_json::json!((margin_to_second_delta * 1_000_000.0).round() / 1_000_000.0),
    );

    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "infer_cli",
            "event": "infer_output_drift_receipt_written",
            "fields": {
                "receipt_path": "<receipt>",
                "receipt": receipt
            }
        })
    );
}

#[test]
fn drift_receipt_fails_when_thresholds_are_exceeded() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let candidate = tmp.path().join("candidate.json");
    let current = tmp.path().join("current.json");
    let policy = tmp.path().join("policy.json");
    fs::write(
        &candidate,
        r#"{"schema_version":"1","artifact":"artifacts/inference/candidate/model.mpk","artifact_version":"0.2.0","backend":"cpu","predicted_class":3,"top_probability":0.91,"margin_to_second":0.44,"logits_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","probabilities_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}"#,
    )
    .expect("write candidate summary");
    fs::write(
        &current,
        r#"{"schema_version":"1","artifact":"artifacts/inference/current/model.mpk","artifact_version":"0.1.0","backend":"cpu","predicted_class":2,"top_probability":0.70,"margin_to_second":0.10,"logits_sha256":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","probabilities_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"}"#,
    )
    .expect("write current summary");
    fs::write(
        &policy,
        r#"{"schema_version":"1","profile_name":"promotion-default","max_top_probability_delta":0.05,"max_margin_to_second_delta":0.05}"#,
    )
    .expect("write drift policy");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("drift-receipt")
        .arg("--candidate")
        .arg(&candidate)
        .arg("--current")
        .arg(&current)
        .arg("--policy")
        .arg(&policy);
    cmd.assert().failure().code(2);
}

fn stderr_event(stderr: &str, event_name: &str) -> Value {
    stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| event.get("event").and_then(Value::as_str) == Some(event_name))
        .unwrap_or_else(|| panic!("missing event `{event_name}` in stderr: {stderr}"))
}

fn normalize_receipt(receipt: &mut Value) {
    let object = receipt
        .as_object_mut()
        .expect("receipt must be a top-level object");
    object.insert(
        "candidate_summary_path".to_string(),
        Value::from("<candidate>"),
    );
    object.insert("current_summary_path".to_string(), Value::from("<current>"));
    object.insert("policy_path".to_string(), Value::from("<policy>"));
    let top_probability_delta = object
        .get("top_probability_delta")
        .and_then(Value::as_f64)
        .expect("top_probability_delta must be numeric");
    object.insert(
        "top_probability_delta".to_string(),
        serde_json::json!((top_probability_delta * 1_000_000.0).round() / 1_000_000.0),
    );
    let margin_to_second_delta = object
        .get("margin_to_second_delta")
        .and_then(Value::as_f64)
        .expect("margin_to_second_delta must be numeric");
    object.insert(
        "margin_to_second_delta".to_string(),
        serde_json::json!((margin_to_second_delta * 1_000_000.0).round() / 1_000_000.0),
    );
}
