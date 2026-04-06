use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

use crate::objective_lock_support::{init_jj_repo, write_sample_cargo_toml};

#[test]
fn objective_lock_can_carry_existing_queue_metadata_into_execute_resume_only_if_unchanged() {
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
        root.join("CHANGELOG.md"),
        "# Changelog\n\nresume baseline\n",
    )
    .expect("edit CHANGELOG.md");
    let lock_file = root.join(".git/afterburner/objective-lock.json");

    let mut pin_execute = cargo_bin_cmd!("workflow_objective_lock");
    pin_execute
        .arg("pin")
        .arg("--objective")
        .arg("execute-top-item")
        .arg("--cargo-toml")
        .arg(root.join("Cargo.toml"))
        .arg("--repo-root")
        .arg(root)
        .arg("--allow-existing-path")
        .arg("CHANGELOG.md")
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

    fs::write(
        root.join("CHANGELOG.md"),
        "# Changelog\n\nresume baseline\nchanged again\n",
    )
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
        stderr.contains("CHANGELOG.md"),
        "execute resume must reject further queue metadata drift after the carried baseline: {stderr}"
    );
}

#[test]
fn objective_lock_can_carry_existing_in_scope_paths_into_top_scope_fix_only_if_unchanged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).expect("create src");
    fs::create_dir_all(root.join("docs")).expect("create docs");
    write_sample_cargo_toml(&root.join("Cargo.toml"));
    fs::write(root.join("CHANGELOG.md"), "# Changelog\n").expect("write CHANGELOG.md");
    fs::write(root.join("src/lib.rs"), "pub fn x() {}\n").expect("write src/lib.rs");
    init_jj_repo(root);

    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"afterburner\"\n",
    )
    .expect("edit Cargo.toml");
    fs::write(root.join("CHANGELOG.md"), "# Changelog\n\nrepaired\n").expect("edit CHANGELOG.md");
    fs::write(root.join("src/lib.rs"), "pub fn repaired() {}\n").expect("edit src/lib.rs");
    let lock_file = root.join(".git/afterburner/objective-lock.json");

    let mut pin_scope_fix = cargo_bin_cmd!("workflow_objective_lock");
    pin_scope_fix
        .arg("pin")
        .arg("--objective")
        .arg("top-scope-fix")
        .arg("--cargo-toml")
        .arg(root.join("Cargo.toml"))
        .arg("--repo-root")
        .arg(root)
        .arg("--allow-existing-path")
        .arg("src/lib.rs")
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

    fs::write(root.join("src/lib.rs"), "pub fn changed_again() {}\n")
        .expect("edit src/lib.rs again");

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
