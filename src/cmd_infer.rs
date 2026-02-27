use std::env;
use std::path::Path;
use std::time::Instant;

use afterburner::infer::{
    CalibrationMetadata, InferError, load_calibration_metadata, load_model, logits_from_model,
    manifest_path_for_weights, parse_weights_path_from_args, validate_artifacts,
};
use afterburner::manifest::ArtifactManifest;
use afterburner::manifest::ManifestError;
use afterburner::observability::emit_event;
use burn::prelude::*;
use burn::tensor::activation::softmax;
use burn::{backend::ndarray::NdArray, backend::wgpu::Wgpu};
use serde_json::{Value, json};

type GpuBackend = Wgpu<f32, i32>;
type CpuBackend = NdArray<f32>;

const ADAPTER_REGISTRY_SCHEMA_VERSION: &str = "1";

const ADAPTER_REGISTRY: AdapterRegistryMetadata = AdapterRegistryMetadata {
    schema_version: ADAPTER_REGISTRY_SCHEMA_VERSION,
    adapters: &[
        AdapterRegistryEntry {
            adapter: "wgpu",
            version: "0.1.0",
        },
        AdapterRegistryEntry {
            adapter: "cpu",
            version: "0.1.0",
        },
    ],
};

struct AdapterRegistryMetadata {
    schema_version: &'static str,
    adapters: &'static [AdapterRegistryEntry],
}

struct AdapterRegistryEntry {
    adapter: &'static str,
    version: &'static str,
}

#[derive(Debug)]
enum CmdInferError {
    Infer(InferError),
    UnsupportedAdapterVersion {
        backend: String,
        artifact_version: String,
    },
}

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

fn run_inner<I>(args: I) -> Result<(), CmdInferError>
where
    I: Iterator<Item = String>,
{
    let weights_path =
        parse_weights_path_from_args(std::iter::once("infer".to_string()).chain(args));

    let backend = resolve_backend();
    let t0 = Instant::now();
    let artifact = weights_path.to_string_lossy().to_string();
    let calibration = load_calibration_metadata(&weights_path);
    let artifact_version = load_artifact_version(&weights_path).map_err(CmdInferError::Infer)?;
    validate_backend_selection(backend.as_str(), artifact_version.as_str())?;

    match backend.as_str() {
        "cpu" => {
            run_infer::<CpuBackend>(
                &weights_path,
                backend.as_str(),
                artifact.as_str(),
                calibration.as_ref(),
            )
            .map_err(CmdInferError::Infer)?;
        }
        "wgpu" => {
            run_infer::<GpuBackend>(
                &weights_path,
                backend.as_str(),
                artifact.as_str(),
                calibration.as_ref(),
            )
            .map_err(CmdInferError::Infer)?;
        }
        _ => {
            return Err(CmdInferError::UnsupportedAdapterVersion {
                backend,
                artifact_version,
            });
        }
    }

    let elapsed_ms = t0.elapsed().as_millis();
    let mut fields = json!({"backend": backend, "artifact": artifact, "elapsed_ms": elapsed_ms, "artifact_version": artifact_version});
    include_calibration_fields(&mut fields, calibration.as_ref());
    emit_event("info", "infer_cli", "infer_done", fields);
    Ok(())
}

fn resolve_backend() -> String {
    env::var("BACKEND")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "wgpu".to_string())
}

fn load_artifact_version(weights_path: &Path) -> Result<String, InferError> {
    validate_artifacts(weights_path)?;
    let manifest_path = manifest_path_for_weights(weights_path);
    let manifest =
        ArtifactManifest::load_from_path(&manifest_path).map_err(InferError::ManifestInvalid)?;
    Ok(manifest.artifact_version)
}

fn validate_backend_selection(backend: &str, artifact_version: &str) -> Result<(), CmdInferError> {
    let is_supported = ADAPTER_REGISTRY
        .adapters
        .iter()
        .any(|entry| entry.adapter == backend && entry.version == artifact_version);

    if is_supported {
        return Ok(());
    }

    Err(CmdInferError::UnsupportedAdapterVersion {
        backend: backend.to_string(),
        artifact_version: artifact_version.to_string(),
    })
}

fn run_infer<B: Backend>(
    weights_path: &Path,
    backend: &str,
    artifact: &str,
    calibration: Option<&CalibrationMetadata>,
) -> Result<(), InferError> {
    let device = B::Device::default();

    let t_load = Instant::now();
    let model = load_model::<B>(weights_path, &device)?;

    let load_ms = t_load.elapsed().as_millis();
    let mut fields = json!({"backend": backend, "artifact": artifact, "elapsed_ms": load_ms});
    include_calibration_fields(&mut fields, calibration);
    emit_event("info", "infer_cli", "artifact_load_ok", fields);
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

fn include_calibration_fields(fields: &mut Value, calibration: Option<&CalibrationMetadata>) {
    let Some(calibration) = calibration else {
        return;
    };
    let Some(object) = fields.as_object_mut() else {
        return;
    };

    object.insert(
        "calibration".to_string(),
        json!({
            "schema_version": calibration.schema_version.to_string(),
            "calibration_artifact": calibration.calibration_artifact,
            "artifact_version": calibration.artifact_version,
            "method": calibration.method,
            "created_at_unix_ms": calibration.created_at_unix_ms,
        }),
    );
}

fn emit_error(err: &CmdInferError) {
    match err {
        CmdInferError::Infer(err) => emit_infer_error(err),
        CmdInferError::UnsupportedAdapterVersion {
            backend,
            artifact_version,
        } => {
            emit_event(
                "error",
                "infer_cli",
                "infer_error",
                json!({
                    "kind": "adapter_registry_unsupported",
                    "backend": backend,
                    "artifact_version": artifact_version,
                    "schema_version": ADAPTER_REGISTRY.schema_version,
                }),
            );
        }
    }
}

fn emit_infer_error(err: &InferError) {
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
