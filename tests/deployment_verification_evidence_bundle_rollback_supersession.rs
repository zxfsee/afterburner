use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

#[path = "support/fixture.rs"]
mod fixture_test_support;

use fixture_test_support::fixture_path;

#[test]
fn deployment_verification_bundle_rollback_supersession_schema_is_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_verification_evidence_bundle_rollback_supersession.schema.json",
    ))
    .expect("read deployment verification bundle rollback supersession schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment verification bundle rollback supersession schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/deployment-verification-evidence-bundle-rollback-supersession/v1"
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
                .expect("schema.required items must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "previous_rollback_path",
        "next_rollback_path",
        "previous_restored_bundle_path",
        "next_restored_bundle_path",
        "previous_restored_artifact_version",
        "next_restored_artifact_version",
        "profile_name",
        "superseded_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn deployment_verification_bundle_rollback_supersession_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let previous = tmp.path().join("previous.json");
    let next = tmp.path().join("next.json");
    fs::write(
        &previous,
        r#"{
  "schema_version": "1",
  "current_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.json",
  "restored_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.previous.json",
  "previous_receipt_path": "artifacts/deploy/deployment_verification_receipt.json",
  "restored_receipt_path": "artifacts/deploy/deployment_verification_receipt.previous.json",
  "previous_artifact_version": "0.2.0",
  "restored_artifact_version": "0.1.0",
  "profile_name": "staging-a",
  "previous_verification_status": "passed",
  "restored_verification_status": "passed",
  "previous_evidence_count": 2,
  "restored_evidence_count": 1,
  "rolled_back_at_unix_ms": 1735689800000
}"#,
    )
    .expect("write previous rollback");
    fs::write(
        &next,
        r#"{
  "schema_version": "1",
  "current_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.json",
  "restored_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.older.json",
  "previous_receipt_path": "artifacts/deploy/deployment_verification_receipt.json",
  "restored_receipt_path": "artifacts/deploy/deployment_verification_receipt.older.json",
  "previous_artifact_version": "0.2.0",
  "restored_artifact_version": "0.0.9",
  "profile_name": "staging-a",
  "previous_verification_status": "passed",
  "restored_verification_status": "passed",
  "previous_evidence_count": 2,
  "restored_evidence_count": 1,
  "rolled_back_at_unix_ms": 1735689850000
}"#,
    )
    .expect("write next rollback");

    let out = tmp.path().join("supersession.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("verification-bundle")
        .arg("supersede-rollback")
        .arg("--previous-rollback")
        .arg(&previous)
        .arg("--next-rollback")
        .arg(&next)
        .arg("--superseded-at-unix-ms")
        .arg("1735689900000")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read supersession");
    let mut supersession: Value = serde_json::from_str(&text).expect("parse supersession");
    let object = supersession
        .as_object_mut()
        .expect("supersession must be object");
    object.insert(
        "previous_rollback_path".to_string(),
        Value::from("<previous>"),
    );
    object.insert("next_rollback_path".to_string(), Value::from("<next>"));
    assert_eq!(
        supersession,
        serde_json::json!({
            "schema_version": "1",
            "previous_rollback_path": "<previous>",
            "next_rollback_path": "<next>",
            "previous_restored_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.previous.json",
            "next_restored_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.older.json",
            "previous_restored_artifact_version": "0.1.0",
            "next_restored_artifact_version": "0.0.9",
            "profile_name": "staging-a",
            "superseded_at_unix_ms": 1735689900000_u64
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("deployment_verification_evidence_bundle_rollback_superseded")
        })
        .unwrap_or_else(|| {
            panic!(
                "missing deployment verification bundle rollback supersession event in stderr: {stderr}"
            )
        });
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be object");
    fields.insert("supersession_path".to_string(), Value::from("<out>"));
    let supersession_obj = fields
        .get_mut("supersession")
        .and_then(Value::as_object_mut)
        .expect("event supersession must be object");
    supersession_obj.insert(
        "previous_rollback_path".to_string(),
        Value::from("<previous>"),
    );
    supersession_obj.insert("next_rollback_path".to_string(), Value::from("<next>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_verification_evidence_bundle_rollback_superseded",
            "fields": {
                "supersession_path": "<out>",
                "supersession": supersession
            }
        })
    );
}
