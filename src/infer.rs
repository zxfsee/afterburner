use std::env;
use std::path::{Path, PathBuf};

use burn::prelude::*;
use burn::record::CompactRecorder;

use crate::manifest::{
    ArtifactManifest, CURRENT_VERSION_FILENAME, MANIFEST_FILENAME, ManifestError, is_semver,
};
use crate::model::ModelConfig;
use crate::preprocess::mnist_image_to_tensor;

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
/// - `argv[1]` (if present) is treated as the artifact path
///
/// Pure function => unit-testable without spawning a process.
pub fn parse_weights_path_from_args<I, S>(args: I) -> PathBuf
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .nth(1)
        .map(|s| PathBuf::from(s.as_ref()))
        .unwrap_or_else(default_weights_path)
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
