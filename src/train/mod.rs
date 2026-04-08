pub mod artifacts;
pub mod contracts;
pub mod distributed_metadata;
pub mod observability;
pub mod runtime;

pub use artifacts::{
    ExportedCalibrationSidecar, artifact_dirs, copy_calibration_metadata_sidecar_if_present,
    export_inference_artifact, inference_version_dir,
};
pub use contracts::{
    KERNEL_ADOPTION_CONTRACT_FILENAME, KERNEL_ADOPTION_CONTRACT_SCHEMA_VERSION,
    TRAINING_SCALABILITY_CONTRACT_FILENAME, TRAINING_SCALABILITY_CONTRACT_SCHEMA_VERSION,
    kernel_adoption_threshold_contract_value, training_scalability_contract_value,
    write_kernel_adoption_threshold_contract, write_training_scalability_contract,
};
pub use distributed_metadata::{
    DistributedShardMetadata, DistributedShardMetadataLoadError,
    distributed_shard_metadata_invalid_event_line, validate_distributed_shard_metadata,
    validate_distributed_shard_metadata_load_path,
};
pub use observability::{
    TrainStartEventContext, artifact_exported_event_line, train_done_event_line,
    train_start_event_line,
};
pub use runtime::{TrainingConfig, train};
