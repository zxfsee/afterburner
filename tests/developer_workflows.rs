use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn developer_workflows_are_repo_managed_and_documented() {
    let flake = repo_file("flake.nix");
    for tool in ["cargo-flamegraph", "git-cliff", "just", "nushell"] {
        assert!(
            flake.contains(tool),
            "flake dev shell must provide `{tool}`"
        );
    }

    let justfile = repo_file("justfile");
    for recipe in [
        "eval-gate:",
        "backend-profile-gate:",
        "profile-infer:",
        "workspace-gate:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }
    assert!(
        justfile.contains("cargo flamegraph"),
        "profile-infer must use cargo flamegraph"
    );
    assert!(
        justfile.contains("xcrun xctrace version"),
        "profiling workflow must document the host-profiler prerequisite"
    );
    assert!(
        justfile.contains("full Xcode"),
        "profiling workflow must document the full Xcode requirement on macOS"
    );

    let readme = repo_file("README.md");
    for workflow in [
        "just eval-gate",
        "just backend-profile-gate",
        "just profile-infer",
    ] {
        assert!(
            readme.contains(workflow),
            "README must document `{workflow}`"
        );
    }
    assert!(
        readme.contains("xcrun xctrace version"),
        "README must document the host-profiler prerequisite"
    );
    assert!(
        readme.contains("full Xcode"),
        "README must document the full Xcode requirement on macOS"
    );
}
