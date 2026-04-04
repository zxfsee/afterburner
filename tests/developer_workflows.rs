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
        "objective-lock-pin-top-scope-fix:",
        "objective-lock-pin-backlog:",
        "objective-lock-pin-docs:",
        "objective-lock-pin-review:",
        "objective-lock-pin-execute-top-item:",
        "objective-lock-check-worktree action:",
        "objective-lock-clear:",
        "queue-refresh:",
        "queue-resume:",
        "queue-completion-boundary-check:",
        "queue-fix-top-scope:",
        "queue-execute-preflight repair='':",
        "changelog-top-scope-fix:",
        "workflow-surface-check-deployment-verification:",
        "workflow-surface-check-routing-orchestration:",
        "workflow-surface-check-distributed-shard-lineage:",
        "workflow-surface-check-pretraining-source:",
        "workflow-surface-check-drift:",
        "workflow-surface-check-cleanup:",
        "workflow-surface-check-profiling:",
        "workflow-surface-check-deployment-stack:",
        "workflow-surface-check-deployment-utility:",
        "workflow-surface-check-deployment-matrix:",
        "workflow-surface-check-public-cli:",
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
        justfile.contains("cargo run --locked --bin afterburner -- profile infer"),
        "profile-infer must delegate to the native profile infer command"
    );
    assert!(
        justfile.contains("--allow-existing-path CHANGELOG.md"),
        "stale-lineage resume flows must carry the repaired CHANGELOG baseline into execute pinning"
    );
    let readme = repo_file("docs/workflows.md");
    for workflow in [
        "just objective-lock-pin-execute-top-item",
        "just objective-lock-pin-top-scope-fix",
        "just objective-lock-pin-queue",
        "just objective-lock-check-worktree <action>",
        "just objective-lock-clear",
        "just queue-refresh",
        "just queue-resume",
        "just queue-fix-top-scope",
        "just queue-execute-preflight",
        "just queue-execute-preflight --repair-stale-snapshot",
        "just queue-completion-boundary-check",
        "just changelog-top-scope-fix",
        "just workflow-surface-check-deployment-verification",
        "just workflow-surface-check-distributed-shard-lineage",
        "just workflow-surface-check-pretraining-source",
        "just workflow-surface-check-drift",
        "just workflow-surface-check-cleanup",
        "just workflow-surface-check-profiling",
        "just workflow-surface-check-deployment-stack",
        "just workflow-surface-check-deployment-utility",
        "just workflow-surface-check-deployment-matrix",
        "just workflow-surface-check-public-cli",
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
        readme.contains("afterburner profile infer"),
        "workflow reference must document the native profile infer command"
    );
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
