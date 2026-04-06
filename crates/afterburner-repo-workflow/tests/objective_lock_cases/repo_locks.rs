use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

#[test]
fn objective_lock_check_repo_locks_fails_on_stale_index_lock() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    fs::create_dir_all(root.join(".git")).expect("create .git");
    fs::write(root.join(".git/index.lock"), "").expect("write stale index lock");

    let assert = cargo_bin_cmd!("workflow_objective_lock")
        .arg("check-repo-locks")
        .arg("--repo-root")
        .arg(root)
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains(".git/index.lock"),
        "repo-lock check must point to the stale index lock path: {stderr}"
    );
    assert!(
        stderr.contains("just repo-lock-repair"),
        "repo-lock check must point to the typed repair path: {stderr}"
    );
}

#[test]
fn objective_lock_repair_repo_locks_removes_aged_index_lock() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    fs::create_dir_all(root.join(".git")).expect("create .git");
    fs::write(root.join(".git/index.lock"), "").expect("write stale index lock");

    cargo_bin_cmd!("workflow_objective_lock")
        .arg("repair-repo-locks")
        .arg("--repo-root")
        .arg(root)
        .arg("--stale-after-seconds")
        .arg("0")
        .assert()
        .success();

    assert!(
        !root.join(".git/index.lock").exists(),
        "repair-repo-locks must remove an aged index lock"
    );
}

#[test]
fn objective_lock_repair_repo_locks_rejects_recent_index_lock() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    fs::create_dir_all(root.join(".git")).expect("create .git");
    fs::write(root.join(".git/index.lock"), "").expect("write recent index lock");

    let assert = cargo_bin_cmd!("workflow_objective_lock")
        .arg("repair-repo-locks")
        .arg("--repo-root")
        .arg(root)
        .arg("--stale-after-seconds")
        .arg("3600")
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains("refusing automatic removal"),
        "repair-repo-locks must refuse to remove a recent index lock: {stderr}"
    );
    assert!(
        stderr.contains("inspect live Git/jj processes or retry later"),
        "repair-repo-locks must explain the next operator action for a recent index lock: {stderr}"
    );
    assert!(
        root.join(".git/index.lock").exists(),
        "repair-repo-locks must keep a recent index lock in place"
    );
}
