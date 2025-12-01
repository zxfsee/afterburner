#![recursion_limit = "256"]

mod data;
mod model;
mod train;

use burn::prelude::*;
use burn::{backend::ndarray::NdArray, backend::wgpu::Wgpu};
use burn_autodiff::Autodiff;

type GpuBackend = Wgpu<f32, i32>;
type CpuBackend = NdArray<f32>;
type GpuAutodiff = Autodiff<GpuBackend>;
type CpuAutodiff = Autodiff<CpuBackend>;

fn main() {
    // BACKEND=cpu to force CPU; otherwise wgpu.
    let use_cpu = std::env::var("BACKEND")
        .map(|v| v.eq_ignore_ascii_case("cpu"))
        .unwrap_or(false);

    let artifact_dir = std::env::var("ARTIFACTS_DIR").ok();

    if use_cpu {
        let mut config = train::TrainingConfig::default();
        if let Some(dir) = artifact_dir.clone() {
            config.artifacts_dir = dir;
        }
        let device = <CpuBackend as Backend>::Device::default();
        train::train::<CpuAutodiff>(config, device);
    } else {
        let mut config = train::TrainingConfig::default();
        if let Some(dir) = artifact_dir {
            config.artifacts_dir = dir;
        }
        let device = <GpuBackend as Backend>::Device::default();
        train::train::<GpuAutodiff>(config, device);
    }
}
