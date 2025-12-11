use std::env;
use std::path::PathBuf;

use burn::prelude::*;
use burn::record::CompactRecorder;
use burn::{backend::ndarray::NdArray, backend::wgpu::Wgpu};

use afterburner::model::Model;

type GpuBackend = Wgpu<f32, i32>;
type CpuBackend = NdArray<f32>;

fn main() {
    let args: Vec<String> = env::args().collect();
    let weights_path = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("artifacts/model"));

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

    let model: Model<B> = Model::load_file(weights_path, &recorder, &device)
        .expect("failed to load model");

    // Example single input: zeros shaped as MNIST image batch [1, 1, 28, 28].
    let input = Tensor::<B, 4>::zeros([1, 1, 28, 28], &device);
    let logits = model.forward(input);
    let probs = logits.softmax(1);

    println!("Logits: {logits}");
    println!("Probabilities: {probs}");
}
