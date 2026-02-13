use std::env;
use std::path::Path;
use std::time::Instant;

use afterburner::infer::{InferError, load_model, logits_from_model, parse_weights_path_from_args};
use afterburner::manifest::ManifestError;
use afterburner::observability::json_escape;
use burn::prelude::*;
use burn::tensor::activation::softmax;
use burn::{backend::ndarray::NdArray, backend::wgpu::Wgpu};

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

    let t_load = Instant::now();
    let model = load_model::<B>(weights_path, &device)?;

    let load_ms = t_load.elapsed().as_millis();
    eprintln!(
        r#"{{"event":"artifact_load_ok","backend":"{backend}","artifact":"{artifact}","elapsed_ms":{load_ms}}}"#
    );
    eprintln!(r#"{{"event":"backend_selected","backend":"{backend}"}}"#);

    let image = [[0.0f32; 28]; 28];
    let logits = logits_from_model(&model, &device, image);
    let probs = softmax(logits, 1);
    println!("Probabilities: {probs}");
    Ok(())
}

fn emit_error(err: &InferError) {
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
}

fn usage() -> &'static str {
    "usage: afterburner infer [artifact_path] [--artifact PATH]"
}
