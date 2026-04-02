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
    mismatch.assert().failure();

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
