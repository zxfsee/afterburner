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

    let expected_intro_lines = [
        "Contract and capability detail lives here so [README.md](../README.md) can stay a frontpage and navigation document.",
        "This index is broader than the public operator CLI. Many artifact families listed here are",
        "inspectable implementation surfaces that should remain behind `just` or `afterburner debug ...`",
        "unless they graduate into a clear operator intent.",
    ];
    let expected_rollout_link =
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map.";
    let expected_command_entrypoint_note =
        "For command entrypoints, use [docs/workflows.md](./workflows.md).";
    let expected_deployment_verification_flows = [
        "Receipt support flows: history/reconciliation and transport stay grouped under the receipt operator flow in [docs/workflows.md](./workflows.md).",
        "Receipt support flows: locator, rollback, and rollback supersession stay grouped under the receipt operator flow in [docs/workflows.md](./workflows.md).",
        "Bundle support flows: history/reconciliation and transport stay grouped under the bundle operator flow in [docs/workflows.md](./workflows.md).",
        "Bundle support flows: locator, rollback, and rollback supersession stay grouped under the bundle operator flow in [docs/workflows.md](./workflows.md).",
        "Handoff support flows: history/reconciliation and transport stay grouped under the handoff operator flow in [docs/workflows.md](./workflows.md).",
    ];
    let expected_capability_summary_lines = [
        "Current distributed capability surface remains single-device execution only while",
        "The distributed-training roadmap still targets the minimum useful capability subset",
        "`world_size`, ranks, and `device_group` metadata stay explicit so shard topology is",
        "Distributed runtime: workload-driven DP-first growth; explicit `world_size`/rank/`device_group` topology;",
        "Scheduler/control plane: the distributed training runtime decides how one job uses GPUs;",
        "Operator surfaces stay grouped under `afterburner deploy <subcommand>`, `afterburner verify <subcommand>`, `afterburner rollback <subcommand>`, `afterburner drift <subcommand>`, `afterburner cleanup <subcommand>`, `afterburner profile <subcommand>`, and `afterburner source <subcommand>`.",
        "Backend/runtime evolution stays measured:",
        "The `.mpk` to `.bpk` cutover now has one explicit inventory artifact, `burn_bpk_migration_surface_inventory.json`,",
        "Burn stable release availability gate: ADR-033 records the checked latest stable Burn line",
        "Profiling remains local-first and adapter-only:",
        "Provenance and retention stay explicit:",
        "Longer-horizon reference points remain explicit:",
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

    for intro_line in expected_intro_lines {
        assert!(
            reference.contains(intro_line),
            "reference index must keep the intro wording `{intro_line}`"
        );
    }

    assert!(
        reference.contains(expected_rollout_link),
        "reference index must keep the grouped rollout workflow linkage `{expected_rollout_link}`"
    );
    assert!(
        reference.contains(expected_command_entrypoint_note),
        "reference index must keep the command-entrypoint note `{expected_command_entrypoint_note}`"
    );

    for flow_line in expected_deployment_verification_flows {
        assert!(
            reference.contains(flow_line),
            "reference index must keep the grouped deployment-verification support-flow wording `{flow_line}`"
        );
    }

    for summary_line in expected_capability_summary_lines {
        assert!(
            reference.contains(summary_line),
            "reference index must keep the capability-summary wording `{summary_line}`"
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
