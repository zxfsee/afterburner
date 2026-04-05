use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

fn recipe_block<'a>(justfile: &'a str, recipe: &str) -> &'a str {
    let anchor = format!("{recipe}:");
    justfile
        .split(&anchor)
        .nth(1)
        .and_then(|rest| rest.split("\n\n").next())
        .unwrap_or_else(|| panic!("missing recipe block `{recipe}`"))
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
    let rollout_check = recipe_block(
        &justfile,
        "rollout-check candidate_artifact candidate_manifest ownership provider destination",
    );
    let rollout_verify = recipe_block(&justfile, "rollout-verify candidate_artifact");
    let rollout_promote = recipe_block(&justfile, "rollout-promote artifact_version");
    let rollout_rollback = recipe_block(&justfile, "rollout-rollback");
    assert!(
        justfile.contains("artifacts/deploy/previous_current_version.txt"),
        "rollout orchestration recipes must preserve explicit eval, upload, deploy, and rollback evidence"
    );
    assert!(
        rollout_check.contains("cargo run --locked --bin afterburner -- deploy rollout-check"),
        "rollout-check must delegate orchestration to the native deploy rollout-check command"
    );
    assert!(
        !rollout_check.contains("eval --artifact")
            && !rollout_check.contains("upload --manifest")
            && !rollout_check.contains("just deploy-check"),
        "rollout-check must not keep shell-chained eval, upload, or just re-entry"
    );
    assert!(
        rollout_verify.contains("cargo run --locked --bin afterburner -- verify rollout"),
        "rollout-verify must delegate orchestration to the native verify rollout command"
    );
    assert!(
        !rollout_verify.contains("cargo run --locked --bin afterburner -- infer")
            && !rollout_verify.contains("cargo run --locked --bin afterburner -- eval"),
        "rollout-verify must not keep shell-chained infer/eval steps"
    );
    assert!(
        rollout_promote.contains("cargo run --locked --bin afterburner -- deploy promote-current"),
        "rollout-promote must delegate pointer mutation to the native deploy promote-current command"
    );
    assert!(
        rollout_rollback
            .contains("cargo run --locked --bin afterburner -- rollback current-pointer"),
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

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Rollout verification stays grouped under these operator flows:",
        "Candidate check: `just rollout-check` delegates to `afterburner deploy rollout-check`.",
        "Promotion: `just rollout-promote` delegates to `afterburner deploy promote-current`.",
        "Post-promotion verification: `just rollout-verify` delegates to `afterburner verify rollout`.",
        "Rollback: `just rollout-rollback` delegates to `afterburner rollback current-pointer`.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped rollout surface `{needle}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Rollout verification and pointer state:",
        "`rollout_check.json`",
        "`rollout_verify.json`",
        "`inference_current_pointer_promotion.json`",
        "`inference_current_pointer_rollback.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the rollout surface `{needle}`"
        );
    }
}
