use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn deployment_stack_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "deploy-launch-plan target_profile stack_profile:",
        "deploy-launch-receipt plan port launched_at_unix_ms:",
        "deploy-launch-bundle receipt:",
        "deploy-reconcile-launch-bundle receipt bundle:",
        "deploy-record-launch-bundle-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deploy-launch-handoff bundle:",
        "deploy-reconcile-launch-handoff bundle handoff:",
        "deploy-record-launch-handoff-history handoff event recorded_at_unix_ms:",
        "deploy-record-launch-handoff-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deploy-point-launch-transport-locator handoff:",
        "deploy-record-launch-transport-locator-history locator event recorded_at_unix_ms:",
        "deploy-reconcile-launch-transport-locator handoff locator:",
        "deploy-record-launch-transport-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deploy-point-launch-locator locator:",
        "deploy-reconcile-launch-locator plan pointer:",
        "deploy-record-launch-locator-history pointer event recorded_at_unix_ms:",
        "deploy-record-launch-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "kube-rs-lease-reconcile lease namespace resource_name:",
        "kube-rs-lease-point reconciliation:",
        "kube-rs-lease-record-history pointer event recorded_at_unix_ms:",
        "kube-rs-lease-record-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "workflow-surface-check-deployment-stack:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Deployment stack launch and kube-rs lease workflows stay grouped under these families:",
        "Launch planning: `just deploy-launch-plan`, `just deploy-launch-receipt`.",
        "Bundle flow: `just deploy-launch-bundle`, `just deploy-reconcile-launch-bundle`, `just deploy-record-launch-bundle-reconciliation-history`.",
        "Handoff flow: `just deploy-launch-handoff`, `just deploy-reconcile-launch-handoff`, `just deploy-record-launch-handoff-history`, `just deploy-record-launch-handoff-reconciliation-history`.",
        "Locator flow: `just deploy-point-launch-transport-locator`, `just deploy-record-launch-transport-locator-history`, `just deploy-reconcile-launch-transport-locator`, `just deploy-record-launch-transport-locator-reconciliation-history`, `just deploy-point-launch-locator`, `just deploy-reconcile-launch-locator`, `just deploy-record-launch-locator-history`, `just deploy-record-launch-locator-reconciliation-history`.",
        "Kube lease flow: `just kube-rs-lease-reconcile`, `just kube-rs-lease-point`, `just kube-rs-lease-record-history`, `just kube-rs-lease-record-reconciliation-history`.",
        "`just workflow-surface-check-deployment-stack` keeps the grouped deployment-stack recipe and reference split checked.",
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
        "- `just deploy-reconcile-launch-bundle` writes `deployment_stack_launch_evidence_bundle_reconciliation.json`.",
        "- `just deploy-record-launch-bundle-reconciliation-history` writes `deployment_stack_launch_evidence_bundle_reconciliation_history.json`.",
        "- `just deploy-launch-handoff` writes `deployment_stack_launch_evidence_handoff.json`.",
        "- `just deploy-reconcile-launch-handoff` writes `deployment_stack_launch_evidence_handoff_reconciliation.json`.",
        "- `just deploy-record-launch-handoff-history` writes `deployment_stack_launch_evidence_handoff_history.json`.",
        "- `just deploy-record-launch-handoff-reconciliation-history` writes `deployment_stack_launch_evidence_handoff_reconciliation_history.json`.",
        "- `just deploy-point-launch-transport-locator` writes `deployment_stack_launch_transport_locator.json`.",
        "- `just deploy-record-launch-transport-locator-history` writes `deployment_stack_launch_transport_locator_history.json`.",
        "- `just deploy-reconcile-launch-transport-locator` writes `deployment_stack_launch_transport_locator_reconciliation.json`.",
        "- `just deploy-record-launch-transport-locator-reconciliation-history` writes `deployment_stack_launch_transport_locator_reconciliation_history.json`.",
        "- `just deploy-point-launch-locator` writes `deployment_stack_launch_locator_pointer.json`.",
        "- `just deploy-reconcile-launch-locator` writes `deployment_stack_launch_locator_reconciliation.json`.",
        "- `just deploy-record-launch-locator-history` writes `deployment_stack_launch_locator_history.json`.",
        "- `just deploy-record-launch-locator-reconciliation-history` writes `deployment_stack_launch_locator_reconciliation_history.json`.",
        "- `just kube-rs-lease-reconcile` writes `kube_rs_gpu_lease_reconciliation.json`.",
        "- `just kube-rs-lease-point` writes `kube_rs_gpu_lease_pointer.json`.",
        "- `just kube-rs-lease-record-history` writes `kube_rs_gpu_lease_history.json`.",
        "- `just kube-rs-lease-record-reconciliation-history` writes `kube_rs_gpu_lease_reconciliation_history.json`.",
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
        "Use [docs/workflows.md](./workflows.md) for the grouped entrypoint map and `workflow-surface-check-deployment-stack` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped deployment-stack surface `{needle}`"
        );
    }
}
