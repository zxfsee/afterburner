use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::Path;
use std::process::Command;

fn run_jj(root: &Path, args: &[&str]) {
    let status = Command::new("jj")
        .current_dir(root)
        .args(args)
        .status()
        .unwrap_or_else(|err| panic!("jj {}: {err}", args.join(" ")));
    assert!(status.success(), "jj {} must succeed", args.join(" "));
}

fn init_jj_repo(root: &Path) {
    run_jj(
        root,
        &[
            "--config",
            "user.name=Afterburner Tests",
            "--config",
            "user.email=afterburner-tests@example.com",
            "git",
            "init",
            ".",
        ],
    );
    run_jj(
        root,
        &[
            "--config",
            "user.name=Afterburner Tests",
            "--config",
            "user.email=afterburner-tests@example.com",
            "commit",
            "-m",
            "init",
        ],
    );
}

#[test]
fn queue_snapshot_stamp_and_verify_round_trip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cargo_toml = tmp.path().join("Cargo.toml");
    let changelog = tmp.path().join("CHANGELOG.md");

    fs::write(
        &cargo_toml,
        "[package]\nname = \"x\"\n[package.metadata.git-cliff.changelog]\nheader = \"# Changelog\\n\\n## TODO\\n\\n- a\\n  - Goal: x\\n  - Kind: `mixed`\\n  - Boundary: `none`\\n  - Contracts: `none`\\n  - Scope: `src/`\\n\\n## [Trunk]\\n\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(&changelog, "# Changelog\n\n## TODO\n\n- a\n\n## [Trunk]\n")
        .expect("write changelog");

    let mut stamp = cargo_bin_cmd!("workflow_queue_snapshot");
    stamp
        .arg("stamp")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--changelog")
        .arg(&changelog)
        .arg("--parent-commit")
        .arg("abc123");
    stamp.assert().success();

    let stamped = fs::read_to_string(&changelog).expect("read stamped changelog");
    assert!(
        stamped.contains("queue-snapshot:"),
        "snapshot comment must be written"
    );
    assert!(
        stamped.contains("\n\n<!-- queue-snapshot:"),
        "snapshot comment must be separated from the TODO list by a blank line"
    );

    let mut verify = cargo_bin_cmd!("workflow_queue_snapshot");
    verify
        .arg("verify")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--changelog")
        .arg(&changelog)
        .arg("--parent-commit")
        .arg("abc123");
    verify.assert().success();

    let mut mismatch = cargo_bin_cmd!("workflow_queue_snapshot");
    mismatch
        .arg("verify")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--changelog")
        .arg(&changelog)
        .arg("--parent-commit")
        .arg("def456");
    let assert = mismatch.assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains("queue snapshot lineage is stale")
            && stderr.contains("just queue-resume")
            && stderr.contains("just queue-refresh"),
        "queue snapshot parent mismatch must classify stale lineage and point at queue-resume plus queue-refresh: {stderr}"
    );

    let mut tolerate_previous_parent = cargo_bin_cmd!("workflow_queue_snapshot");
    tolerate_previous_parent
        .arg("verify")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--changelog")
        .arg(&changelog)
        .arg("--parent-commit")
        .arg("def456")
        .arg("--previous-parent-commit")
        .arg("abc123");
    tolerate_previous_parent.assert().success();
}

#[test]
fn queue_completion_boundary_flags_completed_top_todo_left_active() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let cargo_toml = root.join("Cargo.toml");
    let changelog = root.join("CHANGELOG.md");
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");
    fs::write(src_dir.join("lib.rs"), "pub fn initial() {}\n").expect("write src/lib.rs");

    fs::write(
        &cargo_toml,
        "## TODO\n\n- Top item\n  - Goal: x\n  - Kind: `mixed`\n  - Boundary: `none`\n  - Contracts: `none`\n  - Scope: `src/`\n\n## [Trunk]\n",
    )
    .expect("write Cargo.toml");
    fs::write(
        &changelog,
        "# Changelog\n\n## TODO\n\n- Top item\n\n## [Trunk]\n",
    )
    .expect("write changelog");
    init_jj_repo(root);

    fs::write(src_dir.join("lib.rs"), "pub fn changed() {}\n").expect("update src/lib.rs");
    run_jj(
        root,
        &[
            "--config",
            "user.name=Afterburner Tests",
            "--config",
            "user.email=afterburner-tests@example.com",
            "commit",
            "-m",
            "work",
        ],
    );

    let mut check = cargo_bin_cmd!("workflow_queue_snapshot");
    check
        .arg("check-completion-boundary")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--repo-root")
        .arg(root);
    let assert = check.assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains("queue completion boundary violation")
            && stderr.contains("still appears active")
            && stderr.contains("just queue-resume"),
        "completion boundary must classify an unadvanced completed top TODO: {stderr}"
    );
}

