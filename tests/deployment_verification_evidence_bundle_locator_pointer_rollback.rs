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
fn deployment_verification_bundle_locator_pointer_rollback_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_verification_evidence_bundle_locator_pointer_rollback.schema.json",
    ))
    .expect("read deployment verification bundle locator rollback schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment verification bundle locator rollback schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/deployment-verification-evidence-bundle-locator-pointer-rollback/v1"
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
        "current_pointer_path",
        "restored_pointer_path",
        "previous_locator_path",
        "restored_locator_path",
        "previous_bundle_path",
        "restored_bundle_path",
        "previous_artifact_version",
        "restored_artifact_version",
        "profile_name",
        "rolled_back_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains(
            "deployment-verification-rollback-bundle-locator current_pointer restored_pointer rolled_back_at_unix_ms:"
        ),
        "justfile must expose the deployment-verification-rollback-bundle-locator workflow"
    );

    let workflows = repo_file("docs/workflows.md");
    assert!(
        workflows.contains("deployment_verification_evidence_bundle_locator_pointer_rollback.json"),
        "workflow reference must mention the deployment verification bundle locator rollback artifact"
    );
    assert!(
        workflows.contains("just deployment-verification-rollback-bundle-locator"),
        "workflow reference must mention the deployment verification bundle locator rollback workflow"
    );
}

#[test]
fn deployment_verification_bundle_locator_pointer_rollback_writes_pointer_and_rollback_record() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let current_pointer = tmp.path().join("current-pointer.json");
    let restored_pointer = tmp.path().join("restored-pointer.json");
    fs::write(
        &current_pointer,
        r#"{
  "schema_version": "1",
  "locator_path": "artifacts/deploy/deployment_verification_evidence_bundle_transport_locator.json",
  "bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.json",
  "artifact_version": "0.2.0",
  "profile_name": "staging-a",
  "verification_status": "passed"
}"#,
    )
    .expect("write current pointer");
    fs::write(
        &restored_pointer,
        r#"{
  "schema_version": "1",
  "locator_path": "artifacts/deploy/deployment_verification_evidence_bundle_transport_locator.previous.json",
  "bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.previous.json",
  "artifact_version": "0.1.0",
  "profile_name": "staging-a",
  "verification_status": "passed"
}"#,
    )
    .expect("write restored pointer");

    let out_pointer = tmp.path().join("pointer-out.json");
    let out_record = tmp.path().join("rollback.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("rollback")
        .arg("verification-bundle-locator")
        .arg("--current-pointer")
        .arg(&current_pointer)
        .arg("--restored-pointer")
        .arg(&restored_pointer)
        .arg("--rolled-back-at-unix-ms")
        .arg("1735689800000")
        .arg("--out-pointer")
        .arg(&out_pointer)
        .arg("--out-record")
        .arg(&out_record);
    let assert = cmd.assert().success();

    let pointer_text = fs::read_to_string(&out_pointer).expect("read restored pointer");
    let pointer: Value = serde_json::from_str(&pointer_text).expect("parse restored pointer");
    assert_eq!(
        pointer,
        serde_json::json!({
            "schema_version": "1",
            "locator_path": "artifacts/deploy/deployment_verification_evidence_bundle_transport_locator.previous.json",
            "bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.previous.json",
            "artifact_version": "0.1.0",
            "profile_name": "staging-a",
            "verification_status": "passed"
        })
    );

    let rollback_text = fs::read_to_string(&out_record).expect("read rollback record");
    let mut rollback: Value = serde_json::from_str(&rollback_text).expect("parse rollback record");
    let object = rollback.as_object_mut().expect("rollback must be object");
    object.insert("current_pointer_path".to_string(), Value::from("<current>"));
    object.insert(
        "restored_pointer_path".to_string(),
        Value::from("<restored>"),
    );
    assert_eq!(
        rollback,
        serde_json::json!({
            "schema_version": "1",
            "current_pointer_path": "<current>",
            "restored_pointer_path": "<restored>",
            "previous_locator_path": "artifacts/deploy/deployment_verification_evidence_bundle_transport_locator.json",
            "restored_locator_path": "artifacts/deploy/deployment_verification_evidence_bundle_transport_locator.previous.json",
            "previous_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.json",
            "restored_bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.previous.json",
            "previous_artifact_version": "0.2.0",
            "restored_artifact_version": "0.1.0",
            "profile_name": "staging-a",
            "rolled_back_at_unix_ms": 1735689800000_u64
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("deployment_verification_evidence_bundle_locator_pointer_rolled_back")
        })
        .unwrap_or_else(|| {
            panic!(
                "missing deployment verification bundle locator rollback event in stderr: {stderr}"
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
    fields.insert("pointer_path".to_string(), Value::from("<pointer-out>"));
    fields.insert("rollback_path".to_string(), Value::from("<rollback-out>"));
    let rollback_obj = fields
        .get_mut("rollback")
        .and_then(Value::as_object_mut)
        .expect("event rollback must be object");
    rollback_obj.insert("current_pointer_path".to_string(), Value::from("<current>"));
    rollback_obj.insert(
        "restored_pointer_path".to_string(),
        Value::from("<restored>"),
    );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_verification_evidence_bundle_locator_pointer_rolled_back",
            "fields": {
                "pointer_path": "<pointer-out>",
                "rollback_path": "<rollback-out>",
                "pointer": pointer,
                "rollback": rollback
            }
        })
    );
}
