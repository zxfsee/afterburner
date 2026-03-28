use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn developer_workflows_are_repo_managed_and_documented() {
    let flake = repo_file("flake.nix");
    for tool in [
        "cargo-flamegraph",
        "deploy-rs",
        "git-cliff",
        "just",
        "nushell",
    ] {
        assert!(
            flake.contains(tool),
            "flake dev shell must provide `{tool}`"
        );
    }

    let justfile = repo_file("justfile");
    for recipe in [
        "objective-lock-pin-queue:",
        "objective-lock-pin-backlog:",
        "objective-lock-pin-docs:",
        "objective-lock-pin-review:",
        "objective-lock-pin-execute-top-item:",
        "objective-lock-check-worktree action:",
        "objective-lock-clear:",
        "queue-refresh:",
        "queue-execute-preflight:",
        "workflow-surface-check-deployment-verification:",
        "eval-gate:",
        "backend-profile-gate:",
        "dashboard:",
        "deploy-check:",
        "profile-infer:",
        "changelog:",
        "queue-snapshot-check:",
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
    assert!(
        justfile.contains("XCTRACE=/usr/bin/xctrace")
            && justfile.contains("DEVELOPER_DIR")
            && justfile.contains("SDKROOT"),
        "profiling workflow must document the macOS xctrace override path"
    );

    let readme = repo_file("docs/workflows.md");
    for workflow in [
        "just objective-lock-pin-execute-top-item",
        "just objective-lock-pin-queue",
        "just objective-lock-check-worktree <action>",
        "just objective-lock-clear",
        "just queue-refresh",
        "just queue-execute-preflight",
        "just workflow-surface-check-deployment-verification",
        "just eval-gate",
        "just backend-profile-gate",
        "just dashboard",
        "just changelog",
        "just profile-infer",
        "just queue-snapshot-check",
    ] {
        assert!(
            readme.contains(workflow),
            "workflow reference must document `{workflow}`"
        );
    }
    assert!(
        readme.contains("xcrun xctrace version"),
        "workflow reference must document the host-profiler prerequisite"
    );
    assert!(
        readme.contains("full Xcode"),
        "workflow reference must document the full Xcode requirement on macOS"
    );
    assert!(
        readme.contains("XCTRACE=/usr/bin/xctrace")
            && readme.contains("DEVELOPER_DIR")
            && readme.contains("SDKROOT"),
        "workflow reference must document the macOS xctrace override path"
    );
}
