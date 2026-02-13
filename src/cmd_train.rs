use std::any::Any;

use afterburner::model::ModelConfig;
use afterburner::train;
use burn::prelude::*;
use burn::{backend::ndarray::NdArray, backend::wgpu::Wgpu};
use burn_autodiff::Autodiff;

type GpuBackend = Wgpu<f32, i32>;
type CpuBackend = NdArray<f32>;
type GpuAutodiff = Autodiff<GpuBackend>;
type CpuAutodiff = Autodiff<CpuBackend>;

pub fn run<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    if let Some(arg) = args.next() {
        eprintln!("unknown argument for train: {arg}");
        eprintln!("{}", usage());
        return 2;
    }

    let backend = std::env::var("BACKEND")
        .map(|v| v.to_ascii_lowercase())
        .unwrap_or_else(|_| "wgpu".to_string());

    let config = training_config_from_env();

    if backend == "cpu" {
        let device = <CpuBackend as Backend>::Device::default();
        train::train::<CpuAutodiff>(config, device);
        return 0;
    }

    let gpu_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let device = <GpuBackend as Backend>::Device::default();
        train::train::<GpuAutodiff>(config, device);
    }));

    if let Err(payload) = gpu_result {
        if is_missing_wgpu_adapter_panic(payload.as_ref()) {
            eprintln!(
                r#"{{"event":"backend_fallback","from":"wgpu","to":"cpu","reason":"no_adapter"}}"#
            );
            let cpu_config = training_config_from_env();
            let device = <CpuBackend as Backend>::Device::default();
            train::train::<CpuAutodiff>(cpu_config, device);
            return 0;
        }

        std::panic::resume_unwind(payload);
    }

    0
}

fn training_config_from_env() -> train::TrainingConfig {
    // Explicit runtime knob to mirror production deployment variance:
    // same code + same artifact contract, different execution target.
    // Single override point so CI / experiments can redirect outputs without code changes.
    // The artifact contract lives under `<ARTIFACTS_DIR>/inference/`.
    let artifact_dir = std::env::var("ARTIFACTS_DIR").ok();
    let artifact_version = std::env::var("ARTIFACT_VERSION").ok();

    let mut config = train::TrainingConfig::new(ModelConfig::new(10));
    if let Some(dir) = artifact_dir {
        config.artifacts_dir = dir;
    }
    if let Some(version) = artifact_version {
        config.artifact_version = version;
    }

    config
}

fn is_missing_wgpu_adapter_panic(payload: &(dyn Any + Send)) -> bool {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return message.contains("No possible adapter available for backend");
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.contains("No possible adapter available for backend");
    }
    false
}

fn usage() -> &'static str {
    "usage: afterburner train"
}
