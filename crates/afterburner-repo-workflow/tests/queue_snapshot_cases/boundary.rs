use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

use crate::queue_snapshot_support::{init_jj_repo, run_jj};

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
