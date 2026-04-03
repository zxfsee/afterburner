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
fn deployment_verification_bundle_rollback_reconciliation_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_verification_evidence_bundle_rollback_reconciliation.schema.json",
    ))
    .expect("read deployment verification bundle rollback reconciliation schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment verification bundle rollback reconciliation schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/deployment-verification-evidence-bundle-rollback-reconciliation/v1"
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
        "rollback_path",
        "current_rollback_path",
        "desired",
        "current",
        "reconciliation_status",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains(
            "deployment-verification-reconcile-bundle-rollback rollback current_rollback:"
        ),
        "justfile must expose the deployment-verification-reconcile-bundle-rollback workflow"
    );
}

#[test]
fn deployment_verification_bundle_rollback_reconciliation_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let rollback = tmp.path().join("rollback.json");
    fs::write(
        &rollback,
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
    .expect("write desired rollback");
    let current_rollback = tmp.path().join("current-rollback.json");
    fs::write(
        &current_rollback,
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
    .expect("write current rollback");
    let out = tmp
        .path()
        .join("deployment_verification_evidence_bundle_rollback_reconciliation.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("reconcile-verification-bundle-rollback")
        .arg("--rollback")
        .arg(&rollback)
        .arg("--current-rollback")
        .arg(&current_rollback)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read reconciliation");
    let mut reconciliation: Value = serde_json::from_str(&text).expect("parse reconciliation");
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("rollback_path".to_string(), Value::from("<rollback>"));
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert(
            "current_rollback_path".to_string(),
            Value::from("<current>"),
        );
    let current = reconciliation
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("current object");
    current.insert(
        "current_rollback_path".to_string(),
        Value::from("<current>"),
    );

    let expected_text = fs::read_to_string(fixture_path(
        "deployment_verification_evidence_bundle_rollback_reconciliation.example.json",
    ))
    .expect("read deployment verification bundle rollback reconciliation example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse deployment verification bundle rollback reconciliation example");
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("rollback_path".to_string(), Value::from("<rollback>"));
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert(
            "current_rollback_path".to_string(),
            Value::from("<current>"),
        );
    let expected_current = expected
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("expected current object");
    expected_current.insert(
        "current_rollback_path".to_string(),
        Value::from("<current>"),
    );
    assert_eq!(reconciliation, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("deployment_verification_evidence_bundle_rollback_reconciliation_written")
        })
        .unwrap_or_else(|| {
            panic!(
                "missing deployment verification bundle rollback reconciliation event in stderr: {stderr}"
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
    fields.insert(
        "reconciliation_path".to_string(),
        Value::from("<reconciliation>"),
    );
    let reconciliation_obj = fields
        .get_mut("reconciliation")
        .and_then(Value::as_object_mut)
        .expect("event reconciliation must be an object");
    reconciliation_obj.insert("rollback_path".to_string(), Value::from("<rollback>"));
    reconciliation_obj.insert(
        "current_rollback_path".to_string(),
        Value::from("<current>"),
    );
    let event_current = reconciliation_obj
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("event current object");
    event_current.insert(
        "current_rollback_path".to_string(),
        Value::from("<current>"),
    );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_verification_evidence_bundle_rollback_reconciliation_written",
            "fields": {
                "reconciliation_path": "<reconciliation>",
                "reconciliation": expected
            }
        })
    );
}
