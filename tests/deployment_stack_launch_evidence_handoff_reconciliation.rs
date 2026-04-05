use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn deployment_stack_launch_handoff_reconciliation_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_stack_launch_evidence_handoff_reconciliation.schema.json",
    ))
    .expect("read deployment stack launch handoff reconciliation schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment stack launch handoff reconciliation schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/deployment-stack-launch-evidence-handoff-reconciliation/v1"
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
        "bundle_path",
        "handoff_path",
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
        justfile.contains("deploy-launch-handoff-reconcile bundle handoff:"),
        "justfile must expose the deploy-launch-handoff-reconcile workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("deployment_stack_launch_evidence_handoff_reconciliation.json"),
        "workflow reference must mention the deployment stack launch handoff reconciliation artifact"
    );
    assert!(
        workflows.contains("just deploy-launch-handoff-reconcile"),
        "workflow reference must mention the deployment stack launch handoff reconciliation workflow"
    );
}

#[test]
fn deployment_stack_launch_handoff_reconciliation_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bundle = tmp
        .path()
        .join("deployment_stack_launch_evidence_bundle.json");
    fs::write(
        &bundle,
        r#"{
  "schema_version": "1",
  "plan_path": "artifacts/deploy/deployment_stack_launch_plan.json",
  "receipt_path": "artifacts/deploy/deployment_stack_launch_receipt.json",
  "profile_name": "staging-a",
  "deploy_hostname": "staging.example.net",
  "deployable_unit": "afterburner-runtime-stack",
  "working_directory": "/srv/afterburner/artifacts",
  "service_entrypoint": "./bin/afterburner-http",
  "rollout_entrypoint": "/srv/afterburner/artifacts/inference/current",
  "launch_env": {
    "PORT": "8080",
    "AFTERBURNER_OBS_JSONL_PATH": "/srv/afterburner/artifacts/train/observability.jsonl"
  },
  "launched_at_unix_ms": 1735689600000,
  "evidence_count": 2,
  "evidence_sources": [
    {"artifact_path":"artifacts/deploy/deployment_stack_launch_plan.json","artifact_role":"deployment_stack_launch_plan"},
    {"artifact_path":"artifacts/deploy/deployment_stack_launch_receipt.json","artifact_role":"deployment_stack_launch_receipt"}
  ]
}"#,
    )
    .expect("write bundle");
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
    let out = tmp
        .path()
        .join("deployment_stack_launch_evidence_handoff_reconciliation.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("reconcile-launch-handoff")
        .arg("--bundle")
        .arg(&bundle)
        .arg("--handoff")
        .arg(&handoff)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read reconciliation");
    let mut reconciliation: Value = serde_json::from_str(&text).expect("parse reconciliation");
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("bundle_path".to_string(), Value::from("<bundle>"));
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("handoff_path".to_string(), Value::from("<handoff>"));

    let expected_text = fs::read_to_string(fixture_path(
        "deployment_stack_launch_evidence_handoff_reconciliation.example.json",
    ))
    .expect("read deployment stack launch handoff reconciliation example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse deployment stack launch handoff reconciliation example");
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("bundle_path".to_string(), Value::from("<bundle>"));
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("handoff_path".to_string(), Value::from("<handoff>"));
    assert_eq!(reconciliation, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("deployment_stack_launch_evidence_handoff_reconciliation_written")
        })
        .unwrap_or_else(|| panic!("missing handoff reconciliation event in stderr: {stderr}"));
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
    recon_obj.insert("bundle_path".to_string(), Value::from("<bundle>"));
    recon_obj.insert("handoff_path".to_string(), Value::from("<handoff>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_stack_launch_evidence_handoff_reconciliation_written",
            "fields": {
                "reconciliation_path": "<reconciliation>",
                "reconciliation": expected
            }
        })
    );
}
