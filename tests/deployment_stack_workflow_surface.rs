use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

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
