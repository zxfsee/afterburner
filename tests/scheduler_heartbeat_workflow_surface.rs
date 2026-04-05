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
