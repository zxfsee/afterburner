use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

fn schema_required_set(schema: &Value) -> BTreeSet<String> {
    schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("schema.required entries must be strings")
                .to_string()
        })
        .collect()
}

fn schema_entry_required_set(schema: &Value) -> BTreeSet<String> {
    schema
        .get("properties")
        .and_then(|value| value.get("entries"))
        .and_then(|value| value.get("items"))
        .and_then(|value| value.get("required"))
        .and_then(Value::as_array)
        .expect("history schema entries.items.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("history schema required entries must be strings")
                .to_string()
        })
        .collect()
}

fn schema_payload_required_set(schema: &Value) -> BTreeSet<String> {
    schema
        .get("$defs")
        .and_then(|value| value.get("payload"))
        .and_then(|value| value.get("required"))
        .and_then(Value::as_array)
        .expect("schema payload required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("payload required entries must be strings")
                .to_string()
        })
        .collect()
}

#[test]
fn rollback_helper_rollout_keeps_variant_contracts_aligned() {
    let cases = [
        (
            "gpu_scheduler_heartbeat_pointer_rollback.schema.json",
            "src/cmd_scheduler_heartbeat_pointer_rollback.rs",
            "gpu_scheduler_heartbeat_pointer_rolled_back",
            Some(
                "scheduler-heartbeat-pointer-rollback current_pointer restored_pointer rolled_back_at_unix_ms:",
            ),
            schema_required_set as fn(&Value) -> BTreeSet<String>,
            [
                "schema_version",
                "current_pointer_path",
                "restored_pointer_path",
                "previous_heartbeat_path",
                "restored_heartbeat_path",
                "job_id",
                "lease_id",
                "worker_id",
                "previous_state",
                "restored_state",
                "previous_observed_at_unix_ms",
                "restored_observed_at_unix_ms",
                "rolled_back_at_unix_ms",
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>(),
        ),
        (
            "gpu_scheduler_heartbeat_pointer_rollback_history.schema.json",
            "src/cmd_scheduler_heartbeat_pointer_rollback_history.rs",
            "gpu_scheduler_heartbeat_pointer_rollback_history_written",
            Some(
                "scheduler-heartbeat-pointer-rollback current_pointer restored_pointer rolled_back_at_unix_ms:",
            ),
            schema_entry_required_set as fn(&Value) -> BTreeSet<String>,
            [
                "event",
                "rollback_path",
                "current_pointer_path",
                "restored_pointer_path",
                "previous_heartbeat_path",
                "restored_heartbeat_path",
                "job_id",
                "lease_id",
                "worker_id",
                "previous_state",
                "restored_state",
                "previous_observed_at_unix_ms",
                "restored_observed_at_unix_ms",
                "rolled_back_at_unix_ms",
                "recorded_at_unix_ms",
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>(),
        ),
        (
            "gpu_scheduler_heartbeat_pointer_rollback_reconciliation.schema.json",
            "src/cmd_scheduler_heartbeat_pointer_rollback_reconcile.rs",
            "gpu_scheduler_heartbeat_pointer_rollback_reconciliation_written",
            Some(
                "scheduler-heartbeat-pointer-rollback current_pointer restored_pointer rolled_back_at_unix_ms:",
            ),
            schema_payload_required_set as fn(&Value) -> BTreeSet<String>,
            [
                "current_pointer_path",
                "restored_pointer_path",
                "previous_heartbeat_path",
                "restored_heartbeat_path",
                "job_id",
                "lease_id",
                "worker_id",
                "previous_state",
                "restored_state",
                "previous_observed_at_unix_ms",
                "restored_observed_at_unix_ms",
                "rolled_back_at_unix_ms",
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>(),
        ),
        (
            "gpu_scheduler_heartbeat_pointer_rollback_reconciliation_history.schema.json",
            "src/cmd_scheduler_heartbeat_pointer_rollback_reconciliation_history.rs",
            "gpu_scheduler_heartbeat_pointer_rollback_reconciliation_history_written",
            Some(
                "scheduler-heartbeat-pointer-rollback current_pointer restored_pointer rolled_back_at_unix_ms:",
            ),
            schema_entry_required_set as fn(&Value) -> BTreeSet<String>,
            [
                "event",
                "reconciliation_path",
                "rollback_path",
                "current_rollback_path",
                "reconciliation_status",
                "desired",
                "current",
                "recorded_at_unix_ms",
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>(),
        ),
        (
            "gpu_scheduler_heartbeat_pointer_rollback_supersession.schema.json",
            "src/cmd_scheduler_heartbeat_pointer_rollback_supersession.rs",
            "gpu_scheduler_heartbeat_pointer_rollback_superseded",
            Some(
                "scheduler-heartbeat-pointer-rollback current_pointer restored_pointer rolled_back_at_unix_ms:",
            ),
            schema_required_set as fn(&Value) -> BTreeSet<String>,
            [
                "schema_version",
                "previous_rollback_path",
                "next_rollback_path",
                "previous_restored_pointer_path",
                "next_restored_pointer_path",
                "previous_restored_heartbeat_path",
                "next_restored_heartbeat_path",
                "job_id",
                "lease_id",
                "worker_id",
                "previous_restored_state",
                "next_restored_state",
                "previous_restored_observed_at_unix_ms",
                "next_restored_observed_at_unix_ms",
                "superseded_at_unix_ms",
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>(),
        ),
        (
            "gpu_scheduler_heartbeat_pointer_rollback_supersession_history.schema.json",
            "src/cmd_scheduler_heartbeat_pointer_rollback_supersession_history.rs",
            "gpu_scheduler_heartbeat_pointer_rollback_supersession_history_written",
            Some(
                "scheduler-heartbeat-pointer-rollback current_pointer restored_pointer rolled_back_at_unix_ms:",
            ),
            schema_entry_required_set as fn(&Value) -> BTreeSet<String>,
            [
                "event",
                "supersession_path",
                "previous_restored_pointer_path",
                "next_restored_pointer_path",
                "previous_restored_heartbeat_path",
                "next_restored_heartbeat_path",
                "job_id",
                "lease_id",
                "worker_id",
                "previous_restored_state",
                "next_restored_state",
                "previous_restored_observed_at_unix_ms",
                "next_restored_observed_at_unix_ms",
                "superseded_at_unix_ms",
                "recorded_at_unix_ms",
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>(),
        ),
        (
            "gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation.schema.json",
            "src/cmd_scheduler_heartbeat_pointer_rollback_supersession_reconcile.rs",
            "gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation_written",
            Some(
                "scheduler-heartbeat-pointer-rollback current_pointer restored_pointer rolled_back_at_unix_ms:",
            ),
            schema_payload_required_set as fn(&Value) -> BTreeSet<String>,
            [
                "previous_rollback_path",
                "next_rollback_path",
                "previous_restored_pointer_path",
                "next_restored_pointer_path",
                "previous_restored_heartbeat_path",
                "next_restored_heartbeat_path",
                "job_id",
                "lease_id",
                "worker_id",
                "previous_restored_state",
                "next_restored_state",
                "previous_restored_observed_at_unix_ms",
                "next_restored_observed_at_unix_ms",
                "superseded_at_unix_ms",
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>(),
        ),
        (
            "gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation_history.schema.json",
            "src/cmd_scheduler_heartbeat_pointer_rollback_supersession_reconciliation_history.rs",
            "gpu_scheduler_heartbeat_pointer_rollback_supersession_reconciliation_history_written",
            Some(
                "scheduler-heartbeat-pointer-rollback current_pointer restored_pointer rolled_back_at_unix_ms:",
            ),
            schema_entry_required_set as fn(&Value) -> BTreeSet<String>,
            [
                "event",
                "reconciliation_path",
                "supersession_path",
                "current_supersession_path",
                "reconciliation_status",
                "desired",
                "current",
                "recorded_at_unix_ms",
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>(),
        ),
    ];

    let justfile = repo_file("justfile");
    for (schema_name, source_path, event_name, workflow_snippet, extractor, expected_fields) in
        cases
    {
        let schema_text = fs::read_to_string(fixture_path(schema_name))
            .unwrap_or_else(|err| panic!("read {schema_name}: {err}"));
        let schema: Value = serde_json::from_str(&schema_text)
            .unwrap_or_else(|err| panic!("parse {schema_name}: {err}"));
        let source = repo_file(source_path);

        assert_eq!(
            extractor(&schema),
            expected_fields,
            "{schema_name} required fields drifted after helper rollout"
        );
        assert!(
            source.contains(event_name),
            "{source_path} must emit `{event_name}`"
        );
        if let Some(workflow_snippet) = workflow_snippet {
            assert!(
                justfile.contains(workflow_snippet),
                "justfile must expose `{workflow_snippet}`"
            );
        }
    }
}
