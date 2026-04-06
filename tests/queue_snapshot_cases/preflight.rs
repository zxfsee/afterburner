use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

use crate::queue_snapshot_support::init_jj_repo;

#[test]
fn execute_preflight_carries_repaired_queue_metadata_when_unchanged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let cargo_toml = root.join("Cargo.toml");
    let changelog = root.join("CHANGELOG.md");
    let src_dir = root.join("src");
    let docs_dir = root.join("docs");
    fs::create_dir_all(&src_dir).expect("create src dir");
    fs::create_dir_all(&docs_dir).expect("create docs dir");
    fs::write(src_dir.join("lib.rs"), "pub fn initial() {}\n").expect("write src/lib.rs");
    fs::write(docs_dir.join("workflows.md"), "# Workflows\n").expect("write docs/workflows.md");

    fs::write(
        &cargo_toml,
        "## TODO\n\n- Top item\n  - Goal: x\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `ops`\n  - Scope: `src/`\n\n## [Trunk]\n",
    )
    .expect("write Cargo.toml");
    fs::write(
        &changelog,
        "# Changelog\n\n## TODO\n\n- Top item\n\n## [Trunk]\n",
    )
    .expect("write changelog");
    init_jj_repo(root);

    fs::write(
        &cargo_toml,
        "## TODO\n\n- Top item\n  - Goal: x\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `ops`\n  - Scope: `src/`, `docs/workflows.md`\n\n## [Trunk]\n",
    )
    .expect("widen Cargo.toml scope");

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

    let mut preflight = cargo_bin_cmd!("workflow_queue_snapshot");
    preflight
        .arg("execute-preflight")
        .arg("--cargo-toml")
        .arg(&cargo_toml)
        .arg("--changelog")
        .arg(&changelog)
        .arg("--repo-root")
        .arg(root);
    preflight.assert().success();
}
