use afterburner::model::ModelConfig;
use afterburner::train;
use burn::prelude::*;
use burn::{backend::ndarray::NdArray, backend::wgpu::Wgpu};
use burn_autodiff::Autodiff;

type GpuBackend = Wgpu<f32, i32>;
type CpuBackend = NdArray<f32>;
type GpuAutodiff = Autodiff<GpuBackend>;
type CpuAutodiff = Autodiff<CpuBackend>;

fn main() {
    // Explicit runtime knob to mirror production deployment variance:
    // same code + same artifact contract, different execution target.
    let use_cpu = std::env::var("BACKEND")
        .map(|v| v.eq_ignore_ascii_case("cpu"))
        .unwrap_or(false);

    // Single override point so CI / experiments can redirect outputs without code changes.
    // The artifact contract lives under `<ARTIFACTS_DIR>/infer/`.
    let artifact_dir = std::env::var("ARTIFACTS_DIR").ok();

    let mut config = train::TrainingConfig::new(ModelConfig::new(10));
    if let Some(dir) = artifact_dir {
        config.artifacts_dir = dir;
    }

    if use_cpu {
        let device = <CpuBackend as Backend>::Device::default();
        train::train::<CpuAutodiff>(config, device);
    } else {
        let device = <GpuBackend as Backend>::Device::default();
        train::train::<GpuAutodiff>(config, device);
    }
}
