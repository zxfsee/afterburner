#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn deployment_workflow_matrix_stays_grouped_by_operator_family() {
    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("workflow-surface-check-deployment-matrix:"),
        "justfile must expose `workflow-surface-check-deployment-matrix:`"
    );

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Deployment stays grouped under these operator-facing families:",
        "Contract check: `just deploy-check`.",
        "Deployment stack launch and kube-rs lease stay grouped under these operator flows:",
        "Scheduler heartbeat stays grouped under these operator flows:",
        "Deployment utility stays grouped under these operator flows:",
        "`just workflow-surface-check-deployment-stack` guards the grouped deployment-stack workflow map and reference split.",
        "`just workflow-surface-check-scheduler-heartbeat` keeps the grouped heartbeat workflow map and contract surface checked.",
        "`just workflow-surface-check-deployment-utility` guards the grouped deployment-utility workflow map and reference split.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped deployment matrix surface `{needle}`"
        );
    }

    for forbidden in [
        "- `just deploy-launch-plan` writes `deployment_stack_launch_plan.json`.",
        "- `just deploy-launch-receipt` writes `deployment_stack_launch_receipt.json`.",
        "- `just scheduler-runtime-simulate` writes `scheduler_runtime_simulation_report.json`.",
        "- `just distributed-load-profile` writes `distributed_load_profile.json`.",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must not regress to flat deployment artifact listing `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Deployment stack artifact groups:",
        "`deployment_stack_launch_plan.json`",
        "`kube_rs_gpu_lease_reconciliation.json`",
        "`scheduler_runtime_simulation_report.json`",
        "`distributed_load_profile.json`",
        "`huggingface_publish_receipt.json`",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep deployment matrix artifact detail `{needle}`"
        );
    }
}

#[test]
fn deployment_stack_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "deploy-launch-plan target_profile stack_profile:",
        "deploy-launch-receipt plan port launched_at_unix_ms:",
        "deploy-launch-bundle receipt:",
        "deploy-launch-bundle-reconcile receipt bundle:",
        "deploy-launch-handoff bundle:",
        "deploy-launch-handoff-reconcile bundle handoff:",
        "kube-rs-lease-reconcile lease namespace resource_name:",
        "workflow-surface-check-deployment-stack:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Deployment stack launch and kube-rs lease stay grouped under these operator flows:",
        "Launch planning: `just deploy-launch-plan`, `just deploy-launch-receipt`.",
        "Bundle flow: `just deploy-launch-bundle`, `just deploy-launch-bundle-reconcile`.",
        "Handoff flow: `just deploy-launch-handoff`, `just deploy-launch-handoff-reconcile`.",
        "Launch support artifacts for history, locator, transport, and reconciliation stay grouped behind `afterburner deploy record-launch-...`, `afterburner deploy point-launch-...`, and `afterburner deploy reconcile-launch-...`.",
        "Kube lease flow: `just kube-rs-lease-reconcile`.",
        "Kube lease pointer and history support artifacts stay behind `afterburner deploy point-kube-rs-lease` and `afterburner deploy record-kube-rs-lease-...`.",
        "`just workflow-surface-check-deployment-stack` guards the grouped deployment-stack workflow map and reference split.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped deployment-stack surface `{needle}`"
        );
    }

    for forbidden in [
        "- `just deploy-launch-plan` writes `deployment_stack_launch_plan.json`.",
        "- `just deploy-launch-receipt` writes `deployment_stack_launch_receipt.json`.",
        "- `just deploy-launch-bundle` writes `deployment_stack_launch_evidence_bundle.json`.",
        "- `just deploy-launch-bundle-reconcile` writes `deployment_stack_launch_evidence_bundle_reconciliation.json`.",
        "- `just deploy-launch-handoff` writes `deployment_stack_launch_evidence_handoff.json`.",
        "- `just deploy-launch-handoff-reconcile` writes `deployment_stack_launch_evidence_handoff_reconciliation.json`.",
        "- `just kube-rs-lease-reconcile` writes `kube_rs_gpu_lease_reconciliation.json`.",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating deployment-stack variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "`deployment_stack_launch_plan.json`",
        "`deployment_stack_launch_receipt.json`",
        "`deployment_stack_launch_evidence_bundle.json`",
        "`deployment_stack_launch_evidence_bundle_reconciliation.json`",
        "`deployment_stack_launch_evidence_bundle_reconciliation_history.json`",
        "`deployment_stack_launch_evidence_handoff.json`",
        "`deployment_stack_launch_evidence_handoff_reconciliation.json`",
        "`deployment_stack_launch_evidence_handoff_history.json`",
        "`deployment_stack_launch_evidence_handoff_reconciliation_history.json`",
        "`deployment_stack_launch_transport_locator.json`",
        "`deployment_stack_launch_transport_locator_history.json`",
        "`deployment_stack_launch_transport_locator_reconciliation.json`",
        "`deployment_stack_launch_transport_locator_reconciliation_history.json`",
        "`deployment_stack_launch_locator_pointer.json`",
        "`deployment_stack_launch_locator_reconciliation.json`",
        "`deployment_stack_launch_locator_reconciliation_history.json`",
        "`kube_rs_gpu_lease_reconciliation.json`",
        "`kube_rs_gpu_lease_reconciliation_history.json`",
        "`kube_rs_gpu_lease_pointer.json`",
        "`kube_rs_gpu_lease_history.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-deployment-stack` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped deployment-stack surface `{needle}`"
        );
    }
}

#[test]
fn deployment_utility_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "scheduler-runtime-simulate scenario:",
        "burn-bpk-migration-surface-report:",
        "distributed-load-profile addr requests concurrency latency_budget_ms_p99 error_budget_ratio:",
        "huggingface-publish request:",
        "workflow-surface-check-deployment-utility:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Deployment utility stays grouped under these operator flows:",
        "Scheduler/runtime simulation: `just scheduler-runtime-simulate`.",
        "Burn migration inventory: `just burn-bpk-migration-surface-report`, `cargo nextest run --locked --test burn_stable_release_availability`.",
        "Load and publish utilities: `just distributed-load-profile`, `afterburner deploy hf-publish --request <path>`.",
        "`just workflow-surface-check-deployment-utility` guards the grouped deployment-utility workflow map and reference split.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped deployment-utility surface `{needle}`"
        );
    }

    for forbidden in [
        "- `just scheduler-runtime-simulate` writes `scheduler_runtime_simulation_report.json`.",
        "- `just burn-bpk-migration-surface-report` writes `burn_bpk_migration_surface_inventory.json`.",
        "- `just distributed-load-profile` writes `distributed_load_profile.json`.",
        "- `afterburner deploy hf-publish --request <path>` writes `huggingface_publish_receipt.json`.",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating deployment utility variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "`scheduler_runtime_simulation_report.json`",
        "`burn_bpk_migration_surface_inventory.json`",
        "`distributed_load_profile.json`",
        "`huggingface_publish_receipt.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-deployment-utility` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped deployment-utility surface `{needle}`"
        );
    }
}
