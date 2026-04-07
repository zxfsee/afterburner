use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use crate::observability::event_line;

const DISTRIBUTED_SHARD_METADATA_PATH_ENV: &str = "AFTERBURNER_DISTRIBUTED_SHARD_METADATA_PATH";
pub const DISTRIBUTED_RUNTIME_EXECUTION_FILENAME: &str = "distributed_runtime_execution.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistributedShardMetadata {
    pub shard_index: u64,
    pub shard_count: u64,
    pub owner_worker_id: String,
    pub owner_worker_rank: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistributedShardMetadataLoadError {
    kind: &'static str,
    metadata_path: PathBuf,
    detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistributedRuntimeParticipant {
    pub global_rank: u64,
    pub local_rank: u64,
    pub node_rank: u64,
    pub device_group: String,
    pub device_ref: String,
}

impl DistributedShardMetadataLoadError {
    pub fn kind(&self) -> &'static str {
        self.kind
    }

    pub fn metadata_path(&self) -> &Path {
        &self.metadata_path
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

pub fn validate_distributed_shard_metadata(
    metadata: &[DistributedShardMetadata],
) -> Result<(), String> {
    let mut seen_owners = BTreeSet::new();

    for (fixture_index, shard) in metadata.iter().enumerate() {
        if shard.shard_index >= shard.shard_count {
            return Err(format!(
                "metadata[{fixture_index}] has shard_index {} not less than shard_count {}",
                shard.shard_index, shard.shard_count
            ));
        }

        if !seen_owners.insert((shard.owner_worker_id.clone(), shard.owner_worker_rank)) {
            return Err(format!(
                "metadata[{fixture_index}] duplicates owner_worker_id '{}' and owner_worker_rank {}",
                shard.owner_worker_id, shard.owner_worker_rank
            ));
        }
    }

    Ok(())
}

pub fn validate_distributed_shard_metadata_load_path(
    metadata_path: &Path,
) -> Result<(), DistributedShardMetadataLoadError> {
    let contents = std::fs::read_to_string(metadata_path).map_err(|err| {
        distributed_shard_metadata_load_error(
            "read_error",
            metadata_path,
            format!("failed to read metadata: {err}"),
        )
    })?;

    let parsed: serde_json::Value = serde_json::from_str(contents.as_str()).map_err(|err| {
        distributed_shard_metadata_load_error(
            "parse_error",
            metadata_path,
            format!("failed to parse metadata json: {err}"),
        )
    })?;

    let entries = parsed.as_array().ok_or_else(|| {
        distributed_shard_metadata_load_error(
            "schema_error",
            metadata_path,
            "metadata must be a JSON array".to_string(),
        )
    })?;

    let mut metadata = Vec::with_capacity(entries.len());
    for (entry_index, entry) in entries.iter().enumerate() {
        let object = entry.as_object().ok_or_else(|| {
            distributed_shard_metadata_load_error(
                "schema_error",
                metadata_path,
                format!("metadata[{entry_index}] must be a JSON object"),
            )
        })?;

        metadata.push(DistributedShardMetadata {
            shard_index: parse_u64_field(object, "shard_index", entry_index, metadata_path)?,
            shard_count: parse_u64_field(object, "shard_count", entry_index, metadata_path)?,
            owner_worker_id: parse_string_field(
                object,
                "owner_worker_id",
                entry_index,
                metadata_path,
            )?,
            owner_worker_rank: parse_u64_field(
                object,
                "owner_worker_rank",
                entry_index,
                metadata_path,
            )?,
        });
    }

    validate_distributed_shard_metadata(metadata.as_slice()).map_err(|detail| {
        distributed_shard_metadata_load_error("validation_error", metadata_path, detail)
    })
}

pub fn distributed_shard_metadata_invalid_event_line(
    backend: &str,
    artifact_version: &str,
    err: &DistributedShardMetadataLoadError,
) -> String {
    event_line(
        "error",
        "train",
        "distributed_shard_metadata_invalid",
        serde_json::json!({
            "backend": backend,
            "artifact_version": artifact_version,
            "kind": err.kind(),
            "metadata_path": err.metadata_path().to_string_lossy().to_string(),
            "detail": err.detail(),
        }),
    )
}

pub fn distributed_shard_metadata_path_from_env() -> Option<PathBuf> {
    std::env::var(DISTRIBUTED_SHARD_METADATA_PATH_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

pub fn single_node_data_parallel_participants(
    world_size: u64,
    device_group: &str,
    participant_devices: &[String],
) -> Vec<DistributedRuntimeParticipant> {
    (0..world_size)
        .map(|global_rank| DistributedRuntimeParticipant {
            global_rank,
            local_rank: global_rank,
            node_rank: 0,
            device_group: device_group.to_string(),
            device_ref: participant_devices
                .get(global_rank as usize)
                .cloned()
                .unwrap_or_default(),
        })
        .collect()
}

pub fn distributed_runtime_execution_value(
    backend: &str,
    artifact_version: &str,
    world_size: u64,
    device_group: &str,
    participant_devices: Vec<String>,
    batch_size: usize,
    num_epochs: usize,
    train_items_total: usize,
    valid_items_total: usize,
) -> Result<serde_json::Value, String> {
    if participant_devices.len() != usize::try_from(world_size).unwrap_or(usize::MAX) {
        return Err(format!(
            "distributed runtime execution artifact requires {} participant devices, got {}",
            world_size,
            participant_devices.len()
        ));
    }

    let participants =
        single_node_data_parallel_participants(world_size, device_group, &participant_devices);
    Ok(serde_json::json!({
        "schema_version": "1",
        "artifact_version": artifact_version,
        "backend": backend,
        "runtime_mode": "single_node_dp",
        "node_count": 1,
        "world_size": world_size,
        "device_group": device_group,
        "participant_count": participants.len(),
        "participant_devices": participant_devices,
        "participants": participants
            .iter()
            .map(|participant| {
                serde_json::json!({
                    "global_rank": participant.global_rank,
                    "local_rank": participant.local_rank,
                    "node_rank": participant.node_rank,
                    "device_group": participant.device_group,
                    "device_ref": participant.device_ref,
                })
            })
            .collect::<Vec<_>>(),
        "batch_size": batch_size,
        "num_epochs": num_epochs,
        "train_items_total": train_items_total,
        "valid_items_total": valid_items_total,
        "optimizer_strategy": "main_device",
    }))
}

pub fn write_distributed_runtime_execution_artifact(
    train_dir: &Path,
    artifact: &serde_json::Value,
) -> io::Result<PathBuf> {
    let path = train_dir.join(DISTRIBUTED_RUNTIME_EXECUTION_FILENAME);
    let text = serde_json::to_string_pretty(artifact)
        .map_err(|err| io::Error::other(format!("serialize runtime execution artifact: {err}")))?;
    std::fs::write(&path, text)?;
    Ok(path)
}

pub fn distributed_runtime_execution_written_event_line(
    artifact_path: &Path,
    artifact: &serde_json::Value,
) -> String {
    event_line(
        "info",
        "train",
        "distributed_runtime_execution_written",
        serde_json::json!({
            "artifact_path": artifact_path.to_string_lossy().to_string(),
            "artifact": artifact,
        }),
    )
}

fn distributed_shard_metadata_load_error(
    kind: &'static str,
    metadata_path: &Path,
    detail: String,
) -> DistributedShardMetadataLoadError {
    DistributedShardMetadataLoadError {
        kind,
        metadata_path: metadata_path.to_path_buf(),
        detail,
    }
}

fn parse_u64_field(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    entry_index: usize,
    metadata_path: &Path,
) -> Result<u64, DistributedShardMetadataLoadError> {
    object
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            distributed_shard_metadata_load_error(
                "schema_error",
                metadata_path,
                format!("metadata[{entry_index}].{field} must be a non-negative integer"),
            )
        })
}

fn parse_string_field(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    entry_index: usize,
    metadata_path: &Path,
) -> Result<String, DistributedShardMetadataLoadError> {
    let value = object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            distributed_shard_metadata_load_error(
                "schema_error",
                metadata_path,
                format!("metadata[{entry_index}].{field} must be a string"),
            )
        })?
        .to_string();
    if value.is_empty() {
        return Err(distributed_shard_metadata_load_error(
            "schema_error",
            metadata_path,
            format!("metadata[{entry_index}].{field} must not be empty"),
        ));
    }
    Ok(value)
}
