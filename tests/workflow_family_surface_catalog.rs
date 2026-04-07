use std::path::PathBuf;

#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

fn fixture_exists(path: &str) -> bool {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(path)
        .exists()
}

#[test]
fn cleanup_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "cleanup-inventory:",
        "cleanup-policy profile:",
        "cleanup-dry-run inventory policy generated_at_unix_ms:",
        "cleanup-execute dry_run_receipt artifacts_root executed_at_unix_ms:",
        "cleanup-evidence-bundle execution_receipt:",
        "workflow-surface-check-cleanup:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Cleanup stays grouped under these operator flows:",
        "Scope and policy: `just cleanup-inventory`, `just cleanup-policy`.",
        "Plan and execute: `just cleanup-dry-run`, `just cleanup-execute`.",
        "Evidence pack: `just cleanup-evidence-bundle`.",
        "`just workflow-surface-check-cleanup` guards the grouped cleanup workflow map and reference split.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped cleanup surface `{needle}`"
        );
    }

    for forbidden in [
        "- `just cleanup-inventory` writes `artifact_cleanup_inventory.json`.",
        "- `just cleanup-policy` writes `artifact_cleanup_policy.json`.",
        "- `just cleanup-dry-run` writes `artifact_cleanup_dry_run_receipt.json`.",
        "- `just cleanup-execute` writes `artifact_cleanup_execution_receipt.json`.",
        "- `just cleanup-evidence-bundle` writes `artifact_cleanup_evidence_bundle.json`.",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating cleanup variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Cleanup:",
        "`artifact_cleanup_inventory.json`",
        "`artifact_cleanup_policy.json`",
        "`artifact_cleanup_dry_run_receipt.json`",
        "`artifact_cleanup_execution_receipt.json`",
        "`artifact_cleanup_evidence_bundle.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-cleanup` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped cleanup surface `{needle}`"
        );
    }
}

