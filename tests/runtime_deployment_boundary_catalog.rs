use std::fs;
use std::path::PathBuf;

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

    let adr = repo_file("docs/adr/010-deployment-baseline-and-local-orchestration-substrate.md");
    assert!(
        adr.contains("deploy-rs"),
        "ADR-010 must describe the deploy-rs baseline"
    );
    assert!(
        adr.contains("activate.custom"),
        "ADR-010 must explain the custom activation choice"
    );
    assert!(
        adr.contains("process-compose-flake"),
        "ADR-010 must also capture the local orchestration substrate choice"
    );
}

#[test]
fn deployment_target_profile_schema_and_adr_are_explicit() {
    let schema_text = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/deployment_target_profile.schema.json"),
    )
    .expect("read deployment target profile schema fixture");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse schema fixture");
    let schema = schema
        .as_object()
        .expect("deployment target profile schema must be a json object");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/deployment-target-profile/v1")
    );
    assert_eq!(schema.get("type").and_then(Value::as_str), Some("object"));

    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be an array");
    for key in [
        "schema_version",
        "profile_name",
        "deploy_hostname",
        "ssh_user",
        "system",
        "artifact_root",
        "activation_strategy",
    ] {
        assert!(
            required.iter().any(|value| value.as_str() == Some(key)),
            "schema.required must include `{key}`"
        );
    }

    let properties = schema
        .get("properties")
        .and_then(Value::as_object)
        .expect("schema.properties must be an object");
    assert_eq!(
        properties
            .get("schema_version")
            .and_then(|value| value.get("const"))
            .and_then(Value::as_str),
        Some("1")
    );

    let sample_text = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/deployment_target_profile.example.json"),
    )
    .expect("read deployment target profile example fixture");
    let sample: Value = serde_json::from_str(&sample_text)
        .expect("parse deployment target profile example fixture");

    let sample = sample.as_object().expect("sample must be object");
    for key in [
        "schema_version",
        "profile_name",
        "deploy_hostname",
        "ssh_user",
        "system",
        "artifact_root",
        "activation_strategy",
    ] {
        assert!(
            sample.contains_key(key),
            "sample deployment target profile must include `{key}`"
        );
    }

    let architecture =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ARCHITECTURE.md"))
            .expect("read architecture");
    let adr = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/adr/008-deployment-target-profile.md"),
    )
    .expect("read deployment target profile ADR");

    assert!(
        architecture.contains("ADR-008"),
        "architecture decisions index must link ADR-008"
    );
    assert!(
        adr.contains("deployment target profile"),
        "ADR must describe the deployment target profile contract"
    );
    assert!(
        adr.contains("deploy-rs"),
        "ADR must explain the deploy-rs relationship"
    );
}

#[test]
fn distributed_world_rank_topology_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-039"),
        "architecture decisions index must link ADR-039"
    );

    let adr = repo_file("docs/adr/039-distributed-world-rank-topology.md");
    for needle in [
        "`world_size`",
        "`global_rank`",
        "`local_rank`",
        "`node_rank`",
        "`device_group`",
        "`owner_worker_id`",
        "`owner_worker_rank`",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-039 must mention `{needle}` as part of the topology decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("`world_size`, ranks, and `device_group` metadata"),
        "reference index must document the explicit topology stance"
    );
}

#[test]
fn scheduler_preemption_and_colocation_stance_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-042"),
        "architecture decisions index must link ADR-042"
    );
    for needle in [
        "cooperative checkpoint/resume only",
        "transparent GPU",
        "GPU colocation",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the scheduler stance"
        );
    }

    let adr = repo_file("docs/adr/042-scheduler-preemption-and-colocation.md");
    for needle in [
        "queueing plus priority",
        "cooperative checkpoint/resume",
        "transparent GPU",
        "UVM/MPS",
        "GPU colocation",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-042 must mention `{needle}` as part of the scheduler policy decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("Supported preemption is cooperative checkpoint/resume"),
        "reference index must document the scheduler preemption stance"
    );
}

#[test]
fn async_runtime_decision_record_is_linked_and_explicit() {
    let architecture = repo_file("ARCHITECTURE.md");
    let adr = repo_file("docs/adr/007-async-runtime-decision.md");
    let architecture_lower = architecture.to_ascii_lowercase();

    assert!(
        architecture.contains("ADR-007"),
        "architecture decisions index must link the async runtime ADR"
    );
    assert!(
        architecture_lower.contains("no async runtime is adopted"),
        "architecture must state the current runtime decision explicitly"
    );
    assert!(
        adr.contains("Tokio"),
        "ADR must discuss the Tokio decision explicitly"
    );
    assert!(
        adr.contains("tiny_http"),
        "ADR must record the current HTTP adapter baseline"
    );
    for needle in ["concurrency", "deployment", "trace"] {
        assert!(
            adr.to_ascii_lowercase().contains(needle),
            "ADR must mention `{needle}` as part of the Tokio fit criteria"
        );
    }
    assert!(
        adr.contains("profil"),
        "ADR must record what evidence would justify revisiting the decision"
    );
}

#[test]
fn async_runtime_fit_stays_deferred_across_reference_surfaces() {
    let readme = repo_file("README.md");
    assert!(
        readme.contains("synchronous `tiny_http`"),
        "README must keep the HTTP adapter baseline explicit"
    );
    assert!(
        readme.contains("Tokio"),
        "README must mention the deferred Tokio fit trigger"
    );

    let cargo_toml = repo_file("Cargo.toml");
    assert!(
        !cargo_toml.contains("tokio ="),
        "Cargo.toml must not add Tokio before the adapter runtime gate is intentionally revisited"
    );
}
