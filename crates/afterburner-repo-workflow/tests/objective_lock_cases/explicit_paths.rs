use assert_cmd::cargo::cargo_bin_cmd;
use std::path::PathBuf;

#[test]
fn objective_lock_checks_action_and_explicit_paths() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let lock_file = PathBuf::from(tmp.path()).join("objective-lock.json");

    let mut pin_docs = cargo_bin_cmd!("workflow_objective_lock");
    pin_docs
        .arg("pin")
        .arg("--objective")
        .arg("docs-only")
        .arg("--lock-file")
        .arg(&lock_file)
        .arg("--expected-action")
        .arg("docs-edit");
    pin_docs.assert().success();

    let mut docs_check = cargo_bin_cmd!("workflow_objective_lock");
    docs_check
        .arg("check-paths")
        .arg("--lock-file")
        .arg(&lock_file)
        .arg("--action")
        .arg("docs-edit")
        .arg("--path")
        .arg("docs/workflows.md")
        .arg("--path")
        .arg("README.md");
    docs_check.assert().success();

    let mut path_mismatch = cargo_bin_cmd!("workflow_objective_lock");
    path_mismatch
        .arg("check-paths")
        .arg("--lock-file")
        .arg(&lock_file)
        .arg("--action")
        .arg("docs-edit")
        .arg("--path")
        .arg("Cargo.toml");
    path_mismatch.assert().failure();

    let mut action_mismatch = cargo_bin_cmd!("workflow_objective_lock");
    action_mismatch
        .arg("check-paths")
        .arg("--lock-file")
        .arg(&lock_file)
        .arg("--action")
        .arg("queue-refresh")
        .arg("--path")
        .arg("README.md");
    action_mismatch.assert().failure();
}
