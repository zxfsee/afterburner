use std::path::Path;

use burn::{
    config::Config,
    optim::AdamConfig,
    prelude::*,
    record::CompactRecorder,
    tensor::backend::AutodiffBackend,
    train::{
        LearnerBuilder, TrainOutput, TrainStep, ValidStep,
        metric::{AccuracyMetric, LossMetric},
    },
};

use crate::{
    data::{MnistBatch, test_loader, train_loader},
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
    #[config(default = "\"artifacts/train\".to_string()")]
    pub artifacts_dir: String,
    pub model: ModelConfig,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            batch_size: 64,
            num_epochs: 10,
            learning_rate: 1e-3,
            num_workers: 0,
            seed: 42,
            artifacts_dir: "artifacts".into(),
            model: ModelConfig::new(10),
        }
    }
}

impl<B: AutodiffBackend> TrainStep<MnistBatch<B>, burn::train::ClassificationOutput<B>>
    for Model<B>
{
    fn step(&self, batch: MnistBatch<B>) -> TrainOutput<burn::train::ClassificationOutput<B>> {
        let item = self.forward_classification(batch.images, batch.targets);
        let grads = item.loss.backward();
        TrainOutput::new(self, grads, item)
    }
}

impl<B: Backend> ValidStep<MnistBatch<B>, burn::train::ClassificationOutput<B>> for Model<B> {
    fn step(&self, batch: MnistBatch<B>) -> burn::train::ClassificationOutput<B> {
        self.forward_classification(batch.images, batch.targets)
    }
}

pub fn train<B: AutodiffBackend>(config: TrainingConfig, device: B::Device) {
    B::seed(&device, config.seed);
    std::fs::create_dir_all(&config.artifacts_dir).expect("failed to create artifact directory");

    let train_loader = train_loader::<B>(
        config.batch_size,
        config.num_workers,
        config.seed,
        device.clone(),
    );
    let valid_loader =
        test_loader::<B::InnerBackend>(config.batch_size, config.num_workers, device.clone());

    let optim = AdamConfig::new().init::<B, Model<B>>();

    let learner = LearnerBuilder::new(config.artifacts_dir.clone())
        .metric_train_numeric(LossMetric::new())
        .metric_train_numeric(AccuracyMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .metric_valid_numeric(AccuracyMetric::new())
        .num_epochs(config.num_epochs)
        .with_file_checkpointer(CompactRecorder::new())
        .build(config.model.init::<B>(&device), optim, config.learning_rate);

    let result = learner.fit(train_loader, valid_loader);

    let path = Path::new(&config.artifacts_dir).join("model");
    result
        .model
        .save_file(path, &CompactRecorder::new())
        .expect("failed to save trained model");

    let train_dir = Path::new(&config.artifacts_dir);
    let inference_dir = Path::new("artifacts/inference");

    std::fs::create_dir_all(inference_dir).expect("create inference dir");

    std::fs::copy(train_dir.join("model.mpk"), inference_dir.join("model.mpk"))
        .expect("copy model to inference");
}
