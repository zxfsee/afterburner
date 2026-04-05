use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

#[test]
fn drift_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "drift-receipt candidate_summary current_summary policy:",
        "drift-baseline summary receipt:",
        "drift-approve-baseline baseline approved_by approval_ticket approved_at_unix_ms:",
        "drift-approved-baseline-pointer approval:",
        "drift-approved-baseline-history pointer event recorded_at_unix_ms:",
        "drift-checkpoint-baseline pointer history:",
        "drift-baseline-bundle pointer history:",
        "drift-baseline-handoff bundle:",
        "drift-baseline-transport-locator handoff:",
        "drift-approved-baseline-rollback current_pointer restored_approval rolled_back_at_unix_ms:",
        "drift-approved-baseline-supersede previous_approval next_approval superseded_at_unix_ms:",
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
        "State and checkpointing: `just drift-approved-baseline-pointer`, `just drift-approved-baseline-history`, `just drift-checkpoint-baseline`.",
        "Exports and transport: `just drift-baseline-bundle`, `just drift-baseline-handoff`, `just drift-baseline-transport-locator`.",
        "Baseline changes: `just drift-approved-baseline-rollback`, `just drift-approved-baseline-supersede`, `just drift-refresh-baseline`.",
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
        "- `just drift-approved-baseline-pointer` writes `infer_output_drift_baseline_pointer.json`.",
        "- `just drift-approved-baseline-history` writes `infer_output_drift_baseline_history.json`.",
        "- `just drift-checkpoint-baseline` writes `infer_output_drift_baseline_checkpoint.json`.",
        "- `just drift-baseline-bundle` writes `infer_output_drift_baseline_bundle.json`.",
        "- `just drift-baseline-handoff` writes `infer_output_drift_baseline_handoff.json`.",
        "- `just drift-baseline-transport-locator` writes `infer_output_drift_baseline_transport_locator.json`.",
        "- `just drift-approved-baseline-rollback` writes `infer_output_drift_baseline_rollback.json`.",
        "- `just drift-approved-baseline-supersede` writes `infer_output_drift_baseline_supersession.json`.",
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
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-drift` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped drift surface `{needle}`"
        );
    }
}
