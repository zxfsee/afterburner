use std::path::Path;

use crate::command_reconciliation::{ReconciliationError, read_string, read_u64};
use serde_json::{Map, Value, json};

const ROLLBACK_KIND: &str = "gpu scheduler heartbeat pointer rollback";
const ROLLBACK_SUPERSESSION_KIND: &str = "gpu scheduler heartbeat pointer rollback supersession";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerHeartbeatPointerRollbackRecord {
    pub current_pointer_path: String,
    pub restored_pointer_path: String,
    pub previous_heartbeat_path: String,
    pub restored_heartbeat_path: String,
    pub job_id: String,
    pub lease_id: String,
    pub worker_id: String,
    pub previous_state: String,
    pub restored_state: String,
    pub previous_observed_at_unix_ms: u64,
    pub restored_observed_at_unix_ms: u64,
    pub previous_progress_marker: Option<String>,
    pub restored_progress_marker: Option<String>,
    pub rolled_back_at_unix_ms: u64,
}

impl SchedulerHeartbeatPointerRollbackRecord {
    pub fn from_object(
        object: &Map<String, Value>,
        path: &Path,
    ) -> Result<Self, ReconciliationError> {
        Ok(Self {
            current_pointer_path: read_string(object, "current_pointer_path", path, ROLLBACK_KIND)?,
            restored_pointer_path: read_string(
                object,
                "restored_pointer_path",
                path,
                ROLLBACK_KIND,
            )?,
            previous_heartbeat_path: read_string(
                object,
                "previous_heartbeat_path",
                path,
                ROLLBACK_KIND,
            )?,
            restored_heartbeat_path: read_string(
                object,
                "restored_heartbeat_path",
                path,
                ROLLBACK_KIND,
            )?,
            job_id: read_string(object, "job_id", path, ROLLBACK_KIND)?,
            lease_id: read_string(object, "lease_id", path, ROLLBACK_KIND)?,
            worker_id: read_string(object, "worker_id", path, ROLLBACK_KIND)?,
            previous_state: read_string(object, "previous_state", path, ROLLBACK_KIND)?,
            restored_state: read_string(object, "restored_state", path, ROLLBACK_KIND)?,
            previous_observed_at_unix_ms: read_u64(
                object,
                "previous_observed_at_unix_ms",
                path,
                ROLLBACK_KIND,
            )?,
            restored_observed_at_unix_ms: read_u64(
                object,
                "restored_observed_at_unix_ms",
                path,
                ROLLBACK_KIND,
            )?,
            previous_progress_marker: read_optional_string(object, "previous_progress_marker"),
            restored_progress_marker: read_optional_string(object, "restored_progress_marker"),
            rolled_back_at_unix_ms: read_u64(
                object,
                "rolled_back_at_unix_ms",
                path,
                ROLLBACK_KIND,
            )?,
        })
    }