#[test]
fn queue_top_runnable_check_flags_blocked_top_and_reports_next_runnable() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cargo_toml = tmp.path().join("Cargo.toml");

    fs::write(
        &cargo_toml,
        "## TODO\n\n- blocked item\n  - Goal: x\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `docs`\n  - Scope: `docs/workflows.md`\n  - Blocked-by: external dependency\n\n- runnable item\n  - Goal: y\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `docs`\n  - Scope: `docs/reference.md`\n\n## [Trunk]\n",
    )
    .expect("write Cargo.toml");

    let mut check = cargo_bin_cmd!("workflow_queue_snapshot");
    check
        .arg("check-top-runnable")
        .arg("--cargo-toml")
        .arg(&cargo_toml);
    let assert = check.assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains("top active TODO `blocked item` is blocked by `external dependency`")
            && stderr.contains("next runnable item: `runnable item`")
            && stderr.contains("just queue-promote-next-runnable"),
        "blocked top runnable check must report the next runnable item and repair path: {stderr}"
    );
}

#[test]
fn queue_promote_next_runnable_moves_blocked_top_below_runnable_item() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cargo_toml = tmp.path().join("Cargo.toml");

    fs::write(
        &cargo_toml,
        "## TODO\n\n- blocked item\n  - Goal: x\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `docs`\n  - Scope: `docs/workflows.md`\n  - Blocked-by: external dependency\n\n- runnable item\n  - Goal: y\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `docs`\n  - Scope: `docs/reference.md`\n\n## [Trunk]\n",
    )
    .expect("write Cargo.toml");

    let mut promote = cargo_bin_cmd!("workflow_queue_snapshot");
    promote
        .arg("promote-next-runnable")
        .arg("--cargo-toml")
        .arg(&cargo_toml);
    promote.assert().success();

    let updated = fs::read_to_string(&cargo_toml).expect("read updated Cargo.toml");
    let runnable_pos = updated.find("- runnable item").expect("runnable item remains");
    let blocked_pos = updated.find("- blocked item").expect("blocked item remains");
    assert!(
        runnable_pos < blocked_pos,
        "promote-next-runnable must move the next runnable item above the blocked top item"
    );

    let mut check = cargo_bin_cmd!("workflow_queue_snapshot");
    check
        .arg("check-top-runnable")
        .arg("--cargo-toml")
        .arg(&cargo_toml);
    check.assert().success();
}

#[test]
fn queue_completion_boundary_ignores_shared_docs_overlap_for_docs_family_todo() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let cargo_toml = root.join("Cargo.toml");
    let changelog = root.join("CHANGELOG.md");
    fs::create_dir_all(root.join("docs")).expect("create docs dir");
    fs::create_dir_all(root.join("tests")).expect("create tests dir");
    fs::write(root.join("docs").join("workflows.md"), "workflow v1\n").expect("write workflows");
    fs::write(root.join("docs").join("reference.md"), "reference v1\n").expect("write reference");
    fs::write(root.join("tests").join("unrelated_docs.rs"), "fn x() {}\n").expect("write test");

    fs::write(
        &cargo_toml,
        "## TODO\n\n- Docs family item\n  - Goal: x\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `docs`\n  - Scope: `docs/reference.md`, `docs/workflows.md`, `tests/`\n\n## [Trunk]\n",
    )
    .expect("write Cargo.toml");
    fs::write(
        &changelog,
        "# Changelog\n\n## TODO\n\n- Docs family item\n\n## [Trunk]\n",
    )
    .expect("write changelog");
    init_jj_repo(root);

    fs::write(root.join("docs").join("workflows.md"), "workflow v2\n").expect("update workflows");
    fs::write(root.join("docs").join("reference.md"), "reference v2\n").expect("update reference");
    fs::write(root.join("tests").join("unrelated_docs.rs"), "fn y() {}\n").expect("update test");
    run_jj(
        root,
        &[
            "--config",
            "user.name=Afterburner Tests",
            "--config",
            "user.email=afterburner-tests@example.com",
            "commit",
            "-m",
            "unrelated docs family work",
        ],
    );

    let mut check = cargo_bin_cmd!("workflow_queue_snapshot");
    check
        .arg("check-completion-boundary")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--repo-root")
        .arg(root);
    check.assert().success();
}

