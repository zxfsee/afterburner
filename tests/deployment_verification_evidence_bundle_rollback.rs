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
fn deployment_verification_bundle_rollback_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_verification_evidence_bundle_rollback.schema.json",
    ))
    .expect("read deployment verification bundle rollback schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment verification bundle rollback schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/deployment-verification-evidence-bundle-rollback/v1"
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
        "current_bundle_path",
        "restored_bundle_path",
        "previous_receipt_path",
        "restored_receipt_path",
        "previous_artifact_version",
        "restored_artifact_version",
        "profile_name",
        "previous_verification_status",
        "restored_verification_status",
        "previous_evidence_count",
        "restored_evidence_count",
        "rolled_back_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains(
            "deployment-verification-rollback-bundle current_bundle restored_bundle rolled_back_at_unix_ms:"
        ),
        "justfile must expose the deployment-verification-rollback-bundle workflow"
    );

    let workflows = repo_file("docs/workflows.md");
    assert!(
        workflows.contains("deployment_verification_evidence_bundle_rollback.json"),
        "workflow reference must mention the deployment verification bundle rollback artifact"
    );
    assert!(
        workflows.contains("just deployment-verification-rollback-bundle"),
        "workflow reference must mention the deployment verification bundle rollback workflow"
    );
}

#[test]
fn deployment_verification_bundle_rollback_writes_bundle_and_rollback_record() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let current_bundle = tmp.path().join("current-bundle.json");
    let restored_bundle = tmp.path().join("restored-bundle.json");
    fs::write(
        &current_bundle,
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
    .expect("write current bundle");
    fs::write(
        &restored_bundle,
        r#"{
  "schema_version": "1",
  "receipt_path": "artifacts/deploy/deployment_verification_receipt.previous.json",
  "artifact_version": "0.1.0",
  "profile_name": "staging-a",
  "verification_status": "passed",
  "evidence_count": 1,
  "evidence_sources": [
    {"artifact_path":"artifacts/eval/mnist_eval_summary.previous.json","event_name":"eval_done","observed_at_unix_ms":1735689500000}
  ]
}"#,
    )
    .expect("write restored bundle");

    let out_bundle = tmp.path().join("bundle-out.json");
    let out_record = tmp.path().join("rollback.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("rollback-verification-bundle")
        .arg("--current-bundle")
        .arg(&current_bundle)
        .arg("--restored-bundle")
        .arg(&restored_bundle)
        .arg("--rolled-back-at-unix-ms")
        .arg("1735689800000")
        .arg("--out-bundle")
        .arg(&out_bundle)
        .arg("--out-record")
        .arg(&out_record);
    let assert = cmd.assert().success();

    let bundle_text = fs::read_to_string(&out_bundle).expect("read restored bundle");
    let bundle: Value = serde_json::from_str(&bundle_text).expect("parse restored bundle");
    assert_eq!(
        bundle,
        serde_json::json!({
            "schema_version": "1",
            "receipt_path": "artifacts/deploy/deployment_verification_receipt.previous.json",
            "artifact_version": "0.1.0",
            "profile_name": "staging-a",
            "verification_status": "passed",
            "evidence_count": 1,
            "evidence_sources": [
                {
                    "artifact_path": "artifacts/eval/mnist_eval_summary.previous.json",
                    "event_name": "eval_done",
                    "observed_at_unix_ms": 1735689500000_u64
                }
            ]
        })
    );

    let rollback_text = fs::read_to_string(&out_record).expect("read rollback record");
    let mut rollback: Value = serde_json::from_str(&rollback_text).expect("parse rollback record");
    let object = rollback.as_object_mut().expect("rollback must be object");
    object.insert("current_bundle_path".to_string(), Value::from("<current>"));
    object.insert(
        "restored_bundle_path".to_string(),
        Value::from("<restored>"),
    );
    assert_eq!(
        rollback,
        serde_json::json!({
            "schema_version": "1",
            "current_bundle_path": "<current>",
            "restored_bundle_path": "<restored>",
            "previous_receipt_path": "artifacts/deploy/deployment_verification_receipt.json",
            "restored_receipt_path": "artifacts/deploy/deployment_verification_receipt.previous.json",
            "previous_artifact_version": "0.2.0",
            "restored_artifact_version": "0.1.0",
            "profile_name": "staging-a",
            "previous_verification_status": "passed",
            "restored_verification_status": "passed",
            "previous_evidence_count": 2_u64,
            "restored_evidence_count": 1_u64,
            "rolled_back_at_unix_ms": 1735689800000_u64
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("deployment_verification_evidence_bundle_rolled_back")
        })
        .unwrap_or_else(|| {
            panic!("missing deployment verification bundle rollback event in stderr: {stderr}")
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
    fields.insert("bundle_path".to_string(), Value::from("<bundle-out>"));
    fields.insert("rollback_path".to_string(), Value::from("<rollback-out>"));
    let rollback_obj = fields
        .get_mut("rollback")
        .and_then(Value::as_object_mut)
        .expect("event rollback must be object");
    rollback_obj.insert("current_bundle_path".to_string(), Value::from("<current>"));
    rollback_obj.insert(
        "restored_bundle_path".to_string(),
        Value::from("<restored>"),
    );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_verification_evidence_bundle_rolled_back",
            "fields": {
                "bundle_path": "<bundle-out>",
                "rollback_path": "<rollback-out>",
                "bundle": bundle,
                "rollback": rollback
            }
        })
    );
}
