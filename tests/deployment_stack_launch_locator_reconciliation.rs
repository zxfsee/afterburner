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
fn deployment_stack_launch_locator_reconciliation_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path(
        "deployment_stack_launch_locator_reconciliation.schema.json",
    ))
    .expect("read deployment stack launch locator reconciliation schema");
    let schema: Value = serde_json::from_str(&schema_text)
        .expect("parse deployment stack launch locator reconciliation schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/deployment-stack-launch-locator-reconciliation/v1")
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
        "pointer_path",
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
        justfile.contains("deploy-launch-locator-reconcile plan pointer:"),
        "justfile must expose the deploy-launch-locator-reconcile workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("deployment_stack_launch_locator_reconciliation.json"),
        "workflow reference must mention the deployment stack launch locator reconciliation artifact"
    );
    assert!(
        workflows.contains("just deploy-launch-locator-reconcile"),
        "workflow reference must mention the deployment stack launch locator reconciliation workflow"
    );
}

#[test]
fn deployment_stack_launch_locator_reconciliation_writes_artifact_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("deployment_stack_launch_plan.json");
    fs::write(
        &plan,
        r#"{
  "schema_version": "1",
  "profile_name": "staging-a",
  "deploy_hostname": "staging.example.net",
  "rollout_entrypoint": "/srv/afterburner/artifacts/inference/current"
}"#,
    )
    .expect("write deployment stack launch plan");
    let pointer = tmp
        .path()
        .join("deployment_stack_launch_locator_pointer.json");
    fs::write(
        &pointer,
        r#"{
  "schema_version": "1",
  "locator_path": "artifacts/deploy/deployment_stack_launch_transport_locator.json",
  "profile_name": "staging-a",
  "deploy_hostname": "staging.example.net",
  "rollout_entrypoint": "/srv/afterburner/artifacts/inference/current"
}"#,
    )
    .expect("write deployment stack launch locator pointer");
    let out = tmp
        .path()
        .join("deployment_stack_launch_locator_reconciliation.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("reconcile-launch-locator")
        .arg("--plan")
        .arg(&plan)
        .arg("--pointer")
        .arg(&pointer)
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read reconciliation");
    let mut reconciliation: Value = serde_json::from_str(&text).expect("parse reconciliation");
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("plan_path".to_string(), Value::from("<plan>"));
    reconciliation
        .as_object_mut()
        .expect("reconciliation must be object")
        .insert("pointer_path".to_string(), Value::from("<pointer>"));

    let expected_text = fs::read_to_string(fixture_path(
        "deployment_stack_launch_locator_reconciliation.example.json",
    ))
    .expect("read deployment stack launch locator reconciliation example");
    let mut expected: Value = serde_json::from_str(&expected_text)
        .expect("parse deployment stack launch locator reconciliation example");
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("plan_path".to_string(), Value::from("<plan>"));
    expected
        .as_object_mut()
        .expect("expected reconciliation must be object")
        .insert("pointer_path".to_string(), Value::from("<pointer>"));
    assert_eq!(reconciliation, expected);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("deployment_stack_launch_locator_reconciliation_written")
        })
        .unwrap_or_else(|| {
            panic!(
                "missing deployment stack launch locator reconciliation event in stderr: {stderr}"
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
    let recon_obj = fields
        .get_mut("reconciliation")
        .and_then(Value::as_object_mut)
        .expect("event reconciliation must be an object");
    recon_obj.insert("plan_path".to_string(), Value::from("<plan>"));
    recon_obj.insert("pointer_path".to_string(), Value::from("<pointer>"));
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "deploy_cli",
            "event": "deployment_stack_launch_locator_reconciliation_written",
            "fields": {
                "reconciliation_path": "<reconciliation>",
                "reconciliation": expected
            }
        })
    );
}
