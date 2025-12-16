use std::env;
use std::path::Path;
use std::time::Instant;

use afterburner::infer::parse_weights_path;
use burn::prelude::*;
use burn::record::CompactRecorder;
use burn::tensor::activation::softmax;
use burn::{backend::ndarray::NdArray, backend::wgpu::Wgpu};

type GpuBackend = Wgpu<f32, i32>;
type CpuBackend = NdArray<f32>;

fn main() {
    // Inference consumes only the contract artifact (ADR-002).
    // Default path is artifacts/inference/model.mpk.
    let weights_path = parse_weights_path();
    if !weights_path.exists() {
        let artifact = json_escape(&weights_path.to_string_lossy());
        eprintln!(r#"{{"event":"infer_error","kind":"artifact_missing","artifact":"{artifact}"}}"#);
        std::process::exit(2);
    }

    let use_cpu = env::var("BACKEND")
        .map(|v| v.eq_ignore_ascii_case("cpu"))
        .unwrap_or(false);

    let backend = if use_cpu { "cpu" } else { "wgpu" };
    let t0 = Instant::now();

    let artifact = json_escape(weights_path.to_string_lossy().as_ref());
    eprintln!(r#"{{"event":"infer_start","backend":"{backend}","artifact":"{artifact}"}}"#);

    if use_cpu {
        run_infer::<CpuBackend>(&weights_path, backend, &artifact);
    } else {
        run_infer::<GpuBackend>(&weights_path, backend, &artifact);
    }
    let elapsed_ms = t0.elapsed().as_millis();
    eprintln!(
        r#"{{"event":"infer_done","backend":"{backend}","artifact":"{artifact}","elapsed_ms":{elapsed_ms}}}"#
    );
}

fn run_infer<B: Backend>(weights_path: &Path, backend: &str, artifact: &str) {
    let device = B::Device::default();
    let recorder = CompactRecorder::new();

    eprintln!(r#"{{"event":"artifact_load_start","backend":"{backend}","artifact":"{artifact}"}}"#);

    let t_load = Instant::now();

    // Load into an initialized model instance. This keeps the artifact format stable while
    // allowing the model structure to remain explicit in code (no magic deserialization).
    // NOTE: Failing fast here is intentional.
    // Silent fallback would hide artifact drift and create non-reproducible inference behavior.
    let model = afterburner::model::ModelConfig::new(10)
        .init::<B>(&device)
        .load_file(weights_path, &recorder, &device)
        .expect("load model artifact");

    let load_ms = t_load.elapsed().as_millis();

    eprintln!(
        r#"{{"event":"artifact_load_ok","backend":"{backend}","artifact":"{artifact}","elapsed_ms":{load_ms}}}"#
    );

    // TODO: Replace placeholder input with real preprocessed MNIST input.
    // Must apply the same normalization as training (see data::MnistBatcher).
    let input = Tensor::<B, 4>::zeros([1, 1, 28, 28], &device);
    let logits = model.forward(input);
    let probs = softmax(logits, 1);

    println!("Probabilities: {probs}");
}

// Minimal JSON escaping to keep logs structured without adding a logging dependency.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}
