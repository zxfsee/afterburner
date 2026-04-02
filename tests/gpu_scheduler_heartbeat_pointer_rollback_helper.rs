use std::path::Path;

use afterburner::scheduler_heartbeat_pointer_rollback_helpers::{
    SchedulerHeartbeatPointerRollbackRecord, SchedulerHeartbeatPointerRollbackSupersessionRecord,
};
use serde_json::{Value, json};

#[test]
fn rollback_helper_reads_payload_with_optional_markers() {
    let rollback = json!({
      "schema_version": "1",
      "current_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.json",
      "restored_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.previous.json",
      "previous_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.json",
      "restored_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.previous.json",
      "job_id": "job-0001",
      "lease_id": "lease-job-0001",
      "worker_id": "worker-0",
      "previous_state": "running",
      "restored_state": "checkpointed",
      "previous_observed_at_unix_ms": 1735689960000_u64,
      "restored_observed_at_unix_ms": 1735689900000_u64,
      "previous_progress_marker": "checkpoint-group-0004/step-256",
      "restored_progress_marker": "checkpoint-group-0003/step-128",
      "rolled_back_at_unix_ms": 1735690100000_u64
    });

    let record = SchedulerHeartbeatPointerRollbackRecord::from_object(
        rollback.as_object().expect("rollback object"),
        Path::new("artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback.json"),
    )
    .expect("read rollback record");

    assert_eq!(
        record.to_payload(),
        json!({
          "current_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.json",
          "restored_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.previous.json",
          "previous_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.json",
          "restored_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.previous.json",
          "job_id": "job-0001",
          "lease_id": "lease-job-0001",
          "worker_id": "worker-0",
          "previous_state": "running",
          "restored_state": "checkpointed",
          "previous_observed_at_unix_ms": 1735689960000_u64,
          "restored_observed_at_unix_ms": 1735689900000_u64,
          "previous_progress_marker": "checkpoint-group-0004/step-256",
          "restored_progress_marker": "checkpoint-group-0003/step-128",
          "rolled_back_at_unix_ms": 1735690100000_u64
        })
    );
}

#[test]
fn rollback_supersession_helper_reads_payload_with_optional_markers() {
    let supersession = json!({
      "schema_version": "1",
      "previous_rollback_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback.json",
      "next_rollback_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback.next.json",
      "previous_restored_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.previous.json",
      "next_restored_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.older.json",
      "previous_restored_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.previous.json",
      "next_restored_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.older.json",
      "job_id": "job-0001",
      "lease_id": "lease-job-0001",
      "worker_id": "worker-0",
      "previous_restored_state": "checkpointed",
      "next_restored_state": "rolled-back",
      "previous_restored_observed_at_unix_ms": 1735689900000_u64,
      "next_restored_observed_at_unix_ms": 1735689840000_u64,
      "previous_restored_progress_marker": "checkpoint-group-0003/step-128",
      "next_restored_progress_marker": "checkpoint-group-0002/step-64",
      "superseded_at_unix_ms": 1735690200000_u64
    });

    let record = SchedulerHeartbeatPointerRollbackSupersessionRecord::from_object(
        supersession.as_object().expect("supersession object"),
        Path::new("artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback_supersession.json"),
    )
    .expect("read rollback supersession record");

    let payload = record.to_payload();
    assert_eq!(
        payload,
        json!({
          "previous_rollback_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback.json",
          "next_rollback_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback.next.json",
          "previous_restored_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.previous.json",
          "next_restored_pointer_path": "artifacts/deploy/gpu_scheduler_heartbeat_pointer.older.json",
          "previous_restored_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.previous.json",
          "next_restored_heartbeat_path": "artifacts/deploy/gpu_scheduler_heartbeat.older.json",
          "job_id": "job-0001",
          "lease_id": "lease-job-0001",
          "worker_id": "worker-0",
          "previous_restored_state": "checkpointed",
          "next_restored_state": "rolled-back",
          "previous_restored_observed_at_unix_ms": 1735689900000_u64,
          "next_restored_observed_at_unix_ms": 1735689840000_u64,
          "previous_restored_progress_marker": "checkpoint-group-0003/step-128",
          "next_restored_progress_marker": "checkpoint-group-0002/step-64",
          "superseded_at_unix_ms": 1735690200000_u64
        })
    );

    let current = {
        let mut current = payload.clone();
        current
            .as_object_mut()
            .expect("payload object")
            .insert(
                "current_supersession_path".to_string(),
                Value::from("artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback_supersession.current.json"),
            );
        current
    };
    assert_eq!(
        current["current_supersession_path"],
        Value::from(
            "artifacts/deploy/gpu_scheduler_heartbeat_pointer_rollback_supersession.current.json"
        )
    );
}
