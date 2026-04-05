use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn readme_points_to_reference_docs_and_stays_navigation_first() {
    let readme = repo_file("README.md");
    assert!(
        readme.contains("[Reference index](./docs/reference.md)"),
        "README frontpage must point readers to the reference index"
    );
    assert!(
        readme.contains(
            "Deeper contract and capability detail now lives in [docs/reference.md](./docs/reference.md)."
        ),
        "README must keep the deeper reference split explicit"
    );

    let reference = repo_file("docs/reference.md");
    for heading in [
        "## Training and inference contracts",
        "## Workflow and artifact families",
        "## Capability and stance summary",
    ] {
        assert!(
            reference.contains(heading),
            "reference index must contain `{heading}`"
        );
    }
    for needle in [
        "Deployment:",
        "artifact_upload_request.json",
        "Scheduler heartbeat artifact groups:",
        "Deployment stack artifact groups:",
        "Deployment verification artifact groups:",
        "Receipt support flows: history/reconciliation and transport stay grouped under the receipt operator flow in [docs/workflows.md](./workflows.md).",
        "Receipt support flows: locator, rollback, and rollback supersession stay grouped under the receipt operator flow in [docs/workflows.md](./workflows.md).",
        "Bundle support flows: history/reconciliation and transport stay grouped under the bundle operator flow in [docs/workflows.md](./workflows.md).",
        "Bundle support flows: locator, rollback, and rollback supersession stay grouped under the bundle operator flow in [docs/workflows.md](./workflows.md).",
        "Handoff support flows: history/reconciliation and transport stay grouped under the handoff operator flow in [docs/workflows.md](./workflows.md).",
        "Rollout verification and pointer state:",
        "Cleanup:",
        "`artifact_cleanup_inventory.json`",
        "`artifact_cleanup_policy.json`",
        "`artifact_cleanup_dry_run_receipt.json`",
        "`artifact_cleanup_execution_receipt.json`",
        "`artifact_cleanup_evidence_bundle.json`",
        "Profiling:",
        "`infer_hotspot_summary.json`",
        "`profiling_environment_snapshot.json`",
        "`profiling_environment_snapshot_refresh.json`",
        "`profiling_provenance_receipt.json`",
        "Drift and rollback:",
        "`infer_output_drift_summary.json`",
        "`infer_output_drift_receipt.json`",
        "`infer_output_drift_baseline.json`",
        "`infer_output_drift_baseline_approval.json`",
        "`infer_output_drift_baseline_history.json`",
        "`infer_output_drift_baseline_checkpoint.json`",
        "`infer_output_drift_baseline_bundle.json`",
        "`infer_output_drift_baseline_handoff.json`",
        "`infer_output_drift_baseline_transport_locator.json`",
        "`infer_output_drift_baseline_rollback.json`",
        "`infer_output_drift_baseline_supersession.json`",
        "`infer_output_drift_baseline_refresh.json`",
        "Source and lineage:",
        "Pretraining source artifact groups:",
        "Distributed shard lineage artifact groups:",
        "`deployment_verification_evidence_bundle*.json`",
        "infer_output_drift_baseline_pointer.json",
        "profiling_provenance_evidence_bundle.json",
        "distributed_shard_lineage_evidence_handoff.json",
        "remote locator contract",
        "deployment verification evidence provenance",
        "artifact retention envelope",
        "afterburner deploy <subcommand>",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must contain `{needle}`"
        );
    }
}
