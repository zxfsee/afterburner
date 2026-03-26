use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn workflow_reference_is_split_from_frontpage() {
    let readme = repo_file("README.md");
    assert!(
        readme.contains("[Workflow reference](./docs/workflows.md)"),
        "README frontpage must point operators to the workflow reference"
    );
    assert!(
        readme.contains("Detailed operational workflow reference lives in [docs/workflows.md](./docs/workflows.md)."),
        "README must keep the operational detail split explicit"
    );

    let workflows = repo_file("docs/workflows.md");
    for section in [
        "## Deployment",
        "## Rollout verification",
        "## Cleanup",
        "## Drift",
        "## Profiling",
        "## Source and lineage",
    ] {
        assert!(
            workflows.contains(section),
            "workflow reference must contain `{section}`"
        );
    }

    for workflow in [
        "just deploy-check",
        "just rollout-verify",
        "just cleanup-dry-run",
        "just profile-provenance-bundle",
        "just distributed-shard-lineage-handoff",
    ] {
        assert!(
            workflows.contains(workflow),
            "workflow reference must document `{workflow}`"
        );
    }
}
