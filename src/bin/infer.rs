use std::env;
use std::path::Path;
use std::time::Instant;

use afterburner::infer::{manifest_path_for_weights, parse_weights_path};
use afterburner::manifest::{ArtifactManifest, ManifestError};
use burn::prelude::*;
use burn::record::CompactRecorder;
use burn::tensor::activation::softmax;
use burn::{backend::ndarray::NdArray, backend::wgpu::Wgpu};

type GpuBackend = Wgpu<f32, i32>;
type CpuBackend = NdArray<f32>;

fn main() {
    if let Err(err) = run() {
        emit_error(&err);
    }
}

fn run() -> Result<(), InferError> {
    let weights_path = parse_weights_path();

    if !weights_path.exists() {
        return Err(InferError::ArtifactMissing { path: weights_path });
    }

    let manifest_path = manifest_path_for_weights(&weights_path);
    if !manifest_path.exists() {
        return Err(InferError::ManifestMissing {
            path: manifest_path,
        });
    }

    let manifest =
        ArtifactManifest::load_from_path(&manifest_path).map_err(InferError::ManifestInvalid)?;
    manifest
        .validate_against_current(&weights_path)
        .map_err(InferError::ManifestInvalid)?;

    let use_cpu = env::var("BACKEND")
        .map(|v| v.eq_ignore_ascii_case("cpu"))
        .unwrap_or(false);
    let backend = if use_cpu { "cpu" } else { "wgpu" };
    let t0 = Instant::now();
    let artifact = json_escape(weights_path.to_string_lossy().as_ref());

    if use_cpu {
        run_infer::<CpuBackend>(&weights_path, backend, &artifact)?;
    } else {
        run_infer::<GpuBackend>(&weights_path, backend, &artifact)?;
    }

    let elapsed_ms = t0.elapsed().as_millis();
    eprintln!(
        r#"{{"event":"infer_done","backend":"{backend}","artifact":"{artifact}","elapsed_ms":{elapsed_ms}}}"#
    );
    Ok(())
}

fn run_infer<B: Backend>(
    weights_path: &Path,
    backend: &str,
    artifact: &str,
) -> Result<(), InferError> {
    let device = B::Device::default();
    let recorder = CompactRecorder::new();

    let t_load = Instant::now();
    let model = afterburner::model::ModelConfig::new(10)
        .init::<B>(&device)
        .load_file(weights_path, &recorder, &device)
        .map_err(|err| InferError::ArtifactLoadFailed {
            detail: err.to_string(),
            path: weights_path.to_path_buf(),
        })?;

    let load_ms = t_load.elapsed().as_millis();
    eprintln!(
        r#"{{"event":"artifact_load_ok","backend":"{backend}","artifact":"{artifact}","elapsed_ms":{load_ms}}}"#
    );

    let input = Tensor::<B, 4>::zeros([1, 1, 28, 28], &device);
    let logits = model.forward(input);
    let probs = softmax(logits, 1);
    println!("Probabilities: {probs}");
    Ok(())
}

#[derive(Debug)]
enum InferError {
    ArtifactMissing {
        path: std::path::PathBuf,
    },
    ManifestMissing {
        path: std::path::PathBuf,
    },
    ManifestInvalid(ManifestError),
    ArtifactLoadFailed {
        detail: String,
        path: std::path::PathBuf,
    },
}

fn emit_error(err: &InferError) -> ! {
    match err {
        InferError::ArtifactMissing { path } => {
            let artifact = json_escape(&path.to_string_lossy());
            eprintln!(
                r#"{{"event":"infer_error","kind":"artifact_missing","artifact":"{artifact}"}}"#
            );
        }
        InferError::ManifestMissing { path } => {
            let manifest = json_escape(&path.to_string_lossy());
            eprintln!(
                r#"{{"event":"infer_error","kind":"manifest_missing","manifest":"{manifest}"}}"#
            );
        }
        InferError::ManifestInvalid(manifest_err) => match manifest_err {
            ManifestError::Io(err) => {
                let detail = json_escape(&err.to_string());
                eprintln!(
                    r#"{{"event":"infer_error","kind":"manifest_io_error","detail":"{detail}"}}"#
                );
            }
            ManifestError::TomlParse(err) => {
                let detail = json_escape(&err.to_string());
                eprintln!(
                    r#"{{"event":"infer_error","kind":"manifest_parse_error","detail":"{detail}"}}"#
                );
            }
            ManifestError::MissingField(field) => {
                eprintln!(
                    r#"{{"event":"infer_error","kind":"manifest_missing_field","field":"{field}"}}"#
                );
            }
            ManifestError::InvalidField(field, detail) => {
                let detail = json_escape(detail);
                eprintln!(
                    r#"{{"event":"infer_error","kind":"manifest_invalid_field","field":"{field}","detail":"{detail}"}}"#
                );
            }
            ManifestError::Mismatch {
                field,
                expected,
                actual,
            } => {
                let expected = json_escape(expected);
                let actual = json_escape(actual);
                eprintln!(
                    r#"{{"event":"infer_error","kind":"manifest_mismatch","field":"{field}","expected":"{expected}","actual":"{actual}"}}"#
                );
            }
        },
        InferError::ArtifactLoadFailed { detail, path } => {
            let detail = json_escape(detail);
            let artifact = json_escape(&path.to_string_lossy());
            eprintln!(
                r#"{{"event":"infer_error","kind":"artifact_load_failed","artifact":"{artifact}","detail":"{detail}"}}"#
            );
        }
    }
    std::process::exit(2);
}

// Minimal JSON escaping to keep logs structured without adding a logging dependency.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}
