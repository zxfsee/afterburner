use std::env;
use std::path::PathBuf;

use afterburner::infer::parse_weights_path;
use burn::prelude::*;
use burn::record::CompactRecorder;
use burn::tensor::activation::softmax;
use burn::{backend::ndarray::NdArray, backend::wgpu::Wgpu};

type GpuBackend = Wgpu<f32, i32>;
type CpuBackend = NdArray<f32>;

fn main() {
    // Inference consumes only the contract artifact (no training internals).
    // Default path matches README + ADR-002.
    // NOTE: This path is the inference contract (ADR-002). Training outputs under artifacts/train are not consumed here.
    let weights_path = parse_weights_path();

    if !weights_path.exists() {
        panic!("inference artifact not found: {}", weights_path.display());
    }

    let use_cpu = env::var("BACKEND")
        .map(|v| v.eq_ignore_ascii_case("cpu"))
        .unwrap_or(false);

    if use_cpu {
        run_infer::<CpuBackend>(weights_path);
    } else {
        run_infer::<GpuBackend>(weights_path);
    }
}

fn run_infer<B: Backend>(weights_path: PathBuf) {
    let device = B::Device::default();
    let recorder = CompactRecorder::new();

    eprintln!("Loading inference artifact: {}", weights_path.display());

    // Load into an initialized model instance. This keeps the artifact format stable while
    // allowing the model structure to remain explicit in code (no magic deserialization).
    // NOTE: Failing fast here is intentional.
    // Silent fallback would hide artifact drift and create non-reproducible inference behavior.
    let model = afterburner::model::ModelConfig::new(10)
        .init::<B>(&device)
        .load_file(weights_path, &recorder, &device)
        .expect("load model artifact");

    // TODO: Replace placeholder input with real preprocessed MNIST input.
    // Must apply the same normalization as training (see data::MnistBatcher).
    let input = Tensor::<B, 4>::zeros([1, 1, 28, 28], &device);
    let logits = model.forward(input);
    let probs = softmax(logits, 1);

    println!("Probabilities: {probs}");
}
