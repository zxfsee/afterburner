use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

use crate::objective_lock_support::{init_jj_repo, write_sample_cargo_toml};

#[test]
fn objective_lock_worktree_rejects_queue_drift_and_allows_top_todo_scope() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    fs::create_dir_all(root.join("src/bin")).expect("create src/bin");
    fs::create_dir_all(root.join("tests")).expect("create tests");
    fs::create_dir_all(root.join("docs")).expect("create docs");
    write_sample_cargo_toml(&root.join("Cargo.toml"));
    fs::write(root.join("CHANGELOG.md"), "# Changelog\n").expect("write CHANGELOG.md");
    fs::write(root.join("justfile"), "workflows:\n    just --list\n").expect("write justfile");
    fs::write(root.join("NOTE.md"), "# NOTE.md\n").expect("write NOTE.md");
    fs::write(root.join("docs/workflows.md"), "# Workflows\n").expect("write workflows doc");
    init_jj_repo(root);

    fs::write(
        root.join("src/bin/objective_lock_guard.rs"),
        "fn main() {}\n",
    )
    .expect("write queued source edit");
    let lock_file = root.join(".git/afterburner/objective-lock.json");

    let mut pin_queue = cargo_bin_cmd!("workflow_objective_lock");
    pin_queue
        .arg("pin")
        .arg("--objective")
        .arg("queue-only")
        .arg("--lock-file")
        .arg(&lock_file)
        .arg("--expected-action")
        .arg("queue-refresh");
    pin_queue.assert().success();

    let mut queue_check = cargo_bin_cmd!("workflow_objective_lock");
    queue_check
        .arg("check-worktree")
        .arg("--repo-root")
        .arg(root)
        .arg("--lock-file")
        .arg(&lock_file)
        .arg("--action")
        .arg("queue-refresh");
    queue_check.assert().failure();

    let mut pin_execute = cargo_bin_cmd!("workflow_objective_lock");
    pin_execute
        .arg("pin")
        .arg("--objective")
        .arg("execute-top-item")
        .arg("--cargo-toml")
        .arg(root.join("Cargo.toml"))
        .arg("--lock-file")
        .arg(&lock_file)
        .arg("--expected-action")
        .arg("execute-top-item");
    pin_execute.assert().success();

    let mut execute_check = cargo_bin_cmd!("workflow_objective_lock");
    execute_check
        .arg("check-worktree")
        .arg("--repo-root")
        .arg(root)
        .arg("--lock-file")
        .arg(&lock_file)
        .arg("--action")
        .arg("execute-top-item");
    execute_check.assert().success();
}

#[test]
fn objective_lock_points_scope_repairs_to_top_scope_fix_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    fs::create_dir_all(root.join("src/bin")).expect("create src/bin");
    fs::create_dir_all(root.join("tests")).expect("create tests");
    fs::create_dir_all(root.join("docs")).expect("create docs");
    write_sample_cargo_toml(&root.join("Cargo.toml"));
    fs::write(root.join("CHANGELOG.md"), "# Changelog\n").expect("write CHANGELOG.md");
    fs::write(root.join("justfile"), "workflows:\n    just --list\n").expect("write justfile");
    fs::write(root.join("NOTE.md"), "# NOTE.md\n").expect("write NOTE.md");
    fs::write(root.join("docs/workflows.md"), "# Workflows\n").expect("write workflows doc");
    init_jj_repo(root);

    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"afterburner\"\n",
    )
    .expect("edit Cargo.toml");
    fs::write(root.join("CHANGELOG.md"), "# Changelog\n\n## TODO\n").expect("edit CHANGELOG.md");
    let lock_file = root.join(".git/afterburner/objective-lock.json");

    let mut pin_execute = cargo_bin_cmd!("workflow_objective_lock");
    pin_execute
        .arg("pin")
        .arg("--objective")
        .arg("execute-top-item")
        .arg("--cargo-toml")
        .arg(root.join("Cargo.toml"))
        .arg("--lock-file")
        .arg(&lock_file)
        .arg("--expected-action")
        .arg("execute-top-item");
    pin_execute.assert().failure();

    write_sample_cargo_toml(&root.join("Cargo.toml"));

    let mut pin_execute = cargo_bin_cmd!("workflow_objective_lock");
    pin_execute
        .arg("pin")
        .arg("--objective")
        .arg("execute-top-item")
        .arg("--cargo-toml")
        .arg(root.join("Cargo.toml"))
        .arg("--lock-file")
        .arg(&lock_file)
        .arg("--expected-action")
        .arg("execute-top-item");
    pin_execute.assert().success();

    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"afterburner\"\n",
    )
    .expect("edit Cargo.toml again");
    fs::write(root.join("CHANGELOG.md"), "# Changelog\n\n## TODO\n")
        .expect("edit CHANGELOG.md again");

    let assert = cargo_bin_cmd!("workflow_objective_lock")
        .arg("check-worktree")
        .arg("--repo-root")
        .arg(root)
        .arg("--lock-file")
        .arg(&lock_file)
        .arg("--action")
        .arg("execute-top-item")
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains("run `just queue-fix-top-scope` first"),
        "execute-top-item rejection must point stale top-scope repairs at queue-fix-top-scope: {stderr}"
    );
}

#[test]
fn objective_lock_top_scope_fix_allows_only_queue_metadata_paths() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    fs::create_dir_all(root.join("docs")).expect("create docs");
    write_sample_cargo_toml(&root.join("Cargo.toml"));
    fs::write(root.join("CHANGELOG.md"), "# Changelog\n").expect("write CHANGELOG.md");
    fs::write(root.join("docs/workflows.md"), "# Workflows\n").expect("write workflows doc");
    init_jj_repo(root);

    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"afterburner\"\n",
    )
    .expect("edit Cargo.toml");
    let lock_file = root.join(".git/afterburner/objective-lock.json");

    let mut pin_scope_fix = cargo_bin_cmd!("workflow_objective_lock");
    pin_scope_fix
        .arg("pin")
        .arg("--objective")
        .arg("top-scope-fix")
        .arg("--lock-file")
        .arg(&lock_file)
        .arg("--expected-action")
        .arg("top-scope-fix");
    pin_scope_fix.assert().success();

    let mut allow_check = cargo_bin_cmd!("workflow_objective_lock");
    allow_check
        .arg("check-worktree")
        .arg("--repo-root")
        .arg(root)
        .arg("--lock-file")
        .arg(&lock_file)
        .arg("--action")
        .arg("top-scope-fix");
    allow_check.assert().success();

    fs::write(root.join("docs/workflows.md"), "# Updated Workflows\n").expect("edit workflows");

    let mut reject_check = cargo_bin_cmd!("workflow_objective_lock");
    reject_check
        .arg("check-worktree")
        .arg("--repo-root")
        .arg(root)
        .arg("--lock-file")
        .arg(&lock_file)
        .arg("--action")
        .arg("top-scope-fix");
    reject_check.assert().failure();
}
