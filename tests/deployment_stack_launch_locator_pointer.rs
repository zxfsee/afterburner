use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn deployment_stack_launch_locator_pointer_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_stack_launch_locator_pointer.schema.json",
    ))
    .expect("read deployment stack launch locator pointer schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment stack launch locator pointer schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/deployment-stack-launch-locator-pointer/v1")
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
        "profile_name",
        "deploy_hostname",
        "rollout_entrypoint",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("deploy-launch-locator locator:"),
        "justfile must expose the deploy-launch-locator workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("deployment_stack_launch_locator_pointer.json"),
        "workflow reference must mention the deployment stack launch locator pointer artifact"
    );
    assert!(
        workflows.contains("just deploy-launch-locator"),
        "workflow reference must mention the deployment stack launch locator pointer workflow"
    );
}

#[test]
fn deployment_stack_launch_locator_pointer_writes_pointer_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let locator = tmp
        .path()
        .join("deployment_stack_launch_transport_locator.json");
    fs::write(
        &locator,
        r#"{
  "schema_version": "1",
  "handoff_path": "artifacts/deploy/deployment_stack_launch_evidence_handoff.json",
  "profile_name": "staging-a",
  "deploy_hostname": "staging.example.net",
  "rollout_entrypoint": "/srv/afterburner/artifacts/inference/current"
}"#,
    )
    .expect("write deployment stack launch transport locator");

    let out = tmp
        .path()
        .join("deployment_stack_launch_locator_pointer.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("point-launch-locator")
        .arg("--locator")
        .arg(&locator)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let pointer_text = fs::read_to_string(&out).expect("read locator pointer");
    let mut pointer: Value = serde_json::from_str(&pointer_text).expect("parse locator pointer");
    pointer
        .as_object_mut()
        .expect("pointer must be a top-level object")
        .insert("locator_path".to_string(), Value::from("<locator>"));
    let expected_text = fs::read_to_string(fixture_path(
        "deployment_stack_launch_locator_pointer.example.json",
    ))
    .expect("read deployment stack launch locator pointer example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse deployment stack launch locator pointer example");
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
                == Some("deployment_stack_launch_locator_pointer_written")
        })
        .unwrap_or_else(|| {
            panic!("missing deployment stack launch locator pointer event in stderr: {stderr}")
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
            "event": "deployment_stack_launch_locator_pointer_written",
            "fields": {
                "pointer_path": "<pointer>",
                "pointer": expected
            }
        })
    );
}
