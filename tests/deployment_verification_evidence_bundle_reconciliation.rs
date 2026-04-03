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
fn deployment_verification_evidence_bundle_reconciliation_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_verification_evidence_bundle_reconciliation.schema.json",
    ))
    .expect("read deployment verification evidence bundle reconciliation schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment verification evidence bundle reconciliation schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/deployment-verification-evidence-bundle-reconciliation/v1"
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
        "receipt_path",
        "bundle_path",
        "desired",
        "current",
        "reconciliation_status",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);
}

#[test]
fn deployment_verification_evidence_bundle_reconciliation_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let receipt = tmp.path().join("deployment_verification_receipt.json");
    fs::write(
        &receipt,
        r#"{"schema_version":"1","artifact_version":"0.2.0","profile_name":"staging-a","verification_status":"passed","verified_at_unix_ms":1735689600000,"evidence":"local verify","evidence_sources":[{"artifact_path":"artifacts/eval/mnist_eval_summary.json","event_name":"eval_done","observed_at_unix_ms":1735689601000},{"artifact_path":"artifacts/deploy/candidate_upload_request.json","event_name":"artifact_upload_request_written","observed_at_unix_ms":1735689602000}]}"#,
    )
    .expect("write deployment verification receipt");
    let bundle = tmp
        .path()
        .join("deployment_verification_evidence_bundle.json");
    fs::write(
        &bundle,
        format!(
            r#"{{
  "schema_version": "1",
  "receipt_path": "{}",
  "artifact_version": "0.2.0",
  "profile_name": "staging-a",
  "verification_status": "passed",
  "evidence_count": 2,
  "evidence_sources": [
    {{"artifact_path":"artifacts/eval/mnist_eval_summary.json","event_name":"eval_done","observed_at_unix_ms":1735689601000}},
    {{"artifact_path":"artifacts/deploy/candidate_upload_request.json","event_name":"artifact_upload_request_written","observed_at_unix_ms":1735689602000}}
  ]
}}"#,
            receipt.display()
        ),
    )
    .expect("write deployment verification evidence bundle");
    let out = tmp
        .path()
        .join("deployment_verification_evidence_bundle_reconciliation.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("verification-bundle")
        .arg("reconcile")
        .arg("--receipt")
        .arg(&receipt)
        .arg("--bundle")
        .arg(&bundle)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read reconciliation");
    let mut reconciliation: Value = serde_json::from_str(&text).expect("parse reconciliation");
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("receipt_path".to_string(), Value::from("<receipt>"));
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("bundle_path".to_string(), Value::from("<bundle>"));
    let desired = reconciliation
        .get_mut("desired")
        .and_then(Value::as_object_mut)
        .expect("desired object");
    desired.insert("receipt_path".to_string(), Value::from("<receipt>"));
    let current = reconciliation
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("current object");
    current.insert("receipt_path".to_string(), Value::from("<receipt>"));

    let expected_text = fs::read_to_string(fixture_path(
        "deployment_verification_evidence_bundle_reconciliation.example.json",
    ))
    .expect("read deployment verification evidence bundle reconciliation example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse deployment verification evidence bundle reconciliation example");
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("receipt_path".to_string(), Value::from("<receipt>"));
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("bundle_path".to_string(), Value::from("<bundle>"));
    let expected_desired = expected
        .get_mut("desired")
        .and_then(Value::as_object_mut)
        .expect("expected desired object");
    expected_desired.insert("receipt_path".to_string(), Value::from("<receipt>"));
    let expected_current = expected
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("expected current object");
    expected_current.insert("receipt_path".to_string(), Value::from("<receipt>"));
    assert_eq!(reconciliation, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("deployment_verification_evidence_bundle_reconciliation_written")
        })
        .unwrap_or_else(|| {
            panic!(
                "missing deployment verification evidence bundle reconciliation event in stderr: {stderr}"
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
    reconciliation_obj.insert("receipt_path".to_string(), Value::from("<receipt>"));
    reconciliation_obj.insert("bundle_path".to_string(), Value::from("<bundle>"));
    let event_desired = reconciliation_obj
        .get_mut("desired")
        .and_then(Value::as_object_mut)
        .expect("event desired object");
    event_desired.insert("receipt_path".to_string(), Value::from("<receipt>"));
    let event_current = reconciliation_obj
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("event current object");
    event_current.insert("receipt_path".to_string(), Value::from("<receipt>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_verification_evidence_bundle_reconciliation_written",
            "fields": {
                "reconciliation_path": "<reconciliation>",
                "reconciliation": expected
            }
        })
    );
}
