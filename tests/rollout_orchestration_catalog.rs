use std::fs;

use serde_json::Value;

#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

fn recipe_block<'a>(justfile: &'a str, recipe: &str) -> &'a str {
    let anchor = format!("{recipe}:");
    justfile
        .split(&anchor)
        .nth(1)
        .and_then(|rest| rest.split("\n\n").next())
        .unwrap_or_else(|| panic!("missing recipe block `{recipe}`"))
}

#[test]
fn routing_and_orchestration_guard_is_exposed_via_one_workflow_surface_check() {
    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("workflow-surface-check-routing-orchestration:"),
        "justfile must expose the routing/orchestration workflow surface check"
    );
    assert!(
        justfile.contains("--test routing_orchestration_semantic_regression")
            && justfile.contains("--test cli_dispatch_decomposition"),
        "workflow surface check must run the semantic-thin and cli-dispatch decomposition gates"
    );
}

#[test]
fn routing_and_orchestration_surfaces_stay_semantic_thin() {
    let main_rs = repo_file("src/main.rs");
    let cli_dispatch = repo_file("src/cli_dispatch/mod.rs");
    let deploy_dispatch = repo_file("src/cli_dispatch/deploy.rs");
    let justfile = repo_file("justfile");

    for text in [&main_rs, &cli_dispatch, &deploy_dispatch] {
        for forbidden in [
            "serde_json",
            "json!(",
            "emit_json_artifact_written",
            "write_json_value",
            "load_json_object(",
            "read_string(",
            "read_u64(",
            "\"schema_version\"",
            "\"artifact_role\"",
            "\"evidence_sources\"",
        ] {
            assert!(
                !text.contains(forbidden),
                "routing surface must not own payload/protocol semantics `{forbidden}`"
            );
        }
    }

    for required in [
        "cli_dispatch::run(std::env::args().skip(1))",
        "deploy::run_deploy(args)",
        "deploy::run_verify(args)",
        "deploy::run_rollback(args)",
        "deploy::run_debug_deploy(args)",
    ] {
        assert!(
            main_rs.contains(required) || cli_dispatch.contains(required),
            "routing surface must keep grouped dispatch entry `{required}`"
        );
    }

    for forbidden in [
        "jq ",
        "python -c",
        "python3 -c",
        "\"schema_version\"",
        "\"artifact_role\"",
        "\"evidence_sources\"",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must stay orchestration-only and avoid embedded protocol logic `{forbidden}`"
        );
    }
}

#[test]
fn deploy_promote_current_and_rollback_current_pointer_update_files_and_records() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let current_pointer = tmp.path().join("artifacts/inference/current");
    let previous_version_file = tmp
        .path()
        .join("artifacts/deploy/previous_current_version.txt");
    let promotion_record = tmp
        .path()
        .join("artifacts/deploy/inference_current_pointer_promotion.json");
    let rollback_record = tmp
        .path()
        .join("artifacts/deploy/inference_current_pointer_rollback.json");

    fs::create_dir_all(
        current_pointer
            .parent()
            .expect("current pointer parent directory"),
    )
    .expect("create current pointer parent");
    fs::write(&current_pointer, "0.1.0\n").expect("write current pointer");

    let mut promote = assert_cmd::cargo::cargo_bin_cmd!("afterburner");
    promote
        .current_dir(tmp.path())
        .arg("deploy")
        .arg("promote-current")
        .arg("--artifact-version")
        .arg("0.2.0")
        .arg("--current-pointer")
        .arg(&current_pointer)
        .arg("--previous-version-file")
        .arg(&previous_version_file)
        .arg("--out-record")
        .arg(&promotion_record)
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(&current_pointer).expect("read promoted pointer"),
        "0.2.0\n"
    );
    assert_eq!(
        fs::read_to_string(&previous_version_file).expect("read saved previous version"),
        "0.1.0\n"
    );

    let promotion: Value = serde_json::from_str(
        &fs::read_to_string(&promotion_record).expect("read promotion record"),
    )
    .expect("parse promotion record");
    assert_eq!(promotion["previous_version"], "0.1.0");
    assert_eq!(promotion["promoted_version"], "0.2.0");
    assert_eq!(
        promotion["current_pointer_path"],
        Value::from(current_pointer.display().to_string())
    );

    let mut rollback = assert_cmd::cargo::cargo_bin_cmd!("afterburner");
    rollback
        .current_dir(tmp.path())
        .arg("rollback")
        .arg("current-pointer")
        .arg("--current-pointer")
        .arg(&current_pointer)
        .arg("--previous-version-file")
        .arg(&previous_version_file)
        .arg("--out-record")
        .arg(&rollback_record)
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(&current_pointer).expect("read rolled back pointer"),
        "0.1.0\n"
    );

    let rollback: Value =
        serde_json::from_str(&fs::read_to_string(&rollback_record).expect("read rollback record"))
            .expect("parse rollback record");
    assert_eq!(rollback["previous_version"], "0.2.0");
    assert_eq!(rollback["restored_version"], "0.1.0");
    assert_eq!(
        rollback["previous_version_path"],
        Value::from(previous_version_file.display().to_string())
    );
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
