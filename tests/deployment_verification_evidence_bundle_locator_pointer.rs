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
fn deployment_verification_bundle_locator_pointer_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_verification_evidence_bundle_locator_pointer.schema.json",
    ))
    .expect("read deployment verification bundle locator pointer schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment verification bundle locator pointer schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/deployment-verification-evidence-bundle-locator-pointer/v1"
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
        "bundle_path",
        "artifact_version",
        "profile_name",
        "verification_status",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("deployment-verification-point-bundle-locator locator:"),
        "justfile must expose the deployment-verification-point-bundle-locator workflow"
    );

    let workflows = repo_file("docs/workflows.md");
    assert!(
        workflows.contains("deployment_verification_evidence_bundle_locator_pointer.json"),
        "workflow reference must mention the deployment verification bundle locator pointer artifact"
    );
    assert!(
        workflows.contains("just deployment-verification-point-bundle-locator"),
        "workflow reference must mention the deployment verification bundle locator pointer workflow"
    );
}

#[test]
fn deployment_verification_bundle_locator_pointer_writes_pointer_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let locator = tmp
        .path()
        .join("deployment_verification_evidence_bundle_transport_locator.json");
    fs::write(
        &locator,
        r#"{
  "schema_version": "1",
  "bundle_path": "artifacts/deploy/deployment_verification_evidence_bundle.json",
  "artifact_version": "0.2.0",
  "profile_name": "staging-a",
  "verification_status": "passed"
}"#,
    )
    .expect("write deployment verification evidence bundle transport locator");

    let out = tmp
        .path()
        .join("deployment_verification_evidence_bundle_locator_pointer.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("point-verification-bundle-locator")
        .arg("--locator")
        .arg(&locator)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let pointer_text = fs::read_to_string(&out).expect("read bundle locator pointer");
    let mut pointer: Value =
        serde_json::from_str(&pointer_text).expect("parse bundle locator pointer");
    pointer
        .as_object_mut()
        .expect("pointer must be a top-level object")
        .insert("locator_path".to_string(), Value::from("<locator>"));
    let expected_text = fs::read_to_string(fixture_path(
        "deployment_verification_evidence_bundle_locator_pointer.example.json",
    ))
    .expect("read deployment verification bundle locator pointer example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse deployment verification bundle locator pointer example");
    expected
        .as_object_mut()
        .expect("expected pointer must be a top-level object")
        .insert("locator_path".to_string(), Value::from("<locator>"));
    assert_eq!(pointer, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("deployment_verification_evidence_bundle_locator_pointer_written")
        })
        .unwrap_or_else(|| {
            panic!(
                "missing deployment verification bundle locator pointer event in stderr: {stderr}"
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
    fields.insert("pointer_path".to_string(), Value::from("<pointer>"));
    let pointer_obj = fields
        .get_mut("pointer")
        .and_then(Value::as_object_mut)
        .expect("event pointer must be an object");
    pointer_obj.insert("locator_path".to_string(), Value::from("<locator>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_verification_evidence_bundle_locator_pointer_written",
            "fields": {
                "pointer_path": "<pointer>",
                "pointer": expected
            }
        })
    );
}
