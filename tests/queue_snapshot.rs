use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

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
}