#[test]
fn drift_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "drift-receipt candidate_summary current_summary policy:",
        "drift-baseline summary receipt:",
        "drift-approve-baseline baseline approved_by approval_ticket approved_at_unix_ms:",
        "drift-refresh-baseline summary receipt current_baseline:",
        "workflow-surface-check-drift:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Drift stays grouped under these operator flows:",
        "Signal capture: `just drift-receipt`.",
        "Baseline setup: `just drift-baseline`, `just drift-approve-baseline`.",
        "Detailed baseline state, checkpointing, export, transport, rollback, and supersession artifacts stay behind `afterburner debug drift baseline ...`; the grouped `just drift-*` recipes remain the operator-facing drift entrypoint.",
        "Baseline refresh: `just drift-refresh-baseline`.",
        "`just workflow-surface-check-drift` guards the grouped drift workflow map and reference split.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped drift surface `{needle}`"
        );
    }

    for forbidden in [
        "- `just drift-baseline` writes `infer_output_drift_baseline.json`.",
        "- `just drift-approve-baseline` writes `infer_output_drift_baseline_approval.json`.",
        "- `just drift-refresh-baseline` writes `infer_output_drift_baseline_refresh.json`.",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating drift variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Drift and rollback:",
        "`infer_output_drift_summary.json`",
        "`infer_output_drift_receipt.json`",
        "`infer_output_drift_baseline.json`",
        "`infer_output_drift_baseline_approval.json`",
        "`infer_output_drift_baseline_pointer.json`",
        "`infer_output_drift_baseline_history.json`",
        "`infer_output_drift_baseline_checkpoint.json`",
        "`infer_output_drift_baseline_bundle.json`",
        "`infer_output_drift_baseline_handoff.json`",
        "`infer_output_drift_baseline_transport_locator.json`",
        "`infer_output_drift_baseline_rollback.json`",
        "`infer_output_drift_baseline_supersession.json`",
        "`infer_output_drift_baseline_refresh.json`",
        "Low-level baseline writers stay behind `afterburner debug drift baseline ...`; use the grouped `just drift-*` recipes for operator-facing drift flows.",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-drift` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped drift surface `{needle}`"
        );
    }
}

#[test]
fn profiling_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "profile-infer:",
        "profile-environment-snapshot:",
        "profile-refresh-environment-snapshot current_snapshot profiler_path captured_at_unix_ms:",
        "profile-provenance-receipt snapshot captured_at_unix_ms:",
        "profile-provenance-bundle snapshot summary captured_at_unix_ms:",
        "workflow-surface-check-profiling:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Profiling stays grouped under these operator flows:",
        "Hotspot capture and infer path: `just profile-infer` delegates to `afterburner profile infer`.",
        "Environment capture: `just profile-environment-snapshot`, `just profile-refresh-environment-snapshot`.",
        "Provenance and evidence: `just profile-provenance-receipt`, `just profile-provenance-bundle`.",
        "On macOS, profiling requires `xcrun xctrace version` under full Xcode.",
        "The native profiling command forces `XCTRACE=/usr/bin/xctrace` while clearing `DEVELOPER_DIR` and `SDKROOT`.",
        "`just workflow-surface-check-profiling` guards the grouped profiling workflow map and reference split.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped profiling surface `{needle}`"
        );
    }

    for forbidden in [
        "- `just profile-environment-snapshot` writes `profiling_environment_snapshot.json`.",
        "- `just profile-refresh-environment-snapshot` writes `profiling_environment_snapshot_refresh.json`.",
        "- `just profile-provenance-receipt` writes `profiling_provenance_receipt.json`.",
        "- `just profile-provenance-bundle` writes `profiling_provenance_evidence_bundle.json`.",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating profiling variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Profiling:",
        "`infer_hotspot_summary.json`",
        "`profiling_environment_snapshot.json`",
        "`profiling_environment_snapshot_refresh.json`",
        "`profiling_provenance_receipt.json`",
        "`profiling_provenance_evidence_bundle.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-profiling` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped profiling surface `{needle}`"
        );
    }
}

#[test]
fn pretraining_source_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "pretraining-source-approval source source_revision approval_status approved_by approval_ticket approved_at_unix_ms:",
        "pretraining-source-provenance-receipt approval_receipt registry_entry_path upstream_locator reviewed_metadata_sha256:",
        "pretraining-source-provenance-evidence-bundle provenance_receipt:",
        "workflow-surface-check-pretraining-source:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "pretraining-source-approval-receipt source source_revision approval_status approved_by approval_ticket approved_at_unix_ms:",
        "pretraining-source-provenance action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep generic source bucket `{forbidden}` after collapse"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Pretraining source stays grouped under these operator flows:",
        "Approval flow: `just pretraining-source-approval`.",
        "Provenance flow: `just pretraining-source-provenance-receipt`, `just pretraining-source-provenance-evidence-bundle`.",
        "`just workflow-surface-check-pretraining-source` guards the grouped source workflow map and recipe surface.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped pretraining source surface `{needle}`"
        );
    }

    for forbidden in [
        "just pretraining-source-approval-receipt",
        "just pretraining-source-provenance <receipt|bundle>",
        "- `just pretraining-source-provenance-receipt` writes `pretraining_source_provenance_receipt.json`.",
        "- `just pretraining-source-provenance-evidence-bundle` writes `pretraining_source_provenance_evidence_bundle.json`.",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating source variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Pretraining source artifact groups:",
        "`pretraining_source_approval_receipt.json`",
        "`pretraining_source_provenance_receipt.json`",
        "`pretraining_source_provenance_evidence_bundle.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-pretraining-source` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped pretraining source surface `{needle}`"
        );
    }
}

#[test]
fn distributed_shard_lineage_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "distributed-shard-lineage-receipt metadata shard_id source source_revision checkpoint_group checkpoint_root checked_at_unix_ms:",
        "distributed-shard-lineage-bundle receipt:",
        "distributed-shard-lineage-bundle-reconcile receipt bundle:",
        "distributed-shard-lineage-bundle-history reconciliation event recorded_at_unix_ms:",
        "distributed-shard-lineage-handoff bundle:",
        "distributed-shard-lineage-handoff-reconcile bundle handoff:",
        "distributed-shard-lineage-handoff-history handoff event recorded_at_unix_ms:",
        "distributed-shard-lineage-handoff-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "distributed-shard-lineage-locator-pointer locator:",
        "distributed-shard-lineage-locator-history pointer event recorded_at_unix_ms:",
        "distributed-shard-lineage-transport-locator handoff:",
        "distributed-shard-lineage-transport-locator-reconcile handoff locator:",
        "distributed-shard-lineage-transport-locator-history reconciliation event recorded_at_unix_ms:",
        "workflow-surface-check-distributed-shard-lineage:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "distributed-shard-lineage-evidence-bundle receipt:",
        "distributed-shard-lineage-reconcile-evidence-bundle receipt bundle:",
        "distributed-shard-lineage-record-evidence-bundle-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "distributed-shard-lineage-reconcile-handoff bundle handoff:",
        "distributed-shard-lineage-record-handoff-history handoff event recorded_at_unix_ms:",
        "distributed-shard-lineage-record-handoff-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "distributed-shard-lineage-point-transport-locator handoff:",
        "distributed-shard-lineage-reconcile-transport-locator handoff locator:",
        "distributed-shard-lineage-record-transport-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "distributed-shard-lineage-point-locator locator:",
        "distributed-shard-lineage-record-locator-history pointer event recorded_at_unix_ms:",
        "distributed-shard-lineage-handoff-history action +args:",
        "distributed-shard-lineage-locator action +args:",
        "distributed-shard-lineage-locator-transport action +args:",
        "distributed-shard-lineage-bundle-manage action +args:",
        "distributed-shard-lineage-handoff-manage action +args:",
        "distributed-shard-lineage-locator-manage action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep per-variant lineage recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Distributed shard lineage stays grouped under these operator flows:",
        "Low-level lineage artifact writers stay behind `afterburner debug lineage ...`; the grouped `just distributed-shard-lineage-*` recipes remain the operator-facing entrypoint.",
        "Receipt flow: `just distributed-shard-lineage-receipt`.",
        "Bundle flow: `just distributed-shard-lineage-bundle`, `just distributed-shard-lineage-bundle-reconcile`, `just distributed-shard-lineage-bundle-history`.",
        "Handoff flow: `just distributed-shard-lineage-handoff`, `just distributed-shard-lineage-handoff-reconcile`, `just distributed-shard-lineage-handoff-history`, `just distributed-shard-lineage-handoff-reconciliation-history`.",
        "Locator flow: `just distributed-shard-lineage-locator-pointer`, `just distributed-shard-lineage-locator-history`, `just distributed-shard-lineage-transport-locator`, `just distributed-shard-lineage-transport-locator-reconcile`, `just distributed-shard-lineage-transport-locator-history`.",
        "`just workflow-surface-check-distributed-shard-lineage` guards the grouped lineage workflow map and recipe surface.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped distributed-shard-lineage surface `{needle}`"
        );
    }
    for forbidden in [
        "just distributed-shard-lineage-bundle-manage",
        "just distributed-shard-lineage-handoff-manage",
        "just distributed-shard-lineage-locator-manage",
        "- `just distributed-shard-lineage-bundle` writes `distributed_shard_lineage_evidence_bundle.json`.",
        "- `just distributed-shard-lineage-bundle-reconcile` writes `distributed_shard_lineage_evidence_bundle_reconciliation.json`.",
        "- `just distributed-shard-lineage-bundle-history` writes `distributed_shard_lineage_evidence_bundle_reconciliation_history.json`.",
        "- `just distributed-shard-lineage-handoff` writes `distributed_shard_lineage_evidence_handoff.json`.",
        "- `just distributed-shard-lineage-handoff-reconcile` writes `distributed_shard_lineage_evidence_handoff_reconciliation.json`.",
        "- `just distributed-shard-lineage-handoff-history` writes `distributed_shard_lineage_evidence_handoff_history.json`.",
        "- `just distributed-shard-lineage-handoff-reconciliation-history` writes `distributed_shard_lineage_evidence_handoff_reconciliation_history.json`.",
        "- `just distributed-shard-lineage-locator-pointer` writes `distributed_shard_lineage_locator_pointer.json`.",
        "- `just distributed-shard-lineage-locator-history` writes `distributed_shard_lineage_locator_history.json`.",
        "- `just distributed-shard-lineage-transport-locator` writes `distributed_shard_lineage_transport_locator.json`.",
        "- `just distributed-shard-lineage-transport-locator-reconcile` writes `distributed_shard_lineage_transport_locator_reconciliation.json`.",
        "- `just distributed-shard-lineage-transport-locator-history` writes `distributed_shard_lineage_transport_locator_reconciliation_history.json`.",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating lineage variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Distributed shard lineage artifact groups:",
        "`distributed_shard_lineage_receipt.json`",
        "`distributed_shard_lineage_evidence_bundle.json`",
        "`distributed_shard_lineage_evidence_bundle_reconciliation.json`",
        "`distributed_shard_lineage_evidence_bundle_reconciliation_history.json`",
        "`distributed_shard_lineage_evidence_handoff.json`",
        "`distributed_shard_lineage_evidence_handoff_reconciliation.json`",
        "`distributed_shard_lineage_evidence_handoff_history.json`",
        "`distributed_shard_lineage_evidence_handoff_reconciliation_history.json`",
        "`distributed_shard_lineage_transport_locator.json`",
        "`distributed_shard_lineage_transport_locator_reconciliation.json`",
        "`distributed_shard_lineage_transport_locator_reconciliation_history.json`",
        "`distributed_shard_lineage_locator_pointer.json`",
        "`distributed_shard_lineage_locator_history.json`",
        "Low-level lineage writers stay behind `afterburner debug lineage ...`; use the grouped `just distributed-shard-lineage-*` recipes for operator-facing flows.",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-distributed-shard-lineage` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped distributed-shard-lineage surface `{needle}`"
        );
    }
}

