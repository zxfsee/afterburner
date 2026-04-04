use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

fn fixture_exists(path: &str) -> bool {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(path)
        .exists()
}

#[test]
fn scheduler_heartbeat_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "scheduler-heartbeat job_id lease_id worker_id state observed_at_unix_ms:",
        "scheduler-heartbeat-point heartbeat:",
        "scheduler-heartbeat-history heartbeat event recorded_at_unix_ms:",
        "scheduler-heartbeat-reconcile heartbeat current_heartbeat:",
        "scheduler-heartbeat-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "scheduler-heartbeat-supersede previous_heartbeat next_heartbeat superseded_at_unix_ms:",
        "scheduler-heartbeat-supersession action +args:",
        "scheduler-heartbeat-pointer action +args:",
        "scheduler-heartbeat-pointer-supersession action +args:",
        "scheduler-heartbeat-pointer-rollback action +args:",
        "scheduler-heartbeat-pointer-rollback-supersession action +args:",
        "workflow-surface-check-scheduler-heartbeat:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "scheduler-heartbeat-record-history heartbeat event recorded_at_unix_ms:",
        "scheduler-heartbeat-record-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "scheduler-heartbeat-supersession-reconcile supersession current_supersession:",
        "scheduler-heartbeat-record-supersession-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "scheduler-heartbeat-record-supersession-history supersession event recorded_at_unix_ms:",
        "scheduler-heartbeat-point-record-history pointer event recorded_at_unix_ms:",
        "scheduler-heartbeat-point-reconcile pointer current_pointer:",
        "scheduler-heartbeat-point-supersede previous_pointer next_pointer superseded_at_unix_ms:",
        "scheduler-heartbeat-point-record-supersession-history supersession event recorded_at_unix_ms:",
        "scheduler-heartbeat-point-supersession-reconcile supersession current_supersession:",
        "scheduler-heartbeat-point-record-supersession-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "scheduler-heartbeat-point-rollback current_pointer restored_pointer rolled_back_at_unix_ms:",
        "scheduler-heartbeat-point-record-rollback-history rollback event recorded_at_unix_ms:",
        "scheduler-heartbeat-point-rollback-reconcile rollback current_rollback:",
        "scheduler-heartbeat-point-record-rollback-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "scheduler-heartbeat-point-rollback-supersede previous_rollback next_rollback superseded_at_unix_ms:",
        "scheduler-heartbeat-point-record-rollback-supersession-history supersession event recorded_at_unix_ms:",
        "scheduler-heartbeat-point-rollback-supersession-reconcile supersession current_supersession:",
        "scheduler-heartbeat-point-record-rollback-supersession-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "scheduler-heartbeat-manage action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep per-variant heartbeat recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Scheduler heartbeat stays grouped under these workflow families:",
        "Primary heartbeat flow: `just scheduler-heartbeat`, `just scheduler-heartbeat-point`.",
        "Heartbeat support flows: `just scheduler-heartbeat-history`, `just scheduler-heartbeat-reconcile`, `just scheduler-heartbeat-reconciliation-history`, `just scheduler-heartbeat-supersede`, `just scheduler-heartbeat-supersession <history|reconcile|reconciliation-history>`.",
        "Pointer support flows: `just scheduler-heartbeat-pointer <history|reconcile|supersede|rollback>`, `just scheduler-heartbeat-pointer-supersession <history|reconcile|reconciliation-history>`, `just scheduler-heartbeat-pointer-rollback <history|reconcile|reconciliation-history|supersede>`, `just scheduler-heartbeat-pointer-rollback-supersession <history|reconcile|reconciliation-history>`.",
        "`just workflow-surface-check-scheduler-heartbeat` keeps the exhaustive CLI, fixture, and grouped recipe surface checked.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped scheduler-heartbeat surface `{needle}`"
        );
    }

    for forbidden in [
        "- `just scheduler-heartbeat` writes `gpu_scheduler_heartbeat.json`.",
        "- `just scheduler-heartbeat-point` writes `gpu_scheduler_heartbeat_pointer.json`.",
        "- `just scheduler-heartbeat-history` writes `gpu_scheduler_heartbeat_history.json`.",
        "- `just scheduler-heartbeat-reconcile` writes `gpu_scheduler_heartbeat_reconciliation.json`.",
        "- `just scheduler-heartbeat-reconciliation-history` writes `gpu_scheduler_heartbeat_reconciliation_history.json`.",
        "- `just scheduler-heartbeat-supersede` writes `gpu_scheduler_heartbeat_supersession.json`.",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating scheduler-heartbeat variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Scheduler heartbeat family groups:",
        "`gpu_scheduler_heartbeat*.json`",
        "`gpu_scheduler_heartbeat_pointer*.json`",
        "`gpu_scheduler_heartbeat_pointer_rollback*.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped recipe map and `workflow-surface-check-scheduler-heartbeat` for mechanical coverage.",
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
