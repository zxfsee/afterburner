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

#[test]
fn deployment_verification_bundle_transport_locator_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_verification_evidence_bundle_transport_locator.schema.json",
    ))
    .expect("read deployment verification bundle transport locator schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment verification bundle transport locator schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/deployment-verification-evidence-bundle-transport-locator/v1"
        )
    );
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("required entries must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "bundle_path",
        "artifact_version",
        "profile_name",
        "verification_status",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn deployment_verification_bundle_transport_locator_writes_locator_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bundle = tmp
        .path()
        .join("deployment_verification_evidence_bundle.json");
    fs::write(
        &bundle,
        r#"{
  "schema_version": "1",
  "receipt_path": "artifacts/deploy/deployment_verification_receipt.json",
  "artifact_version": "0.2.0",
  "profile_name": "staging-a",
  "verification_status": "passed",
  "evidence_count": 2,
  "evidence_sources": [
    {"artifact_path":"artifacts/eval/mnist_eval_summary.json","event_name":"eval_done","observed_at_unix_ms":1735689601000},
    {"artifact_path":"artifacts/deploy/candidate_upload_request.json","event_name":"artifact_upload_request_written","observed_at_unix_ms":1735689602000}
  ]
}"#,
    )
    .expect("write deployment verification evidence bundle");

    let out = tmp
        .path()
        .join("deployment_verification_evidence_bundle_transport_locator.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("verification-bundle")
        .arg("point-transport-locator")
        .arg("--bundle")
        .arg(&bundle)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let locator_text = fs::read_to_string(&out).expect("read transport locator");
    let mut locator: Value = serde_json::from_str(&locator_text).expect("parse transport locator");
    locator
        .as_object_mut()
        .expect("locator must be a top-level object")
        .insert("bundle_path".to_string(), Value::from("<bundle>"));
    let expected_text = fs::read_to_string(fixture_path(
        "deployment_verification_evidence_bundle_transport_locator.example.json",
    ))
    .expect("read deployment verification bundle transport locator example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse deployment verification bundle transport locator example");
    expected
        .as_object_mut()
        .expect("expected locator must be a top-level object")
        .insert("bundle_path".to_string(), Value::from("<bundle>"));
    assert_eq!(locator, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("deployment_verification_evidence_bundle_transport_locator_written")
        })
        .unwrap_or_else(|| {
            panic!(
                "missing deployment verification bundle transport locator event in stderr: {stderr}"
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
    fields.insert("locator_path".to_string(), Value::from("<locator>"));
    let locator_obj = fields
        .get_mut("locator")
        .and_then(Value::as_object_mut)
        .expect("event locator must be an object");
    locator_obj.insert("bundle_path".to_string(), Value::from("<bundle>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_verification_evidence_bundle_transport_locator_written",
            "fields": {
                "locator_path": "<locator>",
                "locator": expected
            }
        })
    );
}
