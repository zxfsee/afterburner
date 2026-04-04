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
fn deployment_stack_contract_is_explicit() {
    let schema_text = fs::read_to_string(fixture_path("deployment_stack_profile.schema.json"))
        .expect("read deployment stack profile schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse deployment stack profile schema");
    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/deployment-stack-profile/v1")
    );

    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("stack schema required must be array")
        .iter()
        .map(|value| value.as_str().expect("required entries must be strings"))
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "deployable_unit",
        "service_entrypoint",
        "service_surface",
        "artifact_roots",
        "rollout_entrypoint",
        "observability",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-036"),
        "architecture decisions index must link ADR-036"
    );
    assert!(
        architecture.contains("deployable unit is the Afterburner runtime stack"),
        "architecture must describe the runtime stack deployable unit"
    );

    let adr = repo_file("docs/adr/036-deployment-stack-contract.md");
    for needle in [
        "deployment_stack_profile.example.json",
        "service_entrypoint",
        "service_surface",
        "artifact_roots",
        "rollout_entrypoint",
        "observability",
        "deploy stack-check",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-036 must mention `{needle}` as part of the deployment stack contract"
        );
    }

    let reference = repo_file("docs/reference.md");
    let workflows = repo_file("docs/workflows.md");
    assert!(
        reference.contains("deployment_stack_profile.example.json"),
        "workflow reference must mention the deployment stack profile contract"
    );
    assert!(
        reference.contains("deployment_stack_check.json"),
        "workflow reference must mention the checked deployment stack artifact"
    );
}

#[test]
fn deployment_stack_check_writes_expected_artifact() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out_path = tmp.path().join("deployment_stack_check.json");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("stack-check")
        .arg("--target-profile")
        .arg(fixture_path("deployment_target_profile.example.json"))
        .arg("--stack-profile")
        .arg(fixture_path("deployment_stack_profile.example.json"))
        .env(
            "AFTERBURNER_TRACEPARENT",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
        )
        .arg("--out")
        .arg(&out_path);
    cmd.assert().success().code(0);

    let output = fs::read_to_string(&out_path).expect("read deployment stack check output");
    let actual: Value = serde_json::from_str(&output).expect("parse deployment stack check output");

    let expected_text = fs::read_to_string(fixture_path("deployment_stack_check.example.json"))
        .expect("read deployment stack check example");
    let expected: Value =
        serde_json::from_str(&expected_text).expect("parse deployment stack check example");
    assert_eq!(
        actual, expected,
        "deployment stack check artifact must match the documented contract"
    );

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("deploy stack-check --target-profile fixtures/deployment_target_profile.example.json --stack-profile fixtures/deployment_stack_profile.example.json"),
        "deploy-check must validate the deployment stack contract"
    );
}
