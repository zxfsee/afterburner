use std::path::Path;

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

use crate::{
    data::{MnistBatch, test_loader, train_loader},
    model::{Model, ModelConfig},
};

use super::{
    artifacts::{
        artifact_dirs, copy_calibration_metadata_sidecar_if_present, export_inference_artifact,
        inference_version_dir,
    },
    contracts::{
        kernel_adoption_threshold_contract_value, training_scalability_contract_value,
        write_kernel_adoption_threshold_contract, write_training_scalability_contract,
    },
    distributed_metadata::{
        distributed_shard_metadata_path_from_env, validate_distributed_shard_metadata_load_path,
    },
    observability::{
        write_train_distributed_shard_metadata_invalid_event, write_train_done_event,
        write_train_event, write_train_export_event,
    },
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
    #[config(default = "\"artifacts\".to_string()")]
    pub artifacts_dir: String,
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
    B::seed(&device, config.seed);

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

    if let Some(metadata_path) = distributed_shard_metadata_path_from_env()
        && let Err(err) = validate_distributed_shard_metadata_load_path(&metadata_path)
    {
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

    let train_model_path = train_dir.join("model.mpk");
    result
        .model
        .save_file(&train_model_path, &CompactRecorder::new())
        .expect("save trained model");

    let exported =
        export_inference_artifact(&train_model_path, &inference_root, &config.artifact_version)
            .expect("export inference model");
    let calibration = copy_calibration_metadata_sidecar_if_present(
        &train_dir,
        &inference_dir,
        &config.artifact_version,
    )
    .expect("copy calibration sidecar if present");

    let manifest_path = inference_dir.join(crate::manifest::MANIFEST_FILENAME);
    let current_path = inference_root.join(crate::manifest::CURRENT_VERSION_FILENAME);
    write_train_export_event(
        &train_dir,
        &backend,
        &config.artifact_version,
        &exported,
        &manifest_path,
        &current_path,
        calibration.as_ref(),
    )
    .expect("write train export event");

    let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
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

fn backend_label() -> String {
    std::env::var("BACKEND")
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_else(|_| "wgpu".to_string())
}
