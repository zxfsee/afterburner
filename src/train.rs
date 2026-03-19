use burn::{
    config::Config,
    optim::AdamConfig,
    prelude::*,
    record::CompactRecorder,
    tensor::backend::AutodiffBackend,
    train::{
        InferenceStep, Learner, SupervisedTraining, TrainOutput, TrainStep,
        metric::{AccuracyMetric, LossMetric},
    },
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::{env, io};

use crate::{
    data::{MnistBatch, test_loader, train_loader},
    manifest::{ArtifactManifest, CURRENT_VERSION_FILENAME, INPUT_DTYPE, compute_sha256_hex},
    model::{MODEL_ARCH_ID, MODEL_ARCH_VERSION, MODEL_KERNEL_SITES, Model, ModelConfig},
    observability::{append_json_line, event_line},
};

const DISTRIBUTED_SHARD_METADATA_PATH_ENV: &str = "AFTERBURNER_DISTRIBUTED_SHARD_METADATA_PATH";
const MNIST_TRAIN_SAMPLES_PER_EPOCH: u64 = 60_000;
pub const TRAINING_SCALABILITY_CONTRACT_FILENAME: &str = "training_scalability_contract.json";
pub const TRAINING_SCALABILITY_CONTRACT_SCHEMA_VERSION: &str = "1";
pub const KERNEL_ADOPTION_CONTRACT_FILENAME: &str = "kernel_adoption_thresholds.json";
pub const KERNEL_ADOPTION_CONTRACT_SCHEMA_VERSION: &str = "1";
const KERNEL_ADOPTION_MIN_CONV_SITE_COVERAGE_RATIO: f64 = 0.75;
const KERNEL_ADOPTION_MIN_PROFILE_BATCH_SIZE: u64 = 128;
const KERNEL_ADOPTION_REQUIRED_CONSECUTIVE_REGRESSIONS: u64 = 2;
const KERNEL_ADOPTION_PROFILE_ARTIFACT: &str = "artifacts/eval/backend_performance_profile.json";

#[derive(Config, Debug)]
pub struct TrainingConfig {
    #[config(default = 64)]
    pub batch_size: usize,
    #[config(default = 10)]
    pub num_epochs: usize,
    #[config(default = 1e-3)]
    pub learning_rate: f64,
    #[config(default = 0)]
    pub num_workers: usize,
    #[config(default = 42)]
    pub seed: u64,
    /// Root directory for *all* outputs (train + inference contract).
    /// This is intentionally a single knob so runs are relocatable.
    #[config(default = "\"artifacts\".to_string()")]
    pub artifacts_dir: String,
    /// Semantic version for the inference artifact.
    #[config(default = "\"0.1.0\".to_string()")]
    pub artifact_version: String,

    pub model: ModelConfig,
}

impl<B: AutodiffBackend> TrainStep for Model<B> {
    type Input = MnistBatch<B>;
    type Output = burn::train::ClassificationOutput<B>;

    fn step(&self, batch: Self::Input) -> TrainOutput<Self::Output> {
        let item = self.forward_classification(batch.images, batch.targets);
        let grads = item.loss.backward();
        TrainOutput::new(self, grads, item)
    }
}

impl<B: Backend> InferenceStep for Model<B> {
    type Input = MnistBatch<B>;
    type Output = burn::train::ClassificationOutput<B>;

    fn step(&self, batch: Self::Input) -> Self::Output {
        self.forward_classification(batch.images, batch.targets)
    }
}

pub fn train<B: AutodiffBackend>(config: TrainingConfig, device: B::Device) {
    let start = std::time::Instant::now();
    // INFO: Startup performs only deterministic work.
    // No background initialization to keep failure modes observable.
    B::seed(&device, config.seed);

    // Directory convention:
    // - train/: mutable, iterative outputs (checkpoints, logs, metrics)
    // - inference/: immutable deployment contract consumed by runtime binaries
    let root = Path::new(&config.artifacts_dir);
    let (train_dir, inference_root) = artifact_dirs(root);
    let inference_dir = inference_version_dir(&inference_root, &config.artifact_version);

    std::fs::create_dir_all(&train_dir).expect("create train dir");
    std::fs::create_dir_all(&inference_dir).expect("create inference dir");

    let backend = backend_label();
    write_train_event(
        &train_dir,
        "train_start",
        &backend,
        &config,
        &train_dir,
        &inference_dir,
    )
    .expect("write train start event");

    if let Some(metadata_path) = distributed_shard_metadata_path_from_env() {
        if let Err(err) = validate_distributed_shard_metadata_load_path(&metadata_path) {
            write_train_distributed_shard_metadata_invalid_event(
                &train_dir,
                &backend,
                &config.artifact_version,
                &err,
            )
            .expect("write distributed shard metadata invalid event");
            panic!(
                "distributed shard metadata invalid at {}: {}",
                err.metadata_path().to_string_lossy(),
                err.detail()
            );
        }
    }

    let train_loader = train_loader::<B>(
        config.batch_size,
        config.num_workers,
        config.seed,
        device.clone(),
    );
    let valid_loader =
        test_loader::<B::InnerBackend>(config.batch_size, config.num_workers, device.clone());

    let optim = AdamConfig::new().init::<B, Model<B>>();

    let learner = Learner::new(config.model.init::<B>(&device), optim, config.learning_rate);

    let result = SupervisedTraining::new(&train_dir, train_loader, valid_loader)
        .metric_train_numeric(LossMetric::new())
        .metric_train_numeric(AccuracyMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .metric_valid_numeric(AccuracyMetric::new())
        .num_epochs(config.num_epochs)
        .with_file_checkpointer(CompactRecorder::new())
        .launch(learner);

    // Training output (mutable; not part of inference contract).
    let train_model_path = train_dir.join("model.mpk");
    result
        .model
        .save_file(&train_model_path, &CompactRecorder::new())
        .expect("save trained model");

    // Export the inference contract (immutable input to runtime).
    // This is the *only* file inference binaries depend on.
    // NOTE: Keep this export step narrow: inference must not depend on any other training outputs.
    let exported =
        export_inference_artifact(&train_model_path, &inference_root, &config.artifact_version)
            .expect("export inference model");

    let manifest_path = inference_dir.join(crate::manifest::MANIFEST_FILENAME);
    let current_path = inference_root.join(CURRENT_VERSION_FILENAME);
    write_train_export_event(
        &train_dir,
        &backend,
        &config.artifact_version,
        &exported,
        &manifest_path,
        &current_path,
    )
    .expect("write train export event");

    let elapsed_ms = start.elapsed().as_millis();
    let elapsed_ms = u64::try_from(elapsed_ms).unwrap_or(u64::MAX);
    let contract = training_scalability_contract_value(
        &backend,
        &config.artifact_version,
        config.batch_size,
        config.num_workers,
        config.num_epochs,
        elapsed_ms,
    );
    write_training_scalability_contract(&train_dir, &contract)
        .expect("write training scalability contract");
    let kernel_contract = kernel_adoption_threshold_contract_value(&config.artifact_version);
    write_kernel_adoption_threshold_contract(&train_dir, &kernel_contract)
        .expect("write kernel adoption threshold contract");
    write_train_done_event(&train_dir, &contract).expect("write train done event");
}

pub fn artifact_dirs(root: &Path) -> (PathBuf, PathBuf) {
    let train_dir = root.join("train");
    let inference_root = root.join("inference");
    (train_dir, inference_root)
}

pub fn export_inference_artifact(
    train_model_path: &Path,
    inference_root: &Path,
    artifact_version: &str,
) -> io::Result<PathBuf> {
    let version_dir = inference_version_dir(inference_root, artifact_version);
    std::fs::create_dir_all(&version_dir)?;
    let out = version_dir.join("model.mpk");
    std::fs::copy(train_model_path, &out)?;
    let checksum = compute_sha256_hex(&out)?;
    let manifest = ArtifactManifest::for_current("model.mpk", artifact_version, checksum);
    manifest.write_to_dir(&version_dir)?;
    write_current_version(inference_root, artifact_version)?;
    Ok(out)
}

pub fn inference_version_dir(inference_root: &Path, artifact_version: &str) -> PathBuf {
    inference_root.join(artifact_version)
}

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

fn write_current_version(inference_root: &Path, artifact_version: &str) -> io::Result<PathBuf> {
    let path = inference_root.join(CURRENT_VERSION_FILENAME);
    std::fs::write(&path, format!("{artifact_version}\n"))?;
    Ok(path)
}

fn backend_label() -> String {
    env::var("BACKEND")
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_else(|_| "wgpu".to_string())
}

fn distributed_shard_metadata_path_from_env() -> Option<PathBuf> {
    env::var(DISTRIBUTED_SHARD_METADATA_PATH_ENV)
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

fn observability_path(train_dir: &Path) -> PathBuf {
    train_dir.join("observability.jsonl")
}

fn write_train_event(
    train_dir: &Path,
    event: &str,
    backend: &str,
    config: &TrainingConfig,
    metrics_dir: &Path,
    inference_dir: &Path,
) -> io::Result<()> {
    let path = observability_path(train_dir);
    let line = train_start_event_line(backend, event, config, metrics_dir, inference_dir);
    append_json_line(&path, &line)
}

pub fn train_start_event_line(
    backend: &str,
    event: &str,
    config: &TrainingConfig,
    metrics_dir: &Path,
    inference_dir: &Path,
) -> String {
    let planned_samples = planned_training_samples(config.num_epochs);
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

fn write_train_export_event(
    train_dir: &Path,
    backend: &str,
    artifact_version: &str,
    artifact_path: &Path,
    manifest_path: &Path,
    current_path: &Path,
) -> io::Result<()> {
    let path = observability_path(train_dir);
    let line = artifact_exported_event_line(
        backend,
        artifact_version,
        artifact_path,
        manifest_path,
        current_path,
    );
    append_json_line(&path, &line)
}

pub fn artifact_exported_event_line(
    backend: &str,
    artifact_version: &str,
    artifact_path: &Path,
    manifest_path: &Path,
    current_path: &Path,
) -> String {
    event_line(
        "info",
        "train",
        "artifact_exported",
        serde_json::json!({
            "backend": backend,
            "artifact_version": artifact_version,
            "artifact_path": artifact_path.to_string_lossy().to_string(),
            "manifest_path": manifest_path.to_string_lossy().to_string(),
            "current_path": current_path.to_string_lossy().to_string()
        }),
    )
}

fn write_train_done_event(train_dir: &Path, contract: &serde_json::Value) -> io::Result<()> {
    let path = observability_path(train_dir);
    let line = train_done_event_line(contract);
    append_json_line(&path, &line)
}

fn write_train_distributed_shard_metadata_invalid_event(
    train_dir: &Path,
    backend: &str,
    artifact_version: &str,
    err: &DistributedShardMetadataLoadError,
) -> io::Result<()> {
    let path = observability_path(train_dir);
    let line = distributed_shard_metadata_invalid_event_line(backend, artifact_version, err);
    append_json_line(&path, &line)
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

pub fn training_scalability_contract_value(
    backend: &str,
    artifact_version: &str,
    batch_size: usize,
    worker_parallelism: usize,
    num_epochs: usize,
    elapsed_ms: u64,
) -> serde_json::Value {
    let samples_per_epoch = MNIST_TRAIN_SAMPLES_PER_EPOCH;
    let planned_samples = planned_training_samples(num_epochs);
    let throughput_samples_per_sec = (planned_samples as f64 * 1000.0) / elapsed_ms.max(1) as f64;

    serde_json::json!({
        "schema_version": TRAINING_SCALABILITY_CONTRACT_SCHEMA_VERSION,
        "backend": backend,
        "artifact_version": artifact_version,
        "batch_size": batch_size,
        "worker_parallelism": worker_parallelism,
        "num_epochs": num_epochs,
        "samples_per_epoch": samples_per_epoch,
        "planned_samples": planned_samples,
        "elapsed_ms": elapsed_ms,
        "throughput_samples_per_sec": throughput_samples_per_sec
    })
}

pub fn kernel_adoption_threshold_contract_value(artifact_version: &str) -> serde_json::Value {
    let mut kernel_site_counts = BTreeMap::<String, u64>::new();
    let mut padding_modes = BTreeSet::<String>::new();

    for site in MODEL_KERNEL_SITES {
        let key = format!("{}x{}", site.kernel_size[0], site.kernel_size[1]);
        *kernel_site_counts.entry(key).or_insert(0) += 1;
        padding_modes.insert(site.padding.to_string());
    }

    let kernel_sites = MODEL_KERNEL_SITES
        .iter()
        .map(|site| {
            serde_json::json!({
                "name": site.name,
                "kind": site.kind,
                "kernel_size": site.kernel_size,
                "padding": site.padding,
            })
        })
        .collect::<Vec<_>>();

    let kernel_shape_coverage = kernel_site_counts
        .into_iter()
        .map(|(kernel_size, site_count)| {
            serde_json::json!({
                "kernel_size": kernel_size,
                "site_count": site_count,
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "schema_version": KERNEL_ADOPTION_CONTRACT_SCHEMA_VERSION,
        "artifact_version": artifact_version,
        "decision": "defer_custom_kernels",
        "model": {
            "architecture_id": MODEL_ARCH_ID,
            "architecture_version": MODEL_ARCH_VERSION,
            "input_dtype": INPUT_DTYPE,
            "conv_site_count": MODEL_KERNEL_SITES.len(),
            "kernel_shape_coverage": kernel_shape_coverage,
            "kernel_sites": kernel_sites,
        },
        "adoption_guard": {
            "minimum_conv_site_coverage_ratio": KERNEL_ADOPTION_MIN_CONV_SITE_COVERAGE_RATIO,
            "required_padding_modes": padding_modes.into_iter().collect::<Vec<_>>(),
            "forbid_grouped_convolution": true,
            "forbid_dilated_convolution": true,
            "supporting_evidence": {
                "profile_artifact": KERNEL_ADOPTION_PROFILE_ARTIFACT,
                "minimum_profile_batch_size": KERNEL_ADOPTION_MIN_PROFILE_BATCH_SIZE,
                "required_consecutive_regressions": KERNEL_ADOPTION_REQUIRED_CONSECUTIVE_REGRESSIONS,
            }
        }
    })
}

pub fn write_training_scalability_contract(
    train_dir: &Path,
    contract: &serde_json::Value,
) -> io::Result<PathBuf> {
    let path = train_dir.join(TRAINING_SCALABILITY_CONTRACT_FILENAME);
    let json = serde_json::to_string_pretty(contract).map_err(|err| {
        io::Error::other(format!("serialize training scalability contract: {err}"))
    })?;
    std::fs::write(&path, json)?;
    Ok(path)
}

pub fn write_kernel_adoption_threshold_contract(
    train_dir: &Path,
    contract: &serde_json::Value,
) -> io::Result<PathBuf> {
    let path = train_dir.join(KERNEL_ADOPTION_CONTRACT_FILENAME);
    let json = serde_json::to_string_pretty(contract).map_err(|err| {
        io::Error::other(format!(
            "serialize kernel adoption threshold contract: {err}"
        ))
    })?;
    std::fs::write(&path, json)?;
    Ok(path)
}

pub fn train_done_event_line(contract: &serde_json::Value) -> String {
    event_line("info", "train", "train_done", contract.clone())
}

fn planned_training_samples(num_epochs: usize) -> u64 {
    MNIST_TRAIN_SAMPLES_PER_EPOCH.saturating_mul(num_epochs as u64)
}
