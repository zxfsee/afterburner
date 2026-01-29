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
use std::io;
use std::path::{Path, PathBuf};

use crate::{
    data::{MnistBatch, test_loader, train_loader},
    manifest::{ArtifactManifest, compute_sha256_hex},
    model::{Model, ModelConfig},
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
    // INFO: Startup performs only deterministic work.
    // No background initialization to keep failure modes observable.
    B::seed(&device, config.seed);

    // Directory convention:
    // - train/: mutable, iterative outputs (checkpoints, logs, metrics)
    // - inference/: immutable deployment contract consumed by runtime binaries
    let root = Path::new(&config.artifacts_dir);
    let (train_dir, inference_dir) = artifact_dirs(root);

    std::fs::create_dir_all(&train_dir).expect("create train dir");
    std::fs::create_dir_all(&inference_dir).expect("create inference dir");

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
    export_inference_artifact(&train_model_path, &inference_dir).expect("export inference model");
}

pub fn artifact_dirs(root: &Path) -> (PathBuf, PathBuf) {
    let train_dir = root.join("train");
    let inference_dir = root.join("inference");
    (train_dir, inference_dir)
}

pub fn export_inference_artifact(
    train_model_path: &Path,
    inference_dir: &Path,
) -> io::Result<PathBuf> {
    std::fs::create_dir_all(inference_dir)?;
    let out = inference_dir.join("model.mpk");
    std::fs::copy(train_model_path, &out)?;
    let checksum = compute_sha256_hex(&out)?;
    let manifest = ArtifactManifest::for_current("model.mpk", checksum);
    manifest.write_to_dir(inference_dir)?;
    Ok(out)
}
