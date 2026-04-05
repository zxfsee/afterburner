use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn deployment_stack_launch_evidence_bundle_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_stack_launch_evidence_bundle.schema.json",
    ))
    .expect("read deployment stack launch evidence bundle schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment stack launch evidence bundle schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/deployment-stack-launch-evidence-bundle/v1")
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
        "plan_path",
        "receipt_path",
        "profile_name",
        "deploy_hostname",
        "deployable_unit",
        "working_directory",
        "service_entrypoint",
        "rollout_entrypoint",
        "launch_env",
        "launched_at_unix_ms",
        "evidence_count",
        "evidence_sources",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("deploy-launch-bundle receipt:"),
        "justfile must expose the deploy-launch-bundle workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("deployment_stack_launch_evidence_bundle.json"),
        "workflow reference must mention the deployment stack launch evidence bundle artifact"
    );
    assert!(
        workflows.contains("just deploy-launch-bundle"),
        "workflow reference must mention the deployment stack launch evidence bundle workflow"
    );

    let adr = repo_file("docs/adr/036-deployment-stack-contract.md");
    assert!(
        adr.contains("deployment_stack_launch_evidence_bundle.json"),
        "ADR-036 must mention the stack launch evidence bundle artifact"
    );
}

#[test]
fn deployment_stack_launch_bundle_writes_bundle_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("deployment_stack_launch_plan.json");
    fs::write(
        &plan,
        r#"{
  "schema_version": "1",
  "profile_name": "staging-a",
  "deploy_hostname": "staging.example.net",
  "ssh_user": "deploy",
  "system": "aarch64-darwin",
  "deployable_unit": "afterburner-runtime-stack",
  "activation_strategy": "current-pointer",
  "working_directory": "/srv/afterburner/artifacts",
  "service_entrypoint": "./bin/afterburner-http",
  "rollout_entrypoint": "/srv/afterburner/artifacts/inference/current",
  "required_env": ["PORT", "AFTERBURNER_OBS_JSONL_PATH"],
  "default_env": {
    "AFTERBURNER_OBS_JSONL_PATH": "/srv/afterburner/artifacts/train/observability.jsonl"
  }
}"#,
    )
    .expect("write deployment stack launch plan");

    let receipt = tmp.path().join("deployment_stack_launch_receipt.json");
    fs::write(
        &receipt,
        format!(
            r#"{{
  "schema_version": "1",
  "plan_path": "{}",
  "profile_name": "staging-a",
  "deploy_hostname": "staging.example.net",
  "deployable_unit": "afterburner-runtime-stack",
  "service_entrypoint": "./bin/afterburner-http",
  "launch_env": {{
    "PORT": "8080",
    "AFTERBURNER_OBS_JSONL_PATH": "/srv/afterburner/artifacts/train/observability.jsonl"
  }},
  "launched_at_unix_ms": 1735689600000
}}"#,
            plan.display()
        ),
    )
    .expect("write deployment stack launch receipt");

    let out = tmp
        .path()
        .join("deployment_stack_launch_evidence_bundle.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("stack-launch-bundle")
        .arg("--receipt")
        .arg(&receipt)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read deployment stack launch evidence bundle");
    let mut bundle: Value =
        serde_json::from_str(&text).expect("parse deployment stack launch evidence bundle");
    let object = bundle.as_object_mut().expect("bundle must be an object");
    object.insert("plan_path".to_string(), Value::from("<plan>"));
    object.insert("receipt_path".to_string(), Value::from("<receipt>"));
    let evidence_sources = object
        .get_mut("evidence_sources")
        .and_then(Value::as_array_mut)
        .expect("evidence sources must be an array");
    evidence_sources[0]
        .as_object_mut()
        .expect("evidence source must be object")
        .insert("artifact_path".to_string(), Value::from("<plan>"));
    evidence_sources[1]
        .as_object_mut()
        .expect("evidence source must be object")
        .insert("artifact_path".to_string(), Value::from("<receipt>"));

    let expected_text = fs::read_to_string(fixture_path(
        "deployment_stack_launch_evidence_bundle.example.json",
    ))
    .expect("read deployment stack launch evidence bundle example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse deployment stack launch evidence bundle example");
    let expected_object = expected
        .as_object_mut()
        .expect("expected bundle must be an object");
    expected_object.insert("plan_path".to_string(), Value::from("<plan>"));
    expected_object.insert("receipt_path".to_string(), Value::from("<receipt>"));
    let expected_sources = expected_object
        .get_mut("evidence_sources")
        .and_then(Value::as_array_mut)
        .expect("expected evidence sources must be an array");
    expected_sources[0]
        .as_object_mut()
        .expect("expected evidence source must be object")
        .insert("artifact_path".to_string(), Value::from("<plan>"));
    expected_sources[1]
        .as_object_mut()
        .expect("expected evidence source must be object")
        .insert("artifact_path".to_string(), Value::from("<receipt>"));
    assert_eq!(bundle, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr_event(&stderr, "deployment_stack_launch_evidence_bundle_written");
    let mut normalized_event = event.clone();
    let event_object = normalized_event
        .as_object_mut()
        .expect("event must be an object");
    event_object.insert("ts_ms".to_string(), Value::from(0));
    let fields = event_object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert("bundle_path".to_string(), Value::from("<bundle>"));
    let event_bundle = fields
        .get_mut("bundle")
        .and_then(Value::as_object_mut)
        .expect("event bundle must be an object");
    event_bundle.insert("plan_path".to_string(), Value::from("<plan>"));
    event_bundle.insert("receipt_path".to_string(), Value::from("<receipt>"));
    let event_sources = event_bundle
        .get_mut("evidence_sources")
        .and_then(Value::as_array_mut)
        .expect("event evidence sources must be an array");
    event_sources[0]
        .as_object_mut()
        .expect("event evidence source must be object")
        .insert("artifact_path".to_string(), Value::from("<plan>"));
    event_sources[1]
        .as_object_mut()
        .expect("event evidence source must be object")
        .insert("artifact_path".to_string(), Value::from("<receipt>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_stack_launch_evidence_bundle_written",
            "fields": {
                "bundle_path": "<bundle>",
                "bundle": expected
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
