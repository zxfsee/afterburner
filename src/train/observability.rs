use std::io;
use std::path::{Path, PathBuf};

use crate::observability::{append_json_line, event_line};

use super::{
    TrainingConfig,
    artifacts::ExportedCalibrationSidecar,
    distributed_metadata::{
        DistributedShardMetadataLoadError, distributed_runtime_execution_written_event_line,
    },
};

fn observability_path(train_dir: &Path) -> PathBuf {
    train_dir.join("observability.jsonl")
}

pub fn write_train_event(
    train_dir: &Path,
    event: &str,
    backend: &str,
    config: &TrainingConfig,
    metrics_dir: &Path,
    inference_dir: &Path,
    planned_samples: u64,
) -> io::Result<()> {
    let path = observability_path(train_dir);
    let line = train_start_event_line(
        backend,
        event,
        config,
        metrics_dir,
        inference_dir,
        planned_samples,
    );
    append_json_line(&path, &line)
}

pub fn train_start_event_line(
    backend: &str,
    event: &str,
    config: &TrainingConfig,
    metrics_dir: &Path,
    inference_dir: &Path,
    planned_samples: u64,
) -> String {
    event_line(
        "info",
        "train",
        event,
        serde_json::json!({
            "backend": backend,
            "artifact_version": config.artifact_version,
            "batch_size": config.batch_size,
            "worker_parallelism": config.num_workers,
            "num_epochs": config.num_epochs,
            "planned_samples": planned_samples,
            "metrics_dir": metrics_dir.to_string_lossy().to_string(),
            "inference_dir": inference_dir.to_string_lossy().to_string()
        }),
    )
}

pub fn write_train_export_event(
    train_dir: &Path,
    backend: &str,
    artifact_version: &str,
    artifact_path: &Path,
    manifest_path: &Path,
    current_path: &Path,
    calibration: Option<&ExportedCalibrationSidecar>,
) -> io::Result<()> {
    let path = observability_path(train_dir);
    let line = artifact_exported_event_line(
        backend,
        artifact_version,
        artifact_path,
        manifest_path,
        current_path,
        calibration,
    );
    append_json_line(&path, &line)
}

pub fn artifact_exported_event_line(
    backend: &str,
    artifact_version: &str,
    artifact_path: &Path,
    manifest_path: &Path,
    current_path: &Path,
    calibration: Option<&ExportedCalibrationSidecar>,
) -> String {
    let mut fields = serde_json::json!({
        "backend": backend,
        "artifact_version": artifact_version,
        "artifact_path": artifact_path.to_string_lossy().to_string(),
        "manifest_path": manifest_path.to_string_lossy().to_string(),
        "current_path": current_path.to_string_lossy().to_string()
    });
    if let Some(calibration) = calibration {
        let object = fields
            .as_object_mut()
            .expect("artifact_exported fields must be an object");
        object.insert(
            "calibration_metadata_path".to_string(),
            serde_json::json!(calibration.path.to_string_lossy().to_string()),
        );
        object.insert(
            "calibration".to_string(),
            serde_json::json!({
                "schema_version": calibration.metadata.schema_version.to_string(),
                "calibration_artifact": calibration.metadata.calibration_artifact,
                "artifact_version": calibration.metadata.artifact_version,
                "method": calibration.metadata.method,
                "created_at_unix_ms": calibration.metadata.created_at_unix_ms,
            }),
        );
    }

    event_line("info", "train", "artifact_exported", fields)
}

pub fn write_train_done_event(train_dir: &Path, contract: &serde_json::Value) -> io::Result<()> {
    let path = observability_path(train_dir);
    let line = train_done_event_line(contract);
    append_json_line(&path, &line)
}

pub fn train_done_event_line(contract: &serde_json::Value) -> String {
    event_line("info", "train", "train_done", contract.clone())
}

pub fn write_train_distributed_shard_metadata_invalid_event(
    train_dir: &Path,
    backend: &str,
    artifact_version: &str,
    err: &DistributedShardMetadataLoadError,
) -> io::Result<()> {
    let path = observability_path(train_dir);
    let line = super::distributed_metadata::distributed_shard_metadata_invalid_event_line(
        backend,
        artifact_version,
        err,
    );
    append_json_line(&path, &line)
}

pub fn write_train_distributed_runtime_execution_event(
    train_dir: &Path,
    artifact_path: &Path,
    artifact: &serde_json::Value,
) -> io::Result<()> {
    let path = observability_path(train_dir);
    let line = distributed_runtime_execution_written_event_line(artifact_path, artifact);
    append_json_line(&path, &line)
}
