use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use fixture_test_support::fixture_path;
use repo_test_support::repo_file;

#[test]
fn infer_output_drift_baseline_transport_locator_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "infer_output_drift_baseline_transport_locator.schema.json",
    ))
    .expect("read infer drift baseline transport locator schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse infer drift baseline transport locator schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/infer-output-drift-baseline-transport-locator/v1")
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
        "handoff_path",
        "artifact",
        "artifact_version",
        "policy_profile",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("infer_output_drift_baseline_transport_locator.json"),
        "workflow reference must mention the baseline transport locator artifact"
    );
}

#[test]
fn drift_point_baseline_transport_locator_writes_locator_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handoff = tmp.path().join("handoff.json");
    fs::write(
        &handoff,
        r#"{"schema_version":"1","bundle_path":"bundle.json","approval_path":"approval.json","baseline_path":"baseline.json","artifact":"artifacts/inference/0.2.0/model.mpk","artifact_version":"0.2.0","policy_profile":"promotion-default"}"#,
    )
    .expect("write handoff");

    let out = tmp.path().join("transport_locator.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("drift")
        .arg("point-baseline-transport-locator")
        .arg("--handoff")
        .arg(&handoff)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let locator_text = fs::read_to_string(&out).expect("read locator");
    let mut locator: Value = serde_json::from_str(&locator_text).expect("parse locator");
    locator
        .as_object_mut()
        .expect("locator must be a top-level object")
        .insert("handoff_path".to_string(), Value::from("<handoff>"));
    assert_eq!(
        locator,
        serde_json::json!({
            "schema_version": "1",
            "handoff_path": "<handoff>",
            "artifact": "artifacts/inference/0.2.0/model.mpk",
            "artifact_version": "0.2.0",
            "policy_profile": "promotion-default"
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(
        &stderr,
        "infer_output_drift_baseline_transport_locator_written",
    );
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("locator_path".to_string(), Value::from("<locator>"));
    let locator_obj = fields
        .get_mut("locator")
        .and_then(Value::as_object_mut)
        .expect("event locator must be an object");
    locator_obj.insert("handoff_path".to_string(), Value::from("<handoff>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "infer_cli",
            "event": "infer_output_drift_baseline_transport_locator_written",
            "fields": {
                "locator_path": "<locator>",
                "locator": locator
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
