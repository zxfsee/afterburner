use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

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
        "Contract check: `just deploy-check` validates `deployment_target_profile.example.json`, `deployment_stack_profile.example.json`, and writes `deployment_stack_check.json`.",
        "Deployment stack launch and kube-rs lease workflows stay grouped under these families:",
        "Scheduler heartbeat stays grouped under these workflow families:",
        "Deployment utility workflows stay grouped under these entrypoints:",
        "`just workflow-surface-check-deployment-stack` keeps the grouped deployment-stack recipe and reference split checked.",
        "`just workflow-surface-check-scheduler-heartbeat` keeps the exhaustive CLI, fixture, and grouped recipe surface checked.",
        "`just workflow-surface-check-deployment-utility` keeps the grouped deployment-utility recipe and reference split checked.",
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
