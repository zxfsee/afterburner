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
fn deployment_verification_evidence_bundle_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_verification_evidence_bundle.schema.json",
    ))
    .expect("read deployment verification evidence bundle schema fixture");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment verification evidence bundle schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/deployment-verification-evidence-bundle/v1")
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
        "receipt_path",
        "artifact_version",
        "profile_name",
        "verification_status",
        "evidence_count",
        "evidence_sources",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("deployment-verification-bundle receipt:"),
        "justfile must expose the deployment-verification-bundle workflow"
    );

    let readme = repo_file("docs/workflows.md");
    assert!(
        readme.contains("deployment_verification_evidence_bundle.json"),
        "workflow reference must mention the deployment verification evidence bundle artifact"
    );
    assert!(
        readme.contains("just deployment-verification-bundle"),
        "workflow reference must mention the deployment verification evidence bundle workflow"
    );
}

#[test]
fn deployment_verification_bundle_writes_bundle_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let receipt = tmp.path().join("deployment_verification_receipt.json");
    fs::write(
        &receipt,
        r#"{"schema_version":"1","artifact_version":"0.2.0","profile_name":"staging-a","verification_status":"passed","verified_at_unix_ms":1735689600000,"evidence":"local verify","evidence_sources":[{"artifact_path":"artifacts/eval/mnist_eval_summary.json","event_name":"eval_done","observed_at_unix_ms":1735689601000},{"artifact_path":"artifacts/deploy/candidate_upload_request.json","event_name":"artifact_upload_request_written","observed_at_unix_ms":1735689602000}]}"#,
    )
    .expect("write deployment verification receipt");

    let out = tmp
        .path()
        .join("deployment_verification_evidence_bundle.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("verification-bundle")
        .arg("--receipt")
        .arg(&receipt)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read deployment verification evidence bundle");
    let mut bundle: Value =
        serde_json::from_str(&text).expect("parse deployment verification evidence bundle");
    bundle
        .as_object_mut()
        .expect("bundle must be object")
        .insert("receipt_path".to_string(), Value::from("<receipt>"));
    assert_eq!(
        bundle,
        serde_json::json!({
            "schema_version": "1",
            "receipt_path": "<receipt>",
            "artifact_version": "0.2.0",
            "profile_name": "staging-a",
            "verification_status": "passed",
            "evidence_count": 2,
            "evidence_sources": [
                {
                    "artifact_path": "artifacts/eval/mnist_eval_summary.json",
                    "event_name": "eval_done",
                    "observed_at_unix_ms": 1735689601000i64
                },
                {
                    "artifact_path": "artifacts/deploy/candidate_upload_request.json",
                    "event_name": "artifact_upload_request_written",
                    "observed_at_unix_ms": 1735689602000i64
                }
            ]
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "deployment_verification_evidence_bundle_written");
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("bundle_path".to_string(), Value::from("<bundle>"));
    let bundle_obj = fields
        .get_mut("bundle")
        .and_then(Value::as_object_mut)
        .expect("event bundle must be an object");
    bundle_obj.insert("receipt_path".to_string(), Value::from("<receipt>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_verification_evidence_bundle_written",
            "fields": {
                "bundle_path": "<bundle>",
                "bundle": bundle
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
