use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn reference_doc_keeps_family_level_workflow_links_in_sync() {
    let justfile = repo_file("justfile");
    let reference = repo_file("docs/reference.md");

    let expected_links = [
        (
            "workflow-surface-check-deployment-stack:",
            "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-deployment-stack` for mechanical coverage.",
        ),
        (
            "workflow-surface-check-scheduler-heartbeat:",
            "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-scheduler-heartbeat` for mechanical coverage.",
        ),
        (
            "workflow-surface-check-deployment-verification:",
            "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-deployment-verification` for mechanical coverage.",
        ),
        (
            "workflow-surface-check-deployment-utility:",
            "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-deployment-utility` for mechanical coverage.",
        ),
        (
            "workflow-surface-check-drift:",
            "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-drift` for mechanical coverage.",
        ),
        (
            "workflow-surface-check-cleanup:",
            "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-cleanup` for mechanical coverage.",
        ),
        (
            "workflow-surface-check-profiling:",
            "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-profiling` for mechanical coverage.",
        ),
        (
            "workflow-surface-check-pretraining-source:",
            "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-pretraining-source` for mechanical coverage.",
        ),
        (
            "workflow-surface-check-distributed-shard-lineage:",
            "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-distributed-shard-lineage` for mechanical coverage.",
        ),
    ];

    let expected_headings = [
        "Rollout verification and pointer state:",
        "Scheduler heartbeat artifact groups:",
        "Deployment stack artifact groups:",
        "Deployment verification artifact groups:",
        "Pretraining source artifact groups:",
        "Distributed shard lineage artifact groups:",
    ];

    let expected_rollout_link =
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map.";
    let expected_deployment_verification_flows = [
        "Receipt support flows: history/reconciliation and transport stay grouped under the receipt operator flow in [docs/workflows.md](./workflows.md).",
        "Receipt support flows: locator, rollback, and rollback supersession stay grouped under the receipt operator flow in [docs/workflows.md](./workflows.md).",
        "Bundle support flows: history/reconciliation and transport stay grouped under the bundle operator flow in [docs/workflows.md](./workflows.md).",
        "Bundle support flows: locator, rollback, and rollback supersession stay grouped under the bundle operator flow in [docs/workflows.md](./workflows.md).",
        "Handoff support flows: history/reconciliation and transport stay grouped under the handoff operator flow in [docs/workflows.md](./workflows.md).",
    ];

    for (recipe, link_line) in expected_links {
        assert!(
            justfile.contains(recipe),
            "justfile must expose `{recipe}` so the reference link remains meaningful"
        );
        assert!(
            reference.contains(link_line),
            "reference index must keep the workflow linkage `{link_line}`"
        );
    }

    for heading in expected_headings {
        assert!(
            reference.contains(heading),
            "reference index must keep the grouped source/lineage wording `{heading}`"
        );
    }

    assert!(
        reference.contains(expected_rollout_link),
        "reference index must keep the grouped rollout workflow linkage `{expected_rollout_link}`"
    );

    for flow_line in expected_deployment_verification_flows {
        assert!(
            reference.contains(flow_line),
            "reference index must keep the grouped deployment-verification support-flow wording `{flow_line}`"
        );
    }

    let referenced_checks = reference
        .lines()
        .filter(|line| line.contains("workflow-surface-check-"))
        .collect::<Vec<_>>();
    for line in referenced_checks {
        let start = line
            .find("workflow-surface-check-")
            .expect("contains check");
        let check = &line[start..]
            .split('`')
            .next()
            .unwrap_or("workflow-surface-check-missing");
        let recipe = format!("{check}:");
        assert!(
            justfile.contains(&recipe),
            "reference index must not point at missing workflow surface check `{check}`"
        );
    }
}