#[test]
fn queue_completion_boundary_still_flags_docs_family_when_specific_scope_is_touched() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let cargo_toml = root.join("Cargo.toml");
    let changelog = root.join("CHANGELOG.md");
    fs::create_dir_all(root.join("docs")).expect("create docs dir");
    fs::create_dir_all(root.join("tests")).expect("create tests dir");
    fs::write(root.join("docs").join("workflows.md"), "workflow v1\n").expect("write workflows");
    fs::write(root.join("docs").join("reference.md"), "reference v1\n").expect("write reference");
    fs::write(
        root.join("tests").join("drift_workflow_surface.rs"),
        "fn x() {}\n",
    )
    .expect("write drift test");

    fs::write(
        &cargo_toml,
        "## TODO\n\n- Docs family item\n  - Goal: x\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `docs`\n  - Scope: `docs/reference.md`, `docs/workflows.md`, `tests/drift_workflow_surface.rs`\n\n## [Trunk]\n",
    )
    .expect("write Cargo.toml");
    fs::write(
        &changelog,
        "# Changelog\n\n## TODO\n\n- Docs family item\n\n## [Trunk]\n",
    )
    .expect("write changelog");
    init_jj_repo(root);

    fs::write(
        root.join("tests").join("drift_workflow_surface.rs"),
        "fn y() {}\n",
    )
    .expect("update drift test");
    run_jj(
        root,
        &[
            "--config",
            "user.name=Afterburner Tests",
            "--config",
            "user.email=afterburner-tests@example.com",
            "commit",
            "-m",
            "specific docs family work",
        ],
    );

    let mut check = cargo_bin_cmd!("workflow_queue_snapshot");
    check
        .arg("check-completion-boundary")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--repo-root")
        .arg(root);
    let assert = check.assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains("queue completion boundary violation"),
        "specific docs-family scope touches must still trigger the completion boundary: {stderr}"
    );
}

#[test]
fn queue_snapshot_current_lineage_subcommands_hide_jj_fallback_shell() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let cargo_toml = root.join("Cargo.toml");
    let changelog = root.join("CHANGELOG.md");

    fs::write(
        &cargo_toml,
        "[package]\nname = \"x\"\n[package.metadata.git-cliff.changelog]\nheader = \"# Changelog\\n\\n## TODO\\n\\n- a\\n  - Goal: x\\n  - Kind: `mixed`\\n  - Boundary: `none`\\n  - Contracts: `none`\\n  - Scope: `src/`\\n\\n## [Trunk]\\n\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(&changelog, "# Changelog\n\n## TODO\n\n- a\n\n## [Trunk]\n")
        .expect("write changelog");
    init_jj_repo(root);

    let mut stamp = cargo_bin_cmd!("workflow_queue_snapshot");
    stamp
        .arg("stamp-current-parent")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--changelog")
        .arg(&changelog)
        .arg("--repo-root")
        .arg(root);
    stamp.assert().success();

    let mut verify = cargo_bin_cmd!("workflow_queue_snapshot");
    verify
        .arg("verify-current-lineage")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--changelog")
        .arg(&changelog)
        .arg("--repo-root")
        .arg(root);
    verify.assert().success();

    fs::write(root.join("README.md"), "touch lineage\n").expect("write README");
    run_jj(
        root,
        &[
            "--config",
            "user.name=Afterburner Tests",
            "--config",
            "user.email=afterburner-tests@example.com",
            "commit",
            "-m",
            "queue-only",
        ],
    );

    let mut tolerate_previous_parent = cargo_bin_cmd!("workflow_queue_snapshot");
    tolerate_previous_parent
        .arg("verify-current-lineage")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--changelog")
        .arg(&changelog)
        .arg("--repo-root")
        .arg(root);
    tolerate_previous_parent.assert().success();
}
