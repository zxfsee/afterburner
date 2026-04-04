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
fn deployment_stack_launch_plan_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path("deployment_stack_launch_plan.schema.json"))
        .expect("read deployment stack launch plan schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse deployment stack launch plan schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/deployment-stack-launch-plan/v1")
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
        "profile_name",
        "deploy_hostname",
        "ssh_user",
        "system",
        "deployable_unit",
        "activation_strategy",
        "working_directory",
        "service_entrypoint",
        "rollout_entrypoint",
        "required_env",
        "default_env",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("deploy-launch-plan target_profile stack_profile:"),
        "justfile must expose the deploy-launch-plan workflow"
    );

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("deployment_stack_launch_plan.json"),
        "workflow reference must mention the deployment stack launch plan artifact"
    );
    assert!(
        workflows.contains("just deploy-launch-plan"),
        "workflow reference must mention the deployment stack launch plan workflow"
    );
}

#[test]
fn deployment_stack_launch_plan_writes_expected_artifact() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out_path = tmp.path().join("deployment_stack_launch_plan.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("stack-launch-plan")
        .arg("--target-profile")
        .arg(fixture_path("deployment_target_profile.example.json"))
        .arg("--stack-profile")
        .arg(fixture_path("deployment_stack_profile.example.json"))
        .arg("--out")
        .arg(&out_path);
    cmd.assert().success().code(0);

    let output = fs::read_to_string(&out_path).expect("read deployment stack launch plan");
    let actual: Value = serde_json::from_str(&output).expect("parse deployment stack launch plan");
    let expected_text =
        fs::read_to_string(fixture_path("deployment_stack_launch_plan.example.json"))
            .expect("read deployment stack launch plan example");
    let expected: Value =
        serde_json::from_str(&expected_text).expect("parse deployment stack launch plan example");
    assert_eq!(
        actual, expected,
        "deployment stack launch plan artifact must match the documented contract"
    );
}
