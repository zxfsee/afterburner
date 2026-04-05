use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

#[test]
fn reference_doc_stays_grouped_and_navigation_first() {
    let justfile = repo_file("justfile");
    let reference = repo_file("docs/reference.md");

    for heading in [
        "## Training and inference contracts",
        "## Workflow and artifact families",
        "## Capability and stance summary",
    ] {
        assert!(
            reference.contains(heading),
            "reference index must keep the grouped heading `{heading}`"
        );
    }

    for line in [
        "Contract and capability detail lives here so [README.md](../README.md) can stay a frontpage and navigation document.",
        "This index is broader than the public operator CLI.",
        "For command entrypoints, use [docs/workflows.md](./workflows.md).",
    ] {
        assert!(
            reference.contains(line),
            "reference index must keep the ownership/navigation line `{line}`"
        );
    }

    for heading in [
        "Deployment stack artifact groups:",
        "Scheduler heartbeat artifact groups:",
        "Deployment verification artifact groups:",
        "Rollout verification and pointer state:",
        "Pretraining source artifact groups:",
        "Distributed shard lineage artifact groups:",
    ] {
        assert!(
            reference.contains(heading),
            "reference index must keep the grouped workflow-family heading `{heading}`"
        );
    }

    for line in [
        "Receipt support flows: history/reconciliation and transport stay grouped under the receipt operator flow in [docs/workflows.md](./workflows.md).",
        "Bundle support flows: locator, rollback, and rollback supersession stay grouped under the bundle operator flow in [docs/workflows.md](./workflows.md).",
        "Handoff support flows: history/reconciliation and transport stay grouped under the handoff operator flow in [docs/workflows.md](./workflows.md).",
    ] {
        assert!(
            reference.contains(line),
            "reference index must keep the grouped deployment-verification flow note `{line}`"
        );
    }

    for line in [
        "Distributed runtime and scheduler:",
        "Operator surface and backend fit:",
        "Profiling, provenance, and retention:",
        "Longer-horizon anchors:",
    ] {
        assert!(
            reference.contains(line),
            "reference index must keep the grouped capability-summary heading `{line}`"
        );
    }

    for line in reference
        .lines()
        .filter(|line| line.contains("workflow-surface-check-"))
    {
        let start = line
            .find("workflow-surface-check-")
            .expect("contains workflow surface check");
        let recipe = &line[start..].split('`').next().expect("check token");
        assert!(
            justfile.contains(&format!("{recipe}:")),
            "reference index must not point at missing workflow surface check `{recipe}`"
        );
    }
}
