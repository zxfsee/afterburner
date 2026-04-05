use serde_json::Value;

#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn deploy_rs_baseline_consumes_profile_contract() {
    let profile_text = repo_file("fixtures/deployment_target_profile.example.json");
    let profile: Value =
        serde_json::from_str(&profile_text).expect("parse deployment target profile example");
    let profile = profile
        .as_object()
        .expect("deployment target profile example must be an object");

    let profile_name = profile
        .get("profile_name")
        .and_then(Value::as_str)
        .expect("profile_name must be present");
    let deploy_hostname = profile
        .get("deploy_hostname")
        .and_then(Value::as_str)
        .expect("deploy_hostname must be present");
    let ssh_user = profile
        .get("ssh_user")
        .and_then(Value::as_str)
        .expect("ssh_user must be present");
    let artifact_root = profile
        .get("artifact_root")
        .and_then(Value::as_str)
        .expect("artifact_root must be present");

    let flake = repo_file("flake.nix");
    assert!(
        flake.contains("deploy-rs"),
        "flake must add the deploy-rs input"
    );
    assert!(
        flake.contains("deployment_target_profile.example.json"),
        "flake must consume the deployment target profile example fixture"
    );
    assert!(
        flake.contains("deployChecks self.deploy"),
        "flake checks must validate deploy-rs definitions"
    );
    assert!(
        flake.contains(profile_name) || flake.contains("deploymentProfile.profile_name"),
        "flake deploy output must reference the profile name from the contract fixture"
    );
    assert!(
        flake.contains(deploy_hostname) || flake.contains("deploymentProfile.deploy_hostname"),
        "flake deploy output must reference the profile hostname from the contract fixture"
    );
    assert!(
        flake.contains(ssh_user) || flake.contains("deploymentProfile.ssh_user"),
        "flake deploy output must reference the profile ssh_user from the contract fixture"
    );
    assert!(
        flake.contains(artifact_root) || flake.contains("deploymentProfile.artifact_root"),
        "flake deploy output must reference the profile artifact_root from the contract fixture"
    );
    assert!(
        flake.contains("activate.custom"),
        "baseline deploy-rs integration must use a custom activation path for the app package"
    );

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("deploy-check:"),
        "justfile must expose a deploy-check workflow"
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-010"),
        "architecture decisions index must link ADR-010"
    );
    assert!(
        architecture.contains("deploy-rs"),
        "architecture must document the deploy-rs baseline"
    );

    let adr = repo_file("docs/adr/010-deploy-rs-baseline.md");
    assert!(
        adr.contains("deploy-rs"),
        "ADR-010 must describe the deploy-rs baseline"
    );
    assert!(
        adr.contains("activate.custom"),
        "ADR-010 must explain the custom activation choice"
    );
}
