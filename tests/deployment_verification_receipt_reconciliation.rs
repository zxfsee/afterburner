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
fn deployment_verification_receipt_reconciliation_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_verification_receipt_reconciliation.schema.json",
    ))
    .expect("read deployment verification receipt reconciliation schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment verification receipt reconciliation schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/deployment-verification-receipt-reconciliation/v1")
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
        justfile.contains("deployment-verification-reconcile-receipt receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2:"),
        "justfile must expose the deployment-verification-reconcile-receipt workflow"
    );

    let workflows = repo_file("docs/workflows.md");
    assert!(
        workflows.contains("deployment_verification_receipt_reconciliation.json"),
        "workflow reference must mention the deployment verification receipt reconciliation artifact"
    );
    assert!(
        workflows.contains("just deployment-verification-reconcile-receipt"),
        "workflow reference must mention the deployment verification receipt reconciliation workflow"
    );

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("deployment_verification_receipt_reconciliation.json"),
        "reference index must mention the deployment verification receipt reconciliation artifact"
    );
}

#[test]
fn deployment_verification_receipt_reconciliation_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let receipt = tmp.path().join("deployment_verification_receipt.json");
    fs::write(
        &receipt,
        r#"{"schema_version":"1","artifact_version":"0.2.0","profile_name":"staging-a","verification_status":"passed","verified_at_unix_ms":1735689600000,"evidence":"local verify","evidence_sources":[{"artifact_path":"artifacts/eval/mnist_eval_summary.json","event_name":"eval_done","observed_at_unix_ms":1735689601000},{"artifact_path":"artifacts/deploy/candidate_upload_request.json","event_name":"artifact_upload_request_written","observed_at_unix_ms":1735689602000}]}"#,
    )
    .expect("write deployment verification receipt");
    let out = tmp
        .path()
        .join("deployment_verification_receipt_reconciliation.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("debug")
        .arg("deploy")
        .arg("reconcile-verification-receipt")
        .arg("--receipt")
        .arg(&receipt)
        .arg("--artifact-version")
        .arg("0.2.0")
        .arg("--profile-name")
        .arg("staging-a")
        .arg("--verification-status")
        .arg("passed")
        .arg("--verified-at-unix-ms")
        .arg("1735689600000")
        .arg("--evidence")
        .arg("local verify")
        .arg("--evidence-source")
        .arg("artifacts/eval/mnist_eval_summary.json,eval_done,1735689601000")
        .arg("--evidence-source")
        .arg("artifacts/deploy/candidate_upload_request.json,artifact_upload_request_written,1735689602000")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read reconciliation");
    let mut reconciliation: Value = serde_json::from_str(&text).expect("parse reconciliation");
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("receipt_path".to_string(), Value::from("<receipt>"));

    let expected_text = fs::read_to_string(fixture_path(
        "deployment_verification_receipt_reconciliation.example.json",
    ))
    .expect("read deployment verification receipt reconciliation example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse deployment verification receipt reconciliation example");
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("receipt_path".to_string(), Value::from("<receipt>"));
    assert_eq!(reconciliation, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("deployment_verification_receipt_reconciliation_written")
        })
        .unwrap_or_else(|| {
            panic!(
                "missing deployment verification receipt reconciliation event in stderr: {stderr}"
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
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_verification_receipt_reconciliation_written",
            "fields": {
                "reconciliation_path": "<reconciliation>",
                "reconciliation": expected
            }
        })
    );
}
