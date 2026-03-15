use std::env;
use std::path::Path;
use std::time::Instant;

use afterburner::infer::{
    CalibrationMetadata, CalibrationMetadataLoadError, InferError,
    ensure_calibration_metadata_compatible, load_calibration_metadata, load_model,
    logits_from_model, manifest_path_for_weights, parse_weights_path_from_args, validate_artifacts,
};
use afterburner::manifest::ManifestError;
use afterburner::manifest::{ArtifactManifest, ManifestPrecision};
use afterburner::observability::emit_event;
use burn::prelude::*;
use burn::tensor::activation::softmax;
use burn::{backend::ndarray::NdArray, backend::wgpu::Wgpu};
use serde_json::{Value, json};

type GpuBackend = Wgpu<f32, i32>;
type CpuBackend = NdArray<f32>;

const BACKEND_CPU: &str = "cpu";
const BACKEND_WGPU: &str = "wgpu";
const RUNTIME_SUPPORTED_BACKENDS: [&str; 2] = [BACKEND_CPU, BACKEND_WGPU];

const ADAPTER_REGISTRY_SCHEMA_FIXTURE: &str =
    include_str!("../fixtures/framework_adapter_registry.schema.json");
const ADAPTER_REGISTRY_SUPPORTED_PAIRS_KEY: &str = "x-supported-adapter-version-pairs";

struct AdapterRegistryMetadata {
    schema_version: String,
    adapters: Vec<AdapterRegistryEntry>,
}

struct AdapterRegistryEntry {
    adapter: String,
    version: String,
}

#[derive(Debug)]
enum CmdInferError {
    Infer(InferError),
    AdapterRegistryInvalid {
        detail: String,
    },
    UnsupportedAdapterVersion {
        backend: String,
        artifact_version: String,
        schema_version: String,
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
    let adapter_registry = load_adapter_registry_metadata()
        .map_err(|detail| CmdInferError::AdapterRegistryInvalid { detail })?;
    let manifest = load_artifact_manifest(&weights_path).map_err(CmdInferError::Infer)?;
    let artifact_version = manifest.artifact_version.clone();
    let precision = manifest.precision.clone();
    let calibration = match load_calibration_metadata(&weights_path) {
        Ok(calibration) => calibration,
        Err(err) => {
            emit_calibration_metadata_invalid(&err);
            None
        }
    };
    let calibration = match calibration {
        Some(calibration) => match ensure_calibration_metadata_compatible(
            &weights_path,
            calibration,
            artifact_version.as_str(),
        ) {
            Ok(calibration) => Some(calibration),
            Err(err) => {
                emit_calibration_metadata_invalid(&err);
                None
            }
        },
        None => None,
    };
    if !runtime_supported_backends()
        .iter()
        .any(|candidate| *candidate == backend.as_str())
    {
        return Err(CmdInferError::UnsupportedAdapterVersion {
            backend,
            artifact_version,
            schema_version: adapter_registry.schema_version,
        });
    }
    validate_backend_selection(
        &adapter_registry,
        backend.as_str(),
        artifact_version.as_str(),
    )?;

    match backend.as_str() {
        BACKEND_CPU => {
            run_infer::<CpuBackend>(
                &weights_path,
                backend.as_str(),
                artifact.as_str(),
                &precision,
                calibration.as_ref(),
            )
            .map_err(CmdInferError::Infer)?;
        }
        BACKEND_WGPU => {
            run_infer::<GpuBackend>(
                &weights_path,
                backend.as_str(),
                artifact.as_str(),
                &precision,
                calibration.as_ref(),
            )
            .map_err(CmdInferError::Infer)?;
        }
        _ => {
            return Err(CmdInferError::UnsupportedAdapterVersion {
                backend,
                artifact_version,
                schema_version: adapter_registry.schema_version,
            });
        }
    }

    let duration_ms = t0.elapsed().as_millis();
    let mut fields = json!({
        "backend": backend,
        "artifact": artifact,
        "artifact_version": artifact_version,
        "batch_size": 1,
        "duration_ms": duration_ms
    });
    include_precision_fields(&mut fields, &precision);
    include_calibration_fields(&mut fields, calibration.as_ref());
    emit_event("info", "infer_cli", "infer_done", fields);
    Ok(())
}

fn resolve_backend() -> String {
    let default_backend = BACKEND_WGPU;
    env::var("BACKEND")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default_backend.to_string())
}

pub(crate) fn runtime_supported_backends() -> &'static [&'static str] {
    &RUNTIME_SUPPORTED_BACKENDS
}

fn load_artifact_manifest(weights_path: &Path) -> Result<ArtifactManifest, InferError> {
    validate_artifacts(weights_path)?;
    let manifest_path = manifest_path_for_weights(weights_path);
    ArtifactManifest::load_from_path(&manifest_path).map_err(InferError::ManifestInvalid)
}

