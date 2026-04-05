#[path = "support/markdown.rs"]
mod markdown_test_support;
#[path = "support/repo.rs"]
mod repo_test_support;

use markdown_test_support::markdown_section;
use repo_test_support::repo_file;

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

    for fragment in [
        "Contract and capability detail lives here",
        "[README.md](../README.md)",
        "broader than the public operator CLI",
        "For command entrypoints, use [docs/workflows.md](./workflows.md).",
    ] {
        assert!(
            reference.contains(fragment),
            "reference index must keep the ownership/navigation fragment `{fragment}`"
        );
    }

    let workflow_families = markdown_section(&reference, "## Workflow and artifact families");
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

    for fragment in [
        "Receipt support flows:",
        "receipt operator flow in [docs/workflows.md](./workflows.md).",
        "Bundle support flows:",
        "bundle operator flow in [docs/workflows.md](./workflows.md).",
        "Handoff support flows:",
        "handoff operator flow in [docs/workflows.md](./workflows.md).",
    ] {
        assert!(
            workflow_families.contains(fragment),
            "workflow/artifact families section must keep the grouped deployment-verification fragment `{fragment}`"
        );
    }

    let capability_summary = markdown_section(&reference, "## Capability and stance summary");
    for line in [
        "Distributed runtime and scheduler:",
        "Operator surface and backend fit:",
        "Profiling, provenance, and retention:",
        "Longer-horizon anchors:",
    ] {
        assert!(
            capability_summary.contains(line),
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
