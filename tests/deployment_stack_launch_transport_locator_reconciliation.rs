use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn deployment_stack_launch_transport_locator_reconciliation_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_stack_launch_transport_locator_reconciliation.schema.json",
    ))
    .expect("read deployment stack launch transport locator reconciliation schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment stack launch transport locator reconciliation schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/deployment-stack-launch-transport-locator-reconciliation/v1"
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
        "handoff_path",
        "locator_path",
        "desired",
        "current",
        "reconciliation_status",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let reference = repo_file("docs/reference.md");
    assert!(
        reference.contains("deployment_stack_launch_transport_locator_reconciliation.json"),
        "workflow reference must mention the deployment stack launch transport locator reconciliation artifact"
    );
}

#[test]
fn deployment_stack_launch_transport_locator_reconciliation_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handoff = tmp
        .path()
        .join("deployment_stack_launch_evidence_handoff.json");
    fs::write(
        &handoff,
        r#"{
  "schema_version": "1",
  "bundle_path": "artifacts/deploy/deployment_stack_launch_evidence_bundle.json",
  "profile_name": "staging-a",
  "deploy_hostname": "staging.example.net",
  "deployable_unit": "afterburner-runtime-stack",
  "rollout_entrypoint": "/srv/afterburner/artifacts/inference/current",
  "launched_at_unix_ms": 1735689600000,
  "evidence_count": 2
}"#,
    )
    .expect("write handoff");
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
    .expect("write locator");
    let out = tmp
        .path()
        .join("deployment_stack_launch_transport_locator_reconciliation.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("reconcile-launch-transport-locator")
        .arg("--handoff")
        .arg(&handoff)
        .arg("--locator")
        .arg(&locator)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read reconciliation");
    let mut reconciliation: Value = serde_json::from_str(&text).expect("parse reconciliation");
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("handoff_path".to_string(), Value::from("<handoff>"));
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("locator_path".to_string(), Value::from("<locator>"));

    let expected_text = fs::read_to_string(fixture_path(
        "deployment_stack_launch_transport_locator_reconciliation.example.json",
    ))
    .expect("read deployment stack launch transport locator reconciliation example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse deployment stack launch transport locator reconciliation example");
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("handoff_path".to_string(), Value::from("<handoff>"));
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("locator_path".to_string(), Value::from("<locator>"));
    assert_eq!(reconciliation, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("deployment_stack_launch_transport_locator_reconciliation_written")
        })
        .unwrap_or_else(|| {
            panic!("missing transport locator reconciliation event in stderr: {stderr}")
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
    recon_obj.insert("handoff_path".to_string(), Value::from("<handoff>"));
    recon_obj.insert("locator_path".to_string(), Value::from("<locator>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_stack_launch_transport_locator_reconciliation_written",
            "fields": {
                "reconciliation_path": "<reconciliation>",
                "reconciliation": expected
            }
        })
    );
}
