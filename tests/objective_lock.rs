use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn write_sample_cargo_toml(path: &Path) {
    fs::write(
        path,
        "[package.metadata.git-cliff.changelog]\nheader = \"\"\"# Changelog\n\n## TODO\n\n- Turn objective lock guard [Runtime Infra]\n  - Goal: Pin the current execution objective class and reject out-of-scope repo mutations before work proceeds so queue-only, backlog-only, docs-only, and execute-item turns cannot silently drift into each other.\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `ops`\n  - Scope: `src/bin/`, `tests/`, `justfile`, `docs/workflows.md`, `NOTE.md`\n\n## [Trunk]\n\"\"\"\n",
    )
    .expect("write Cargo.toml");
}

fn init_jj_repo(root: &Path) {
    run_jj(
        root,
        &[
            "--config",
            "user.name='Afterburner Tests'",
            "--config",
            "user.email='afterburner-tests@example.com'",
            "git",
            "init",
            ".",
        ],
    );
    run_jj(
        root,
        &[
            "--config",
            "user.name='Afterburner Tests'",
            "--config",
            "user.email='afterburner-tests@example.com'",
            "commit",
            "-m",
            "init",
        ],
    );
}

fn run_jj(root: &Path, args: &[&str]) {
    let status = Command::new("jj")
        .current_dir(root)
        .args(args)
        .status()
        .unwrap_or_else(|err| panic!("jj {}: {err}", args.join(" ")));
    assert!(status.success(), "jj {} must succeed", args.join(" "));
}

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
        stderr.contains("remove it and retry"),
        "repo-lock check must explain the minimal recovery action: {stderr}"
    );
}
