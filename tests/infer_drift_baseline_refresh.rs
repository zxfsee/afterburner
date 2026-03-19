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
fn infer_output_drift_baseline_refresh_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "infer_output_drift_baseline_refresh.schema.json",
    ))
    .expect("read infer drift baseline refresh schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse infer drift baseline refresh");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/infer-output-drift-baseline-refresh/v1")
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
        "previous_baseline_path",
        "archived_baseline_path",
        "refreshed_baseline_path",
        "summary_path",
        "receipt_path",
        "previous_artifact_version",
        "next_artifact_version",
        "policy_profile",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("drift-refresh-baseline summary receipt current_baseline:"),
        "justfile must expose the drift-refresh-baseline workflow"
    );

    let readme = repo_file("README.md");
    assert!(
        readme.contains("infer_output_drift_baseline_refresh.json"),
        "README must mention the baseline refresh artifact"
    );
    assert!(
        readme.contains("just drift-refresh-baseline"),
        "README must mention the baseline refresh workflow"
    );
}

#[test]
fn drift_refresh_baseline_archives_previous_baseline_and_writes_refresh_receipt() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let summary = tmp.path().join("summary.json");
    let receipt = tmp.path().join("receipt.json");
    let current_baseline = tmp.path().join("baseline.json");
    let out = tmp.path().join("refreshed_baseline.json");
    let archive = tmp.path().join("baseline.previous.json");
    let refresh = tmp.path().join("baseline_refresh.json");

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
    fs::write(
        &current_baseline,
        r#"{"schema_version":"1","summary_path":"old-summary.json","receipt_path":"old-receipt.json","policy_path":"old-policy.json","policy_profile":"promotion-default","artifact":"artifacts/inference/current/model.mpk","artifact_version":"0.1.0","backend":"cpu","predicted_class":3,"top_probability":0.89,"margin_to_second":0.40,"logits_sha256":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","probabilities_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"}"#,
    )
    .expect("write current baseline");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("drift-refresh-baseline")
        .arg("--summary")
        .arg(&summary)
        .arg("--receipt")
        .arg(&receipt)
        .arg("--current-baseline")
        .arg(&current_baseline)
        .arg("--out")
        .arg(&out)
        .arg("--archive")
        .arg(&archive)
        .arg("--refresh-receipt")
        .arg(&refresh);
    let assert = cmd.assert().success();

    assert!(
        archive.exists(),
        "refresh must archive the previous baseline"
    );
    assert!(out.exists(), "refresh must write the refreshed baseline");
    assert!(refresh.exists(), "refresh must write the refresh receipt");

    let archived_text = fs::read_to_string(&archive).expect("read archived baseline");
    assert!(
        archived_text.contains("\"artifact_version\":\"0.1.0\""),
        "archived baseline must preserve the previous artifact version"
    );

    let refresh_text = fs::read_to_string(&refresh).expect("read refresh receipt");
    let mut refresh_value: Value =
        serde_json::from_str(&refresh_text).expect("parse refresh receipt");
    normalize_refresh_receipt(&mut refresh_value);
    assert_eq!(
        refresh_value,
        serde_json::json!({
            "schema_version": "1",
            "previous_baseline_path": "<current-baseline>",
            "archived_baseline_path": "<archive>",
            "refreshed_baseline_path": "<out>",
            "summary_path": "<summary>",
            "receipt_path": "<receipt>",
            "previous_artifact_version": "0.1.0",
            "next_artifact_version": "0.2.0",
            "policy_profile": "promotion-default"
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "infer_output_drift_baseline_refreshed");
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("refresh_receipt_path".to_string(), Value::from("<refresh>"));
    let refresh_obj = fields
        .get_mut("refresh_receipt")
        .and_then(Value::as_object_mut)
        .expect("event refresh_receipt must be an object");
    normalize_refresh_receipt_object(refresh_obj);
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "infer_cli",
            "event": "infer_output_drift_baseline_refreshed",
            "fields": {
                "refresh_receipt_path": "<refresh>",
                "refresh_receipt": refresh_value
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

fn normalize_refresh_receipt(value: &mut Value) {
    let object = value
        .as_object_mut()
        .expect("refresh receipt must be a top-level object");
    normalize_refresh_receipt_object(object);
}

fn normalize_refresh_receipt_object(object: &mut serde_json::Map<String, Value>) {
    object.insert(
        "previous_baseline_path".to_string(),
        Value::from("<current-baseline>"),
    );
    object.insert(
        "archived_baseline_path".to_string(),
        Value::from("<archive>"),
    );
    object.insert("refreshed_baseline_path".to_string(), Value::from("<out>"));
    object.insert("summary_path".to_string(), Value::from("<summary>"));
    object.insert("receipt_path".to_string(), Value::from("<receipt>"));
}
