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
use std::path::{Path, PathBuf};
use std::{env, io};

use crate::{
    data::{MnistBatch, test_loader, train_loader},
    manifest::{ArtifactManifest, CURRENT_VERSION_FILENAME, compute_sha256_hex},
    model::{Model, ModelConfig},
    observability::{append_json_line, json_escape},
};

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
        &config.artifact_version,
        &train_dir,
        &inference_dir,
    )
    .expect("write train start event");

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
    write_train_done_event(&train_dir, &backend, &config.artifact_version, elapsed_ms)
        .expect("write train done event");
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

fn observability_path(train_dir: &Path) -> PathBuf {
    train_dir.join("observability.jsonl")
}

fn write_train_event(
    train_dir: &Path,
    event: &str,
    backend: &str,
    artifact_version: &str,
    metrics_dir: &Path,
    inference_dir: &Path,
) -> io::Result<()> {
    let path = observability_path(train_dir);
    let metrics_dir = json_escape(&metrics_dir.to_string_lossy());
    let inference_dir = json_escape(&inference_dir.to_string_lossy());
    let artifact_version = json_escape(artifact_version);
    let backend = json_escape(backend);
    let line = format!(
        r#"{{"event":"{event}","backend":"{backend}","artifact_version":"{artifact_version}","metrics_dir":"{metrics_dir}","inference_dir":"{inference_dir}"}}"#
    );
    append_json_line(&path, &line)
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
    let backend = json_escape(backend);
    let artifact_version = json_escape(artifact_version);
    let artifact_path = json_escape(&artifact_path.to_string_lossy());
    let manifest_path = json_escape(&manifest_path.to_string_lossy());
    let current_path = json_escape(&current_path.to_string_lossy());
    let line = format!(
        r#"{{"event":"artifact_exported","backend":"{backend}","artifact_version":"{artifact_version}","artifact_path":"{artifact_path}","manifest_path":"{manifest_path}","current_path":"{current_path}"}}"#
    );
    append_json_line(&path, &line)
}

fn write_train_done_event(
    train_dir: &Path,
    backend: &str,
    artifact_version: &str,
    elapsed_ms: u128,
) -> io::Result<()> {
    let path = observability_path(train_dir);
    let backend = json_escape(backend);
    let artifact_version = json_escape(artifact_version);
    let line = format!(
        r#"{{"event":"train_done","backend":"{backend}","artifact_version":"{artifact_version}","elapsed_ms":{elapsed_ms}}}"#
    );
    append_json_line(&path, &line)
}
