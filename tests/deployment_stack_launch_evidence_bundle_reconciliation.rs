use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn deployment_stack_launch_evidence_bundle_reconciliation_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_stack_launch_evidence_bundle_reconciliation.schema.json",
    ))
    .expect("read deployment stack launch evidence bundle reconciliation schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment stack launch evidence bundle reconciliation schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(
            "https://afterburner.local/schemas/deployment-stack-launch-evidence-bundle-reconciliation/v1"
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

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("deploy-launch-bundle-reconcile receipt bundle:"),
        "justfile must expose the deploy-launch-bundle-reconcile workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("deployment_stack_launch_evidence_bundle_reconciliation.json"),
        "workflow reference must mention the deployment stack launch evidence bundle reconciliation artifact"
    );
    assert!(
        workflows.contains("just deploy-launch-bundle-reconcile"),
        "workflow reference must mention the deployment stack launch evidence bundle reconciliation workflow"
    );
}

#[test]
fn deployment_stack_launch_evidence_bundle_reconciliation_writes_artifact_and_event() {
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
    .expect("write plan");
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
    .expect("write receipt");
    let bundle = tmp
        .path()
        .join("deployment_stack_launch_evidence_bundle.json");
    fs::write(
        &bundle,
        format!(
            r#"{{
  "schema_version": "1",
  "plan_path": "{}",
  "receipt_path": "{}",
  "profile_name": "staging-a",
  "deploy_hostname": "staging.example.net",
  "deployable_unit": "afterburner-runtime-stack",
  "working_directory": "/srv/afterburner/artifacts",
  "service_entrypoint": "./bin/afterburner-http",
  "rollout_entrypoint": "/srv/afterburner/artifacts/inference/current",
  "launch_env": {{
    "PORT": "8080",
    "AFTERBURNER_OBS_JSONL_PATH": "/srv/afterburner/artifacts/train/observability.jsonl"
  }},
  "launched_at_unix_ms": 1735689600000,
  "evidence_count": 2,
  "evidence_sources": [
    {{"artifact_path":"{}","artifact_role":"deployment_stack_launch_plan"}},
    {{"artifact_path":"{}","artifact_role":"deployment_stack_launch_receipt"}}
  ]
}}"#,
            plan.display(),
            receipt.display(),
            plan.display(),
            receipt.display()
        ),
    )
    .expect("write bundle");
    let out = tmp
        .path()
        .join("deployment_stack_launch_evidence_bundle_reconciliation.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("reconcile-launch-bundle")
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
    let recon_obj = reconciliation.as_object_mut().expect("object");
    let desired = recon_obj
        .get_mut("desired")
        .and_then(Value::as_object_mut)
        .expect("desired object");
    desired.insert("plan_path".to_string(), Value::from("<plan>"));
    desired.insert("receipt_path".to_string(), Value::from("<receipt>"));
    let current = recon_obj
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("current object");
    current.insert("plan_path".to_string(), Value::from("<plan>"));
    current.insert("receipt_path".to_string(), Value::from("<receipt>"));

    let expected_text = fs::read_to_string(fixture_path(
        "deployment_stack_launch_evidence_bundle_reconciliation.example.json",
    ))
    .expect("read bundle reconciliation example");
    let mut expected: Value =
        serde_json::from_str(&expected_text).expect("parse bundle reconciliation example");
    let expected_obj = expected.as_object_mut().expect("expected object");
    expected_obj.insert("receipt_path".to_string(), Value::from("<receipt>"));
    expected_obj.insert("bundle_path".to_string(), Value::from("<bundle>"));
    let expected_desired = expected_obj
        .get_mut("desired")
        .and_then(Value::as_object_mut)
        .expect("expected desired");
    expected_desired.insert("plan_path".to_string(), Value::from("<plan>"));
    expected_desired.insert("receipt_path".to_string(), Value::from("<receipt>"));
    let expected_current = expected_obj
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("expected current");
    expected_current.insert("plan_path".to_string(), Value::from("<plan>"));
    expected_current.insert("receipt_path".to_string(), Value::from("<receipt>"));
    assert_eq!(reconciliation, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("deployment_stack_launch_evidence_bundle_reconciliation_written")
        })
        .unwrap_or_else(|| panic!("missing bundle reconciliation event in stderr: {stderr}"));
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
    let event_recon = fields
        .get_mut("reconciliation")
        .and_then(Value::as_object_mut)
        .expect("event reconciliation object");
    event_recon.insert("receipt_path".to_string(), Value::from("<receipt>"));
    event_recon.insert("bundle_path".to_string(), Value::from("<bundle>"));
    let event_desired = event_recon
        .get_mut("desired")
        .and_then(Value::as_object_mut)
        .expect("event desired");
    event_desired.insert("plan_path".to_string(), Value::from("<plan>"));
    event_desired.insert("receipt_path".to_string(), Value::from("<receipt>"));
    let event_current = event_recon
        .get_mut("current")
        .and_then(Value::as_object_mut)
        .expect("event current");
    event_current.insert("plan_path".to_string(), Value::from("<plan>"));
    event_current.insert("receipt_path".to_string(), Value::from("<receipt>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_stack_launch_evidence_bundle_reconciliation_written",
            "fields": {
                "reconciliation_path": "<reconciliation>",
                "reconciliation": expected
            }
        })
    );
}