#[test]
fn scheduler_heartbeat_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "scheduler-heartbeat job_id lease_id worker_id state observed_at_unix_ms:",
        "workflow-surface-check-scheduler-heartbeat:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "scheduler-heartbeat-supersession action +args:",
        "scheduler-heartbeat-pointer action +args:",
        "scheduler-heartbeat-pointer-supersession action +args:",
        "scheduler-heartbeat-pointer-rollback action +args:",
        "scheduler-heartbeat-pointer-rollback-supersession action +args:",
        "scheduler-heartbeat-manage action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep generic scheduler-heartbeat bucket `{forbidden}` after expansion"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Scheduler heartbeat stays grouped under these operator flows:",
        "Primary heartbeat flow: `just scheduler-heartbeat`.",
        "Detailed heartbeat history, reconciliation, supersession, pointer, and rollback support artifacts stay behind `afterburner debug deploy scheduler-heartbeat ...`.",
        "`just workflow-surface-check-scheduler-heartbeat` keeps the grouped heartbeat workflow map and contract surface checked.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped scheduler-heartbeat surface `{needle}`"
        );
    }

    for forbidden in ["- `just scheduler-heartbeat` writes `gpu_scheduler_heartbeat.json`."] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating scheduler-heartbeat variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Scheduler heartbeat artifact groups:",
        "`gpu_scheduler_heartbeat*.json`",
        "`gpu_scheduler_heartbeat_pointer*.json`",
        "`gpu_scheduler_heartbeat_pointer_rollback*.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-scheduler-heartbeat` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped scheduler-heartbeat surface `{needle}`"
        );
    }

    for fixture in [
        "gpu_scheduler_heartbeat.schema.json",
        "gpu_scheduler_heartbeat.example.json",
        "gpu_scheduler_heartbeat_history.schema.json",
        "gpu_scheduler_heartbeat_history.example.json",
        "gpu_scheduler_heartbeat_reconciliation.schema.json",
        "gpu_scheduler_heartbeat_reconciliation.example.json",
        "gpu_scheduler_heartbeat_reconciliation_history.schema.json",
        "gpu_scheduler_heartbeat_reconciliation_history.example.json",
        "gpu_scheduler_heartbeat_supersession.schema.json",
        "gpu_scheduler_heartbeat_supersession.example.json",
        "gpu_scheduler_heartbeat_supersession_history.schema.json",
        "gpu_scheduler_heartbeat_supersession_history.example.json",
        "gpu_scheduler_heartbeat_supersession_reconciliation.schema.json",
        "gpu_scheduler_heartbeat_supersession_reconciliation.example.json",
        "gpu_scheduler_heartbeat_supersession_reconciliation_history.schema.json",
        "gpu_scheduler_heartbeat_supersession_reconciliation_history.example.json",
        "gpu_scheduler_heartbeat_pointer.schema.json",
        "gpu_scheduler_heartbeat_pointer.example.json",
        "gpu_scheduler_heartbeat_pointer_history.schema.json",
        "gpu_scheduler_heartbeat_pointer_history.example.json",
        "gpu_scheduler_heartbeat_pointer_reconciliation.schema.json",
        "gpu_scheduler_heartbeat_pointer_reconciliation.example.json",
        "gpu_scheduler_heartbeat_pointer_supersession.schema.json",
        "gpu_scheduler_heartbeat_pointer_supersession.example.json",
        "gpu_scheduler_heartbeat_pointer_supersession_history.schema.json",
        "gpu_scheduler_heartbeat_pointer_supersession_history.example.json",
        "gpu_scheduler_heartbeat_pointer_supersession_reconciliation.schema.json",
        "gpu_scheduler_heartbeat_pointer_supersession_reconciliation.example.json",
        "gpu_scheduler_heartbeat_pointer_supersession_reconciliation_history.schema.json",
        "gpu_scheduler_heartbeat_pointer_supersession_reconciliation_history.example.json",
        "gpu_scheduler_heartbeat_pointer_rollback.schema.json",
        "gpu_scheduler_heartbeat_pointer_rollback.example.json",
        "gpu_scheduler_heartbeat_pointer_rollback_history.schema.json",
        "gpu_scheduler_heartbeat_pointer_rollback_history.example.json",
        "gpu_scheduler_heartbeat_pointer_rollback_reconciliation.schema.json",
        "gpu_scheduler_heartbeat_pointer_rollback_reconciliation.example.json",
        "gpu_scheduler_heartbeat_pointer_rollback_reconciliation_history.schema.json",
        "gpu_scheduler_heartbeat_pointer_rollback_reconciliation_history.example.json",
        "gpu_scheduler_heartbeat_pointer_rollback_supersession.schema.json",
        "gpu_scheduler_heartbeat_pointer_rollback_supersession.example.json",
        "gpu_scheduler_heartbeat_pointer_rollback_supersession_history.schema.json",
        "gpu_scheduler_heartbeat_pointer_rollback_supersession_history.example.json",
        "gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation.schema.json",
        "gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation.example.json",
        "gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation_history.schema.json",
        "gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation_history.example.json",
    ] {
        assert!(fixture_exists(fixture), "missing fixture `{fixture}`");
    }
}
