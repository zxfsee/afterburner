use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn promotion_orchestration_contract_is_documented_and_wired() {
    let justfile = repo_file("justfile");
    for recipe in [
        "rollout-check candidate_artifact candidate_manifest ownership provider destination:",
        "rollout-promote artifact_version:",
        "rollout-verify candidate_artifact:",
        "rollout-rollback:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }
    assert!(
        justfile.contains("eval --artifact")
            && justfile.contains("upload --manifest")
            && justfile.contains("deploy-check")
            && justfile.contains("artifacts/deploy/previous_current_version.txt"),
        "rollout orchestration recipes must preserve explicit eval, upload, deploy, and rollback evidence"
    );
    assert!(
        justfile.contains("cargo run --locked --bin afterburner -- deploy promote-current"),
        "rollout-promote must delegate pointer mutation to the native deploy promote-current command"
    );
    assert!(
        justfile.contains("cargo run --locked --bin afterburner -- rollback current-pointer"),
        "rollout-rollback must delegate pointer mutation to the native rollback current-pointer command"
    );
    assert!(
        !justfile.contains("open artifacts/inference/current | str trim | save --force artifacts/deploy/previous_current_version.txt")
            && !justfile.contains("\"{{ artifact_version }}\" | save --force artifacts/inference/current")
            && !justfile.contains("open artifacts/deploy/previous_current_version.txt | str trim | save --force artifacts/inference/current"),
        "rollout pointer mutation must not stay encoded as recipe-local file writes"
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-017"),
        "architecture decisions index must link ADR-017"
    );
    assert!(
        architecture.contains("rollout"),
        "architecture must mention the promotion and rollback orchestration contract"
    );

    let adr = repo_file("docs/adr/017-promotion-orchestration.md");
    assert!(
        adr.contains("rollout-check"),
        "ADR-017 must describe the rollout-check recipe"
    );
    assert!(
        adr.contains("rollout-rollback"),
        "ADR-017 must describe the rollback recipe"
    );

    let readme = repo_file("docs/workflows.md");
    for workflow in [
        "just rollout-check",
        "just rollout-promote",
        "just rollout-verify",
        "just rollout-rollback",
        "afterburner deploy promote-current",
        "afterburner rollback current-pointer",
    ] {
        assert!(
            readme.contains(workflow),
            "workflow reference must document `{workflow}`"
        );
    }
}
