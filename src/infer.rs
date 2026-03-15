use std::env;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use burn::prelude::*;
use burn::record::CompactRecorder;
use serde_json::Value;

use crate::manifest::{
    ArtifactManifest, CURRENT_VERSION_FILENAME, MANIFEST_FILENAME, ManifestError, is_semver,
};
use crate::model::ModelConfig;
use crate::preprocess::mnist_image_to_tensor;

pub const CALIBRATION_METADATA_FILENAME: &str = "calibration_artifact_metadata.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalibrationMetadata {
    pub schema_version: u8,
    pub calibration_artifact: String,
    pub artifact_version: String,
    pub method: String,
    pub created_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalibrationMetadataLoadError {
    Read { path: PathBuf, detail: String },
    Invalid { path: PathBuf, detail: String },
    Incompatible { path: PathBuf, detail: String },
}

impl CalibrationMetadataLoadError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Read { .. } => "read_error",
            Self::Invalid { .. } => "parse_error",
            Self::Incompatible { .. } => "artifact_version_mismatch",
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::Read { path, .. }
            | Self::Invalid { path, .. }
            | Self::Incompatible { path, .. } => path.as_path(),
        }
    }

    pub fn detail(&self) -> &str {
        match self {
            Self::Read { detail, .. }
            | Self::Invalid { detail, .. }
            | Self::Incompatible { detail, .. } => detail.as_str(),
        }
    }
}

/// Default inference artifact path (ADR-002).
pub fn default_weights_path() -> PathBuf {
    let inference_root = PathBuf::from("artifacts/inference");
    if let Some(version) = read_current_version(&inference_root) {
        return inference_root.join(version).join("model.mpk");
    }
    inference_root.join("model.mpk")
}

/// Parse optional CLI args into a weights path.
/// - `argv[0]` is ignored
/// - `--artifact <path>` or `--artifact=<path>` is preferred
/// - `argv[1]` positional path remains supported for compatibility
///
/// Pure function => unit-testable without spawning a process.
pub fn parse_weights_path_from_args<I, S>(args: I) -> PathBuf
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut args = args.into_iter().skip(1).map(|arg| arg.as_ref().to_string());

    while let Some(arg) = args.next() {
        if let Some(path) = arg.strip_prefix("--artifact=") {
            if !path.is_empty() {
                return PathBuf::from(path);
            }
            return default_weights_path();
        }

        if arg == "--artifact" {
            return args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(default_weights_path);
        }

        if !arg.starts_with("--") {
            return PathBuf::from(arg);
        }
    }

    default_weights_path()
}

/// Parse weights path from the actual process arguments.
pub fn parse_weights_path() -> PathBuf {
    parse_weights_path_from_args(env::args())
}

/// Derive the manifest path for a given weights file.
pub fn manifest_path_for_weights(weights_path: &Path) -> PathBuf {
    let parent = weights_path.parent().unwrap_or_else(|| Path::new("."));
    parent.join(MANIFEST_FILENAME)
}

/// Derive calibration metadata path for a given weights file.
pub fn calibration_metadata_path_for_weights(weights_path: &Path) -> PathBuf {
    let parent = weights_path.parent().unwrap_or_else(|| Path::new("."));
    parent.join(CALIBRATION_METADATA_FILENAME)
}

/// Load optional calibration metadata sidecar from the artifact directory.
///
/// Returns `Ok(None)` when the sidecar is absent.
pub fn load_calibration_metadata(
    weights_path: &Path,
) -> Result<Option<CalibrationMetadata>, CalibrationMetadataLoadError> {
    let metadata_path = calibration_metadata_path_for_weights(weights_path);
    let contents = match std::fs::read_to_string(&metadata_path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(CalibrationMetadataLoadError::Read {
                path: metadata_path,
                detail: err.to_string(),
            });
        }
    };

    let metadata = parse_calibration_metadata(contents.as_str()).map_err(|detail| {
        CalibrationMetadataLoadError::Invalid {
            path: metadata_path,
            detail,
        }
    })?;
    Ok(Some(metadata))
}

pub fn ensure_calibration_metadata_compatible(
    weights_path: &Path,
    metadata: CalibrationMetadata,
    runtime_artifact_version: &str,
) -> Result<CalibrationMetadata, CalibrationMetadataLoadError> {
    if metadata.artifact_version == runtime_artifact_version {
        return Ok(metadata);
    }

    Err(CalibrationMetadataLoadError::Incompatible {
        path: calibration_metadata_path_for_weights(weights_path),
        detail: format!(
            "calibration metadata artifact_version `{}` does not match runtime artifact_version `{runtime_artifact_version}`",
            metadata.artifact_version
        ),
    })
}