fn load_adapter_registry_metadata() -> Result<AdapterRegistryMetadata, String> {
    let schema: Value = serde_json::from_str(ADAPTER_REGISTRY_SCHEMA_FIXTURE)
        .map_err(|err| format!("failed to parse adapter registry schema fixture json: {err}"))?;
    let root = schema
        .as_object()
        .ok_or_else(|| "adapter registry schema fixture must be a json object".to_string())?;

    let properties = root
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| "adapter registry schema fixture missing `properties` object".to_string())?;
    let schema_version = properties
        .get("schema_version")
        .and_then(|v| v.get("const"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            "adapter registry schema fixture missing `properties.schema_version.const`".to_string()
        })?
        .to_string();

    let adapters = root
        .get(ADAPTER_REGISTRY_SUPPORTED_PAIRS_KEY)
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!(
                "adapter registry schema fixture missing `{ADAPTER_REGISTRY_SUPPORTED_PAIRS_KEY}` array"
            )
        })?;

    if adapters.is_empty() {
        return Err(format!(
            "adapter registry schema fixture `{ADAPTER_REGISTRY_SUPPORTED_PAIRS_KEY}` must not be empty"
        ));
    }

    let adapters = adapters
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let object = entry.as_object().ok_or_else(|| {
                format!(
                    "adapter registry schema fixture `{ADAPTER_REGISTRY_SUPPORTED_PAIRS_KEY}[{idx}]` must be an object"
                )
            })?;

            let adapter = object
                .get("adapter")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    format!(
                        "adapter registry schema fixture `{ADAPTER_REGISTRY_SUPPORTED_PAIRS_KEY}[{idx}].adapter` must be a string"
                    )
                })?
                .to_string();
            if adapter.is_empty() {
                return Err(format!(
                    "adapter registry schema fixture `{ADAPTER_REGISTRY_SUPPORTED_PAIRS_KEY}[{idx}].adapter` must not be empty"
                ));
            }

            let version = object
                .get("version")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    format!(
                        "adapter registry schema fixture `{ADAPTER_REGISTRY_SUPPORTED_PAIRS_KEY}[{idx}].version` must be a string"
                    )
                })?
                .to_string();
            if version.is_empty() {
                return Err(format!(
                    "adapter registry schema fixture `{ADAPTER_REGISTRY_SUPPORTED_PAIRS_KEY}[{idx}].version` must not be empty"
                ));
            }

            Ok(AdapterRegistryEntry { adapter, version })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AdapterRegistryMetadata {
        schema_version,
        adapters,
    })
}

fn validate_backend_selection(
    adapter_registry: &AdapterRegistryMetadata,
    backend: &str,
    artifact_version: &str,
) -> Result<(), CmdInferError> {
    let is_supported = adapter_registry
        .adapters
        .iter()
        .any(|entry| entry.adapter == backend && entry.version == artifact_version);

    if is_supported {
        return Ok(());
    }

    Err(CmdInferError::UnsupportedAdapterVersion {
        backend: backend.to_string(),
        artifact_version: artifact_version.to_string(),
        schema_version: adapter_registry.schema_version.clone(),
    })
}

fn run_infer<B: Backend>(
    weights_path: &Path,
    backend: &str,
    artifact: &str,
    precision: &ManifestPrecision,
    calibration: Option<&CalibrationMetadata>,
) -> Result<(), InferError> {
    let device = B::Device::default();

    let t_load = Instant::now();
    let model = load_model::<B>(weights_path, &device)?;

    let load_ms = t_load.elapsed().as_millis();
    let mut fields = json!({"backend": backend, "artifact": artifact, "elapsed_ms": load_ms});
    include_precision_fields(&mut fields, precision);
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
    let probabilities = softmax(logits.clone(), 1);
    let logits = extract_single_row(logits, "logits")?;
    let probabilities = extract_single_row(probabilities, "probabilities")?;
    println!(
        "{}",
        json!({"logits": [logits], "probabilities": [probabilities]})
    );
    Ok(())
}

fn extract_single_row<B: Backend>(
    tensor: Tensor<B, 2>,
    field: &'static str,
) -> Result<Vec<f32>, InferError> {
    let data = tensor.to_data();
    let values: Vec<f32> = data.iter().collect();
    if values.len() != 10 {
        return Err(InferError::ArtifactLoadFailed {
            detail: format!(
                "infer output shape invalid for {field}: expected 10 values, got {}",
                values.len()
            ),
            path: Path::new(field).to_path_buf(),
        });
    }

    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(InferError::ArtifactLoadFailed {
                detail: format!(
                    "infer output numerics invalid for {field}[{index}]: value is not finite ({value})"
                ),
                path: Path::new(field).to_path_buf(),
            });
        }
    }

    Ok(values)
}

fn include_precision_fields(fields: &mut Value, precision: &ManifestPrecision) {
    let Some(object) = fields.as_object_mut() else {
        return;
    };

    object.insert(
        "precision".to_string(),
        json!({
            "weights_dtype": precision.weights_dtype,
            "activation_dtype": precision.activation_dtype,
            "quantization": precision.quantization,
        }),
    );
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

fn emit_calibration_metadata_invalid(err: &CalibrationMetadataLoadError) {
    emit_event(
        "error",
        "infer_cli",
        "calibration_metadata_invalid",
        json!({
            "kind": err.kind(),
            "metadata": err.path().to_string_lossy().to_string(),
            "detail": err.detail(),
        }),
    );
}

fn emit_error(err: &CmdInferError) {
    match err {
        CmdInferError::Infer(err) => emit_infer_error(err),
        CmdInferError::AdapterRegistryInvalid { detail } => {
            emit_event(
                "error",
                "infer_cli",
                "infer_error",
                json!({
                    "kind": "adapter_registry_invalid",
                    "detail": detail,
                }),
            );
        }
        CmdInferError::UnsupportedAdapterVersion {
            backend,
            artifact_version,
            schema_version,
        } => {
            emit_event(
                "error",
                "infer_cli",
                "infer_error",
                json!({
                    "kind": "adapter_registry_unsupported",
                    "backend": backend,
                    "artifact_version": artifact_version,
                    "schema_version": schema_version,
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
