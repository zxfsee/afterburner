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
fn deployment_verification_receipt_locator_pointer_reconciliation_schema_and_workflow_are_explicit()
{
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_verification_receipt_locator_pointer_reconciliation.schema.json",
    ))
    .expect("read deployment verification receipt locator pointer reconciliation schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment verification receipt locator pointer reconciliation schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/deployment-verification-receipt-locator-pointer-reconciliation/v1"
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
        "locator_path",
        "pointer_path",
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
        justfile.contains("deployment-verification-reconcile-receipt-locator locator pointer:"),
        "justfile must expose the deployment-verification-reconcile-receipt-locator workflow"
    );

    let workflows = repo_file("docs/workflows.md");
    assert!(
        workflows.contains("deployment_verification_receipt_locator_pointer_reconciliation.json"),
        "workflow reference must mention the deployment verification receipt locator pointer reconciliation artifact"
    );
    assert!(
        workflows.contains("just deployment-verification-reconcile-receipt-locator"),
        "workflow reference must mention the deployment verification receipt locator pointer reconciliation workflow"
    );
}

#[test]
fn deployment_verification_receipt_locator_pointer_reconciliation_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let locator = tmp
        .path()
        .join("deployment_verification_receipt_transport_locator.json");
    fs::write(
        &locator,
        r#"{
  "schema_version": "1",
  "receipt_path": "artifacts/deploy/deployment_verification_receipt.json",
  "artifact_version": "0.2.0",
  "profile_name": "staging-a",
  "verification_status": "passed"
}"#,
    )
    .expect("write transport locator");
    let pointer = tmp
        .path()
        .join("deployment_verification_receipt_locator_pointer.json");
    fs::write(
        &pointer,
        r#"{
  "schema_version": "1",
  "locator_path": "artifacts/deploy/deployment_verification_receipt_transport_locator.json",
  "receipt_path": "artifacts/deploy/deployment_verification_receipt.json",
  "artifact_version": "0.2.0",
  "profile_name": "staging-a",
  "verification_status": "passed"
}"#,
    )
    .expect("write locator pointer");
    let out = tmp
        .path()
        .join("deployment_verification_receipt_locator_pointer_reconciliation.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("reconcile-verification-receipt-locator")
        .arg("--locator")
        .arg(&locator)
        .arg("--pointer")
        .arg(&pointer)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read reconciliation");
    let mut reconciliation: Value = serde_json::from_str(&text).expect("parse reconciliation");
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("locator_path".to_string(), Value::from("<locator>"));
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("pointer_path".to_string(), Value::from("<pointer>"));
    let current = reconciliation
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("current object");
    current.insert("locator_path".to_string(), Value::from("<locator>"));

    let expected_text = fs::read_to_string(fixture_path(
        "deployment_verification_receipt_locator_pointer_reconciliation.example.json",
    ))
    .expect("read deployment verification receipt locator pointer reconciliation example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse deployment verification receipt locator pointer reconciliation example");
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("locator_path".to_string(), Value::from("<locator>"));
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("pointer_path".to_string(), Value::from("<pointer>"));
    let expected_current = expected
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("expected current object");
    expected_current.insert("locator_path".to_string(), Value::from("<locator>"));
    assert_eq!(reconciliation, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("deployment_verification_receipt_locator_pointer_reconciliation_written")
        })
        .unwrap_or_else(|| {
            panic!(
                "missing deployment verification receipt locator pointer reconciliation event in stderr: {stderr}"
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
    let recon_obj = fields
        .get_mut("reconciliation")
        .and_then(Value::as_object_mut)
        .expect("event reconciliation must be an object");
    recon_obj.insert("locator_path".to_string(), Value::from("<locator>"));
    recon_obj.insert("pointer_path".to_string(), Value::from("<pointer>"));
    let event_current = recon_obj
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("event current object");
    event_current.insert("locator_path".to_string(), Value::from("<locator>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_verification_receipt_locator_pointer_reconciliation_written",
            "fields": {
                "reconciliation_path": "<reconciliation>",
                "reconciliation": expected
            }
        })
    );
}
