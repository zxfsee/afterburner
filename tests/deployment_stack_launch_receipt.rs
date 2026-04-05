use std::collections::BTreeSet;
use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn deployment_stack_launch_receipt_schema_and_workflow_are_explicit() {
    let schema_text =
        fs::read_to_string(fixture_path("deployment_stack_launch_receipt.schema.json"))
            .expect("read deployment stack launch receipt schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse deployment stack launch receipt schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/deployment-stack-launch-receipt/v1")
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
        "profile_name",
        "deploy_hostname",
        "deployable_unit",
        "service_entrypoint",
        "launch_env",
        "launched_at_unix_ms",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("deploy-launch-receipt plan port launched_at_unix_ms:"),
        "justfile must expose the deploy-launch-receipt workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("deployment_stack_launch_receipt.json"),
        "workflow reference must mention the deployment stack launch receipt artifact"
    );
    assert!(
        workflows.contains("just deploy-launch-receipt"),
        "workflow reference must mention the deployment stack launch receipt workflow"
    );
}

#[test]
fn deployment_stack_launch_receipt_writes_expected_artifact() {
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

    let out_path = tmp.path().join("deployment_stack_launch_receipt.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("stack-launch-receipt")
        .arg("--plan")
        .arg(&plan)
        .arg("--port")
        .arg("8080")
        .arg("--launched-at-unix-ms")
        .arg("1735689600000")
        .arg("--out")
        .arg(&out_path);
    cmd.assert().success().code(0);

    let output = fs::read_to_string(&out_path).expect("read deployment stack launch receipt");
    let mut actual: Value =
        serde_json::from_str(&output).expect("parse deployment stack launch receipt");
    actual
        .as_object_mut()
        .expect("launch receipt must be an object")
        .insert(
            "plan_path".to_string(),
            Value::from("artifacts/deploy/deployment_stack_launch_plan.json"),
        );
    let expected_text =
        fs::read_to_string(fixture_path("deployment_stack_launch_receipt.example.json"))
            .expect("read deployment stack launch receipt example");
    let expected: Value = serde_json::from_str(&expected_text)
        .expect("parse deployment stack launch receipt example");
    assert_eq!(
        actual, expected,
        "deployment stack launch receipt artifact must match the documented contract"
    );
}
