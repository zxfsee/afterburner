use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

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
fn queue_top_runnable_check_reports_runnable_backlog_item_when_active_queue_has_none() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cargo_toml = tmp.path().join("Cargo.toml");
    let backlog = tmp.path().join("backlog.md");

    fs::write(
        &cargo_toml,
        "## TODO\n\n- blocked item\n  - Goal: x\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `docs`\n  - Scope: `docs/workflows.md`\n  - Blocked-by: external dependency\n\n## [Trunk]\n",
    )
    .expect("write Cargo.toml");
    fs::write(
        &backlog,
        "# Backlog\n\n## Items\n\n- blocked backlog item\n  - Goal: z\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `docs`\n  - Scope: `docs/backlog.md`\n  - Blocked-by: external dependency\n\n- runnable backlog item\n  - Goal: y\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `docs`\n  - Scope: `docs/reference.md`\n",
    )
    .expect("write backlog");

    let mut check = cargo_bin_cmd!("workflow_queue_snapshot");
    check
        .arg("check-top-runnable")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--backlog")
        .arg(&backlog);
    let assert = check.assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains("top active TODO `blocked item` is blocked by `external dependency`")
            && stderr.contains("next runnable backlog item: `runnable backlog item`")
            && stderr.contains("just queue-promote-next-runnable"),
        "blocked top runnable check must report the next runnable backlog item and repair path: {stderr}"
    );
}

#[test]
fn queue_top_runnable_check_reports_runnable_backlog_item_when_active_queue_is_empty() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cargo_toml = tmp.path().join("Cargo.toml");
    let backlog = tmp.path().join("backlog.md");

    fs::write(&cargo_toml, "## TODO\n\n## [Trunk]\n").expect("write Cargo.toml");
    fs::write(
        &backlog,
        "# Backlog\n\n## Items\n\n- runnable backlog item\n  - Goal: y\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `docs`\n  - Scope: `docs/reference.md`\n",
    )
    .expect("write backlog");

    let mut check = cargo_bin_cmd!("workflow_queue_snapshot");
    check
        .arg("check-top-runnable")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--backlog")
        .arg(&backlog);
    let assert = check.assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains("active TODO queue is empty")
            && stderr.contains("next runnable backlog item: `runnable backlog item`")
            && stderr.contains("just queue-promote-next-runnable"),
        "empty active queue check must report the next runnable backlog item and repair path: {stderr}"
    );
}

#[test]
fn queue_top_runnable_check_accepts_empty_active_queue_when_backlog_has_no_runnable_item() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cargo_toml = tmp.path().join("Cargo.toml");
    let backlog = tmp.path().join("backlog.md");

    fs::write(&cargo_toml, "## TODO\n\n## [Trunk]\n").expect("write Cargo.toml");
    fs::write(
        &backlog,
        "# Backlog\n\n## Items\n\n- blocked backlog item\n  - Goal: z\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `docs`\n  - Scope: `docs/backlog.md`\n  - Blocked-by: external dependency\n",
    )
    .expect("write backlog");

    let mut check = cargo_bin_cmd!("workflow_queue_snapshot");
    check
        .arg("check-top-runnable")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--backlog")
        .arg(&backlog);
    check.assert().success();
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
    let runnable_pos = updated
        .find("- runnable item")
        .expect("runnable item remains");
    let blocked_pos = updated
        .find("- blocked item")
        .expect("blocked item remains");
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
fn queue_promote_next_runnable_promotes_backlog_item_when_active_queue_has_none() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cargo_toml = tmp.path().join("Cargo.toml");
    let backlog = tmp.path().join("backlog.md");

    fs::write(
        &cargo_toml,
        "## TODO\n\n- blocked item\n  - Goal: x\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `docs`\n  - Scope: `docs/workflows.md`\n  - Blocked-by: external dependency\n\n## [Trunk]\n",
    )
    .expect("write Cargo.toml");
    fs::write(
        &backlog,
        "# Backlog\n\n## Items\n\n- blocked backlog item\n  - Goal: z\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `docs`\n  - Scope: `docs/backlog.md`\n  - Blocked-by: external dependency\n\n- runnable backlog item\n  - Goal: y\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `docs`\n  - Scope: `docs/reference.md`\n",
    )
    .expect("write backlog");

    let mut promote = cargo_bin_cmd!("workflow_queue_snapshot");
    promote
        .arg("promote-next-runnable")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--backlog")
        .arg(&backlog);
    promote.assert().success();

    let updated = fs::read_to_string(&cargo_toml).expect("read updated Cargo.toml");
    let promoted_pos = updated
        .find("- runnable backlog item")
        .expect("promoted backlog item remains");
    let blocked_pos = updated
        .find("- blocked item")
        .expect("blocked item remains");
    assert!(
        promoted_pos < blocked_pos,
        "promote-next-runnable must move the highest-priority runnable backlog item above the blocked top item when the active queue has no runnable work"
    );

    let updated_backlog = fs::read_to_string(&backlog).expect("read updated backlog");
    assert!(
        !updated_backlog.contains("- runnable backlog item"),
        "promoted backlog item must be removed from the backlog"
    );
    assert!(
        updated_backlog.contains("- blocked backlog item"),
        "blocked backlog item must remain parked"
    );

    let mut check = cargo_bin_cmd!("workflow_queue_snapshot");
    check
        .arg("check-top-runnable")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--backlog")
        .arg(&backlog);
    check.assert().success();
}
