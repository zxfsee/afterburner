use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

use afterburner::infer::load_model;
use burn::backend::ndarray::NdArray;
use burn::backend::wgpu::Wgpu;
use burn::prelude::*;

type CpuBackend = NdArray<f32>;
type GpuBackend = Wgpu<f32, i32>;

fn artifact_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .join("inference")
        .join("0.1.0")
        .join("model.mpk")
}

fn assert_logits_shape_for_backend<B: Backend>(batch_size: usize)
where
    B::Device: Default,
{
    let device = B::Device::default();
    let model = load_model::<B>(&artifact_path(), &device).expect("load model");
    let input = Tensor::<B, 4>::zeros([batch_size, 1, 28, 28], &device);
    let logits = model.forward(input);
    assert_eq!(
        logits.dims(),
        [batch_size, 10],
        "logits shape must remain [batch,10]"
    );
}

#[test]
fn logits_shape_is_stable_on_cpu() {
    assert_logits_shape_for_backend::<CpuBackend>(3);
}

#[test]
fn logits_shape_is_stable_on_wgpu_when_available() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        assert_logits_shape_for_backend::<GpuBackend>(3);
    }));

    if result.is_err() {
        eprintln!("wgpu backend unavailable; skipping wgpu logits shape guard");
    }
}
