use std::env;
use std::path::Path;
use std::time::Instant;

use afterburner::infer::{InferError, load_model, logits_from_model, parse_weights_path_from_args};
use afterburner::manifest::ManifestError;
use afterburner::observability::emit_event;
use burn::prelude::*;
use burn::tensor::activation::softmax;
use burn::{backend::ndarray::NdArray, backend::wgpu::Wgpu};
use serde_json::json;

type GpuBackend = Wgpu<f32, i32>;
type CpuBackend = NdArray<f32>;

pub fn run<I>(args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let args: Vec<String> = args.collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", usage());
        return 0;
    }

    if let Err(err) = run_inner(args.into_iter()) {
        emit_error(&err);
        return 2;
    }

    0
}

fn run_inner<I>(args: I) -> Result<(), InferError>
where
    I: Iterator<Item = String>,
{
    let weights_path =
        parse_weights_path_from_args(std::iter::once("infer".to_string()).chain(args));

    let use_cpu = env::var("BACKEND")
        .map(|v| v.eq_ignore_ascii_case("cpu"))
        .unwrap_or(false);
    let backend = if use_cpu { "cpu" } else { "wgpu" };
    let t0 = Instant::now();
    let artifact = weights_path.to_string_lossy().to_string();

    if use_cpu {
        run_infer::<CpuBackend>(&weights_path, backend, artifact.as_str())?;
    } else {
        run_infer::<GpuBackend>(&weights_path, backend, artifact.as_str())?;
    }

    let elapsed_ms = t0.elapsed().as_millis();
    emit_event(
        "info",
        "infer_cli",
        "infer_done",
        json!({"backend": backend, "artifact": artifact, "elapsed_ms": elapsed_ms}),
    );
    Ok(())
}

fn run_infer<B: Backend>(
    weights_path: &Path,
    backend: &str,
    artifact: &str,
) -> Result<(), InferError> {
    let device = B::Device::default();

    let t_load = Instant::now();
    let model = load_model::<B>(weights_path, &device)?;

    let load_ms = t_load.elapsed().as_millis();
    emit_event(
        "info",
        "infer_cli",
        "artifact_load_ok",
        json!({"backend": backend, "artifact": artifact, "elapsed_ms": load_ms}),
    );
    emit_event(
        "info",
        "infer_cli",
        "backend_selected",
        json!({"backend": backend}),
    );

    let image = [[0.0f32; 28]; 28];
    let logits = logits_from_model(&model, &device, image);
    let probs = softmax(logits, 1);
    println!("Probabilities: {probs}");
    Ok(())
}

fn emit_error(err: &InferError) {
    match err {
        InferError::ArtifactMissing { path } => {
            emit_event(
                "error",
                "infer_cli",
                "infer_error",
                json!({"kind": "artifact_missing", "artifact": path.to_string_lossy().to_string()}),
            );
        }
        InferError::ManifestMissing { path } => {
            emit_event(
                "error",
                "infer_cli",
                "infer_error",
                json!({"kind": "manifest_missing", "manifest": path.to_string_lossy().to_string()}),
            );
        }
        InferError::ManifestInvalid(manifest_err) => match manifest_err {
            ManifestError::Io(err) => {
                emit_event(
                    "error",
                    "infer_cli",
                    "infer_error",
                    json!({"kind": "manifest_io_error", "detail": err.to_string()}),
                );
            }
            ManifestError::TomlParse(err) => {
                emit_event(
                    "error",
                    "infer_cli",
                    "infer_error",
                    json!({"kind": "manifest_parse_error", "detail": err.to_string()}),
                );
            }
            ManifestError::MissingField(field) => {
                emit_event(
                    "error",
                    "infer_cli",
                    "infer_error",
                    json!({"kind": "manifest_missing_field", "field": field}),
                );
            }
            ManifestError::InvalidField(field, detail) => {
                emit_event(
                    "error",
                    "infer_cli",
                    "infer_error",
                    json!({"kind": "manifest_invalid_field", "field": field, "detail": detail}),
                );
            }
            ManifestError::Mismatch {
                field,
                expected,
                actual,
            } => {
                emit_event(
                    "error",
                    "infer_cli",
                    "infer_error",
                    json!({"kind": "manifest_mismatch", "field": field, "expected": expected, "actual": actual}),
                );
            }
        },
        InferError::ArtifactLoadFailed { detail, path } => {
            emit_event(
                "error",
                "infer_cli",
                "infer_error",
                json!({"kind": "artifact_load_failed", "artifact": path.to_string_lossy().to_string(), "detail": detail}),
            );
        }
    }
}

fn usage() -> &'static str {
    "usage: afterburner infer [artifact_path] [--artifact PATH]"
}
