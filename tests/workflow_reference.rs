mod support;

use support::repo_file;

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
    assert!(
        workflows.contains("`justfile` is the executable source of truth"),
        "workflow reference must keep justfile as the executable source of truth"
    );
    for section in [
        "## Core repo workflows",
        "## Deployment",
        "## Validation and hygiene",
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
        "just build",
        "just check",
        "just fmt",
        "just workflows",
        "just train",
        "just infer",
        "just eval",
        "just deploy-check",
        "just workspace-gate",
        "just huggingface-publish",
        "just rollout-verify",
        "just cleanup-dry-run",
        "just profile-provenance-bundle",
        "just distributed-shard-lineage-bundle",
    ] {
        assert!(
            workflows.contains(workflow),
            "workflow reference must document `{workflow}`"
        );
    }
}