fn parse_calibration_metadata(contents: &str) -> Result<CalibrationMetadata, String> {
    let value: Value =
        serde_json::from_str(contents).map_err(|err| format!("invalid json: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "metadata must be a JSON object".to_string())?;

    let required_fields = [
        "schema_version",
        "calibration_artifact",
        "artifact_version",
        "method",
        "created_at_unix_ms",
    ];

    for field in required_fields {
        if !object.contains_key(field) {
            return Err(format!("missing required field `{field}`"));
        }
    }

    for field in object.keys() {
        if !required_fields.iter().any(|allowed| allowed == field) {
            return Err(format!("unknown field `{field}`"));
        }
    }

    let schema_version = object
        .get("schema_version")
        .and_then(Value::as_str)
        .ok_or_else(|| "schema_version must be a string".to_string())?;
    if schema_version != "1" {
        return Err("schema_version must be \"1\"".to_string());
    }

    let calibration_artifact = object
        .get("calibration_artifact")
        .and_then(Value::as_str)
        .ok_or_else(|| "calibration_artifact must be a string".to_string())?;
    if calibration_artifact.is_empty() {
        return Err("calibration_artifact must be non-empty".to_string());
    }

    let artifact_version = object
        .get("artifact_version")
        .and_then(Value::as_str)
        .ok_or_else(|| "artifact_version must be a string".to_string())?;
    if artifact_version.is_empty() {
        return Err("artifact_version must be non-empty".to_string());
    }

    let method = object
        .get("method")
        .and_then(Value::as_str)
        .ok_or_else(|| "method must be a string".to_string())?;
    if method.is_empty() {
        return Err("method must be non-empty".to_string());
    }

    let created_at_unix_ms = object
        .get("created_at_unix_ms")
        .and_then(Value::as_u64)
        .ok_or_else(|| "created_at_unix_ms must be a non-negative integer".to_string())?;

    Ok(CalibrationMetadata {
        schema_version: 1,
        calibration_artifact: calibration_artifact.to_string(),
        artifact_version: artifact_version.to_string(),
        method: method.to_string(),
        created_at_unix_ms,
    })
}

fn read_current_version(inference_root: &Path) -> Option<String> {
    let path = inference_root.join(CURRENT_VERSION_FILENAME);
    let contents = std::fs::read_to_string(path).ok()?;
    let version = contents.trim();
    if version.is_empty() || !is_semver(version) {
        return None;
    }
    Some(version.to_string())
}

#[derive(Debug)]
pub enum InferError {
    ArtifactMissing { path: PathBuf },
    ManifestMissing { path: PathBuf },
    ManifestInvalid(ManifestError),
    ArtifactLoadFailed { detail: String, path: PathBuf },
}

pub fn validate_artifacts(weights_path: &Path) -> Result<(), InferError> {
    if !weights_path.exists() {
        return Err(InferError::ArtifactMissing {
            path: weights_path.to_path_buf(),
        });
    }

    let manifest_path = manifest_path_for_weights(weights_path);
    if !manifest_path.exists() {
        return Err(InferError::ManifestMissing {
            path: manifest_path,
        });
    }

    let manifest =
        ArtifactManifest::load_from_path(&manifest_path).map_err(InferError::ManifestInvalid)?;
    manifest
        .validate_against_current(weights_path)
        .map_err(InferError::ManifestInvalid)?;
    Ok(())
}

pub fn load_model<B: Backend>(
    weights_path: &Path,
    device: &B::Device,
) -> Result<crate::model::Model<B>, InferError> {
    validate_artifacts(weights_path)?;
    let recorder = CompactRecorder::new();
    ModelConfig::new(10)
        .init::<B>(device)
        .load_file(weights_path, &recorder, device)
        .map_err(|err| InferError::ArtifactLoadFailed {
            detail: err.to_string(),
            path: weights_path.to_path_buf(),
        })
}

pub fn logits_from_model<B: Backend>(
    model: &crate::model::Model<B>,
    device: &B::Device,
    image: [[f32; 28]; 28],
) -> Tensor<B, 2> {
    let tensor = mnist_image_to_tensor::<B>(image, device);
    let input = Tensor::stack(vec![tensor], 0);
    model.forward(input)
}
