use std::path::Path;
use std::sync::Arc;

use burn::{
    backend::{ndarray::NdArrayDevice, wgpu::WgpuDevice},
    config::Config,
    data::dataloader::DataLoader,
    optim::AdamConfig,
    prelude::*,
    record::CompactRecorder,
    tensor::backend::{AutodiffBackend, DeviceOps},
    train::{
        InferenceStep, Learner, MultiDeviceOptim, SupervisedTraining, TrainOutput, TrainStep,
        TrainingStrategy,
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
        distributed_runtime_execution_value, distributed_shard_metadata_path_from_env,
        validate_distributed_shard_metadata_load_path,
        write_distributed_runtime_execution_artifact,
    },
    observability::{
        write_train_distributed_runtime_execution_event,
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
    #[config(default = 1)]
    pub world_size: usize,
    #[config(default = "\"\".to_string()")]
    pub device_group: String,
    #[config(default = "Vec::<String>::new()")]
    pub participant_devices: Vec<String>,
    #[config(default = 0)]
    pub max_train_items: usize,
    #[config(default = 0)]
    pub max_valid_items: usize,

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

pub trait ExplicitParticipantDevice:
    Clone + std::fmt::Debug + DeviceOps + Eq + PartialEq + Sized
{
    fn resolve_participant_devices(device_refs: &[String]) -> Result<Vec<Self>, String>;
    fn participant_device_ref(device: &Self) -> String;
}

impl ExplicitParticipantDevice for NdArrayDevice {
    fn resolve_participant_devices(device_refs: &[String]) -> Result<Vec<Self>, String> {
        if device_refs.is_empty() {
            return Err("explicit DP runtime requires participant devices".to_string());
        }
        Err(format!(
            "explicit DP runtime requires distinct participant devices, but backend `cpu` only exposes one device; requested refs: {}",
            device_refs.join(",")
        ))
    }

    fn participant_device_ref(_device: &Self) -> String {
        "cpu".to_string()
    }
}

impl ExplicitParticipantDevice for WgpuDevice {
    fn resolve_participant_devices(device_refs: &[String]) -> Result<Vec<Self>, String> {
        if device_refs.is_empty() {
            return Err("explicit DP runtime requires participant devices".to_string());
        }

        let mut devices = Vec::with_capacity(device_refs.len());
        let mut seen_ids = std::collections::BTreeSet::new();
        for device_ref in device_refs {
            let device = parse_wgpu_participant_device_ref(device_ref)?;
            reject_ambiguous_wgpu_device_ref(&device, device_ref)?;
            if !seen_ids.insert(wgpu_device_key(&device)) {
                return Err(format!(
                    "explicit DP runtime requires distinct participant devices, but `{device_ref}` duplicates an existing device selection"
                ));
            }
            devices.push(device);
        }
        Ok(devices)
    }

    fn participant_device_ref(device: &Self) -> String {
        match device {
            WgpuDevice::DiscreteGpu(index) => format!("discrete:{index}"),
            WgpuDevice::IntegratedGpu(index) => format!("integrated:{index}"),
            WgpuDevice::VirtualGpu(index) => format!("virtual:{index}"),
            WgpuDevice::Cpu => "cpu".to_string(),
            WgpuDevice::DefaultDevice => "default".to_string(),
            #[allow(deprecated)]
            WgpuDevice::BestAvailable => "default".to_string(),
            WgpuDevice::Existing(id) => format!("existing:{id}"),
        }
    }
}

pub fn train<B: AutodiffBackend>(config: TrainingConfig, device: B::Device) -> Result<(), String>
where
    B::Device: ExplicitParticipantDevice,
{
    let train_loader = train_loader::<B>(
        config.batch_size,
        config.num_workers,
        config.seed,
        device.clone(),
    );
    let train_loader = limit_loader(train_loader, config.max_train_items);
    let valid_loader =
        test_loader::<B::InnerBackend>(config.batch_size, config.num_workers, device.clone());
    let valid_loader = limit_loader(valid_loader, config.max_valid_items);
    train_with_loaders(config, device, train_loader, valid_loader)
}

fn train_with_loaders<B: AutodiffBackend>(
    config: TrainingConfig,
    device: B::Device,
    train_loader: Arc<dyn DataLoader<B, MnistBatch<B>>>,
    valid_loader: Arc<dyn DataLoader<B::InnerBackend, MnistBatch<B::InnerBackend>>>,
) -> Result<(), String>
where
    B::Device: ExplicitParticipantDevice,
{
    let start = std::time::Instant::now();
    B::seed(&device, config.seed);

    let root = Path::new(&config.artifacts_dir);
    let (train_dir, inference_root) = artifact_dirs(root);
    let inference_dir = inference_version_dir(&inference_root, &config.artifact_version);
    let train_items_total = train_loader.num_items();
    let valid_items_total = valid_loader.num_items();
    let samples_per_epoch = u64::try_from(train_items_total).unwrap_or(u64::MAX);
    let participant_devices =
        resolve_training_devices::<B::Device>(&config, train_items_total, valid_items_total)?;

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
        samples_per_epoch.saturating_mul(config.num_epochs as u64),
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

    let optim = AdamConfig::new().init::<B, Model<B>>();
    let learner = Learner::new(config.model.init::<B>(&device), optim, config.learning_rate);
    let mut training = SupervisedTraining::new(&train_dir, train_loader, valid_loader)
        .metric_train_numeric(LossMetric::new())
        .metric_train_numeric(AccuracyMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .metric_valid_numeric(AccuracyMetric::new())
        .num_epochs(config.num_epochs)
        .with_file_checkpointer(CompactRecorder::new());
    if let Some(devices) = participant_devices.as_ref() {
        training = training.with_training_strategy(TrainingStrategy::MultiDevice(
            devices.clone(),
            MultiDeviceOptim::OptimMainDevice,
        ));
    }
    let result = training.launch(learner);

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

    if let Some(participant_devices) = participant_devices.as_ref() {
        let runtime_artifact = distributed_runtime_execution_value(
            &backend,
            &config.artifact_version,
            u64::try_from(config.world_size).unwrap_or(u64::MAX),
            &config.device_group,
            participant_devices
                .iter()
                .map(B::Device::participant_device_ref)
                .collect(),
            config.batch_size,
            config.num_epochs,
            train_items_total,
            valid_items_total,
        )
        .map_err(|err| format!("build distributed runtime execution artifact: {err}"))?;
        let runtime_artifact_path =
            write_distributed_runtime_execution_artifact(&train_dir, &runtime_artifact)
                .expect("write distributed runtime execution artifact");
        write_train_distributed_runtime_execution_event(
            &train_dir,
            &runtime_artifact_path,
            &runtime_artifact,
        )
        .expect("write distributed runtime execution event");
    }

    let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let contract = training_scalability_contract_value(
        &backend,
        &config.artifact_version,
        config.batch_size,
        config.num_workers,
        config.num_epochs,
        samples_per_epoch,
        elapsed_ms,
    );
    write_training_scalability_contract(&train_dir, &contract)
        .expect("write training scalability contract");
    let kernel_contract = kernel_adoption_threshold_contract_value(&config.artifact_version);
    write_kernel_adoption_threshold_contract(&train_dir, &kernel_contract)
        .expect("write kernel adoption threshold contract");
    write_train_done_event(&train_dir, &contract).expect("write train done event");
    Ok(())
}

fn backend_label() -> String {
    std::env::var("BACKEND")
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_else(|_| "wgpu".to_string())
}

fn limit_loader<B: Backend, O>(
    dataloader: Arc<dyn DataLoader<B, O>>,
    max_items: usize,
) -> Arc<dyn DataLoader<B, O>> {
    if max_items == 0 {
        return dataloader;
    }

    let end = max_items.min(dataloader.num_items());
    dataloader.slice(0, end)
}

fn resolve_training_devices<D: ExplicitParticipantDevice>(
    config: &TrainingConfig,
    train_items_total: usize,
    valid_items_total: usize,
) -> Result<Option<Vec<D>>, String> {
    if config.world_size <= 1 {
        if !config.device_group.trim().is_empty() || !config.participant_devices.is_empty() {
            return Err(
                "single-device train path must not declare device_group or participant_devices"
                    .to_string(),
            );
        }
        return Ok(None);
    }

    if config.device_group.trim().is_empty() {
        return Err("explicit DP runtime requires a non-empty device_group".to_string());
    }
    if config.participant_devices.len() != config.world_size {
        return Err(format!(
            "explicit DP runtime requires {} participant devices, got {}",
            config.world_size,
            config.participant_devices.len()
        ));
    }
    if train_items_total < config.world_size {
        return Err(format!(
            "explicit DP runtime requires at least {} train items so every participant executes, got {}",
            config.world_size, train_items_total
        ));
    }
    if valid_items_total < config.world_size {
        return Err(format!(
            "explicit DP runtime requires at least {} validation items so every participant executes, got {}",
            config.world_size, valid_items_total
        ));
    }

    D::resolve_participant_devices(&config.participant_devices).map(Some)
}

fn parse_wgpu_participant_device_ref(value: &str) -> Result<WgpuDevice, String> {
    match value {
        "default" => Ok(WgpuDevice::DefaultDevice),
        "cpu" => Ok(WgpuDevice::Cpu),
        _ if value.starts_with("discrete:") => value
            .trim_start_matches("discrete:")
            .parse::<usize>()
            .map(WgpuDevice::DiscreteGpu)
            .map_err(|_| format!("invalid discrete device ref `{value}`")),
        _ if value.starts_with("integrated:") => value
            .trim_start_matches("integrated:")
            .parse::<usize>()
            .map(WgpuDevice::IntegratedGpu)
            .map_err(|_| format!("invalid integrated device ref `{value}`")),
        _ if value.starts_with("virtual:") => value
            .trim_start_matches("virtual:")
            .parse::<usize>()
            .map(WgpuDevice::VirtualGpu)
            .map_err(|_| format!("invalid virtual device ref `{value}`")),
        _ if value.starts_with("existing:") => value
            .trim_start_matches("existing:")
            .parse::<u32>()
            .map(WgpuDevice::Existing)
            .map_err(|_| format!("invalid existing device ref `{value}`")),
        _ => Err(format!("unknown participant device ref `{value}`")),
    }
}

fn reject_ambiguous_wgpu_device_ref(device: &WgpuDevice, device_ref: &str) -> Result<(), String> {
    match device {
        WgpuDevice::DefaultDevice => Err(format!(
            "explicit DP runtime requires concrete participant devices; `{device_ref}` is ambiguous"
        )),
        #[allow(deprecated)]
        WgpuDevice::BestAvailable => Err(format!(
            "explicit DP runtime requires concrete participant devices; `{device_ref}` is ambiguous"
        )),
        WgpuDevice::Existing(_) => Err(format!(
            "explicit DP runtime does not accept `existing:*` participant devices because they cannot be proven distinct at the contract boundary; got `{device_ref}`"
        )),
        _ => Ok(()),
    }
}

fn wgpu_device_key(device: &WgpuDevice) -> (&'static str, usize) {
    match device {
        WgpuDevice::DiscreteGpu(index) => ("discrete", *index),
        WgpuDevice::IntegratedGpu(index) => ("integrated", *index),
        WgpuDevice::VirtualGpu(index) => ("virtual", *index),
        WgpuDevice::Cpu => ("cpu", 0),
        WgpuDevice::DefaultDevice => ("default", 0),
        #[allow(deprecated)]
        WgpuDevice::BestAvailable => ("default", 0),
        WgpuDevice::Existing(id) => ("existing", *id as usize),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TrainingConfig, WgpuDevice, limit_loader, resolve_training_devices, train_with_loaders,
    };
    use crate::data::MnistBatcher;
    use crate::model::ModelConfig;
    use burn::backend::ndarray::NdArray;
    use burn::data::dataloader::DataLoaderBuilder;
    use burn::data::dataset::InMemDataset;
    use burn::data::dataset::vision::MnistItem;
    use burn::prelude::Backend;
    use burn_autodiff::Autodiff;
    type CpuBackend = NdArray<f32>;
    type CpuAutodiff = Autodiff<CpuBackend>;

    #[test]
    fn explicit_single_node_dp_runtime_rejects_cpu_without_distinct_participants() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let device = <CpuAutodiff as Backend>::Device::default();
        let train_loader = synthetic_loader::<CpuAutodiff>(&device, 16, 8);
        let valid_loader =
            synthetic_loader::<CpuBackend>(&<CpuBackend as Backend>::Device::default(), 8, 4);

        let mut config = TrainingConfig::new(ModelConfig::new(10));
        config.artifacts_dir = tmp.path().join("artifacts").display().to_string();
        config.artifact_version = "0.3.0-dp".to_string();
        config.batch_size = 8;
        config.num_epochs = 1;
        config.world_size = 2;
        config.device_group = "single-node:cpu".to_string();
        config.participant_devices = vec!["cpu".to_string(), "cpu".to_string()];

        let err = train_with_loaders(config, device, train_loader, valid_loader)
            .expect_err("cpu dp path must reject non-distinct participant devices");
        assert!(
            err.contains("backend `cpu` only exposes one device"),
            "runtime must reject explicit dp on cpu without distinct devices: {err}"
        );
    }

    #[test]
    fn bounded_loader_limit_preserves_requested_item_count() {
        let device = <CpuAutodiff as Backend>::Device::default();
        let loader = synthetic_loader::<CpuAutodiff>(&device, 16, 8);
        let limited = limit_loader(loader, 5);
        assert_eq!(limited.num_items(), 5);
    }

    #[test]
    fn explicit_single_node_dp_requires_every_participant_to_receive_work() {
        let mut config = TrainingConfig::new(ModelConfig::new(10));
        config.world_size = 2;
        config.device_group = "single-node:wgpu".to_string();
        config.participant_devices = vec!["default".to_string(), "cpu".to_string()];

        let err = resolve_training_devices::<WgpuDevice>(&config, 1, 2)
            .expect_err("insufficient train items must fail");
        assert!(err.contains("requires at least 2 train items"));
    }

    #[test]
    fn explicit_single_node_dp_rejects_duplicate_participant_devices() {
        let mut config = TrainingConfig::new(ModelConfig::new(10));
        config.world_size = 2;
        config.device_group = "single-node:wgpu".to_string();
        config.participant_devices = vec!["discrete:0".to_string(), "discrete:0".to_string()];

        let err = resolve_training_devices::<WgpuDevice>(&config, 16, 8)
            .expect_err("duplicate participant devices must fail");
        assert!(err.contains("requires distinct participant devices"));
    }

    #[test]
    fn explicit_single_node_dp_rejects_ambiguous_default_device_refs() {
        let mut config = TrainingConfig::new(ModelConfig::new(10));
        config.world_size = 2;
        config.device_group = "single-node:wgpu".to_string();
        config.participant_devices = vec!["default".to_string(), "discrete:0".to_string()];

        let err = resolve_training_devices::<WgpuDevice>(&config, 16, 8)
            .expect_err("ambiguous default device ref must fail");
        assert!(err.contains("requires concrete participant devices"));
    }

    fn synthetic_loader<B: Backend>(
        device: &B::Device,
        sample_count: usize,
        batch_size: usize,
    ) -> std::sync::Arc<dyn burn::data::dataloader::DataLoader<B, crate::data::MnistBatch<B>>> {
        let items = (0..sample_count)
            .map(|index| MnistItem {
                image: synthetic_image(index),
                label: (index % 10) as u8,
            })
            .collect::<Vec<_>>();
        let dataset = InMemDataset::new(items);
        DataLoaderBuilder::new(MnistBatcher)
            .batch_size(batch_size)
            .num_workers(0)
            .set_device(device.clone())
            .build(dataset)
    }

    fn synthetic_image(seed: usize) -> [[f32; 28]; 28] {
        let mut image = [[0.0; 28]; 28];
        for (row_index, row) in image.iter_mut().enumerate() {
            for (col_index, pixel) in row.iter_mut().enumerate() {
                let value = ((seed + row_index + col_index) % 7) as f32 / 6.0;
                *pixel = value;
            }
        }
        image
    }
}
