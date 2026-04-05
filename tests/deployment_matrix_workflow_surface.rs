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