    pub fn to_payload(&self) -> Value {
        let mut payload = json!({
            "current_pointer_path": self.current_pointer_path,
            "restored_pointer_path": self.restored_pointer_path,
            "previous_heartbeat_path": self.previous_heartbeat_path,
            "restored_heartbeat_path": self.restored_heartbeat_path,
            "job_id": self.job_id,
            "lease_id": self.lease_id,
            "worker_id": self.worker_id,
            "previous_state": self.previous_state,
            "restored_state": self.restored_state,
            "previous_observed_at_unix_ms": self.previous_observed_at_unix_ms,
            "restored_observed_at_unix_ms": self.restored_observed_at_unix_ms,
            "rolled_back_at_unix_ms": self.rolled_back_at_unix_ms,
        });
        if let Some(previous_progress_marker) = &self.previous_progress_marker {
            payload
                .as_object_mut()
                .expect("rollback payload must be object")
                .insert(
                    "previous_progress_marker".to_string(),
                    previous_progress_marker.clone().into(),
                );
        }
        if let Some(restored_progress_marker) = &self.restored_progress_marker {
            payload
                .as_object_mut()
                .expect("rollback payload must be object")
                .insert(
                    "restored_progress_marker".to_string(),
                    restored_progress_marker.clone().into(),
                );
        }
        payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerHeartbeatPointerRollbackSupersessionRecord {
    pub previous_rollback_path: String,
    pub next_rollback_path: String,
    pub previous_restored_pointer_path: String,
    pub next_restored_pointer_path: String,
    pub previous_restored_heartbeat_path: String,
    pub next_restored_heartbeat_path: String,
    pub job_id: String,
    pub lease_id: String,
    pub worker_id: String,
    pub previous_restored_state: String,
    pub next_restored_state: String,
    pub previous_restored_observed_at_unix_ms: u64,
    pub next_restored_observed_at_unix_ms: u64,
    pub previous_restored_progress_marker: Option<String>,
    pub next_restored_progress_marker: Option<String>,
    pub superseded_at_unix_ms: u64,
}

impl SchedulerHeartbeatPointerRollbackSupersessionRecord {
    pub fn from_object(
        object: &Map<String, Value>,
        path: &Path,
    ) -> Result<Self, ReconciliationError> {
        Ok(Self {
            previous_rollback_path: read_string(
                object,
                "previous_rollback_path",
                path,
                ROLLBACK_SUPERSESSION_KIND,
            )?,
            next_rollback_path: read_string(
                object,
                "next_rollback_path",
                path,
                ROLLBACK_SUPERSESSION_KIND,
            )?,
            previous_restored_pointer_path: read_string(
                object,
                "previous_restored_pointer_path",
                path,
                ROLLBACK_SUPERSESSION_KIND,
            )?,
            next_restored_pointer_path: read_string(
                object,
                "next_restored_pointer_path",
                path,
                ROLLBACK_SUPERSESSION_KIND,
            )?,
            previous_restored_heartbeat_path: read_string(
                object,
                "previous_restored_heartbeat_path",
                path,
                ROLLBACK_SUPERSESSION_KIND,
            )?,
            next_restored_heartbeat_path: read_string(
                object,
                "next_restored_heartbeat_path",
                path,
                ROLLBACK_SUPERSESSION_KIND,
            )?,
            job_id: read_string(object, "job_id", path, ROLLBACK_SUPERSESSION_KIND)?,
            lease_id: read_string(object, "lease_id", path, ROLLBACK_SUPERSESSION_KIND)?,
            worker_id: read_string(object, "worker_id", path, ROLLBACK_SUPERSESSION_KIND)?,
            previous_restored_state: read_string(
                object,
                "previous_restored_state",
                path,
                ROLLBACK_SUPERSESSION_KIND,
            )?,
            next_restored_state: read_string(
                object,
                "next_restored_state",
                path,
                ROLLBACK_SUPERSESSION_KIND,
            )?,
            previous_restored_observed_at_unix_ms: read_u64(
                object,
                "previous_restored_observed_at_unix_ms",
                path,
                ROLLBACK_SUPERSESSION_KIND,
            )?,
            next_restored_observed_at_unix_ms: read_u64(
                object,
                "next_restored_observed_at_unix_ms",
                path,
                ROLLBACK_SUPERSESSION_KIND,
            )?,
            previous_restored_progress_marker: read_optional_string(
                object,
                "previous_restored_progress_marker",
            ),
            next_restored_progress_marker: read_optional_string(
                object,
                "next_restored_progress_marker",
            ),
            superseded_at_unix_ms: read_u64(
                object,
                "superseded_at_unix_ms",
                path,
                ROLLBACK_SUPERSESSION_KIND,
            )?,
        })
    }

    pub fn to_payload(&self) -> Value {
        let mut payload = json!({
            "previous_rollback_path": self.previous_rollback_path,
            "next_rollback_path": self.next_rollback_path,
            "previous_restored_pointer_path": self.previous_restored_pointer_path,
            "next_restored_pointer_path": self.next_restored_pointer_path,
            "previous_restored_heartbeat_path": self.previous_restored_heartbeat_path,
            "next_restored_heartbeat_path": self.next_restored_heartbeat_path,
            "job_id": self.job_id,
            "lease_id": self.lease_id,
            "worker_id": self.worker_id,
            "previous_restored_state": self.previous_restored_state,
            "next_restored_state": self.next_restored_state,
            "previous_restored_observed_at_unix_ms": self.previous_restored_observed_at_unix_ms,
            "next_restored_observed_at_unix_ms": self.next_restored_observed_at_unix_ms,
            "superseded_at_unix_ms": self.superseded_at_unix_ms,
        });
        if let Some(previous_restored_progress_marker) = &self.previous_restored_progress_marker {
            payload
                .as_object_mut()
                .expect("supersession payload must be object")
                .insert(
                    "previous_restored_progress_marker".to_string(),
                    previous_restored_progress_marker.clone().into(),
                );
        }
        if let Some(next_restored_progress_marker) = &self.next_restored_progress_marker {
            payload
                .as_object_mut()
                .expect("supersession payload must be object")
                .insert(
                    "next_restored_progress_marker".to_string(),
                    next_restored_progress_marker.clone().into(),
                );
        }
        payload
    }
}

fn read_optional_string(object: &Map<String, Value>, key: &'static str) -> Option<String> {
    object.get(key).and_then(Value::as_str).map(str::to_string)
}
