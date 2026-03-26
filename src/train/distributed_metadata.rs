use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::observability::event_line;

const DISTRIBUTED_SHARD_METADATA_PATH_ENV: &str = "AFTERBURNER_DISTRIBUTED_SHARD_METADATA_PATH";

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
