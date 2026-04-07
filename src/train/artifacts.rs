use std::io;
use std::path::{Path, PathBuf};

use crate::{
    infer::{
        CALIBRATION_METADATA_FILENAME, CalibrationMetadata, load_calibration_metadata_from_path,
    },
    manifest::{ArtifactManifest, CURRENT_VERSION_FILENAME, compute_sha256_hex},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedCalibrationSidecar {
    pub path: PathBuf,
    pub metadata: CalibrationMetadata,
}

pub fn artifact_dirs(root: &Path) -> (PathBuf, PathBuf) {
    let train_dir = root.join("train");
    let inference_root = root.join("inference");
    (train_dir, inference_root)
}

pub fn checkpoint_runtime_root(train_dir: &Path, checkpoint_group: Option<&str>) -> PathBuf {
    match checkpoint_group {
        Some(group) => train_dir.join("checkpoint_groups").join(group),
        None => train_dir.to_path_buf(),
    }
}

pub fn checkpoint_root(runtime_root: &Path) -> PathBuf {
    runtime_root.join("checkpoint")
}

pub fn export_inference_artifact(
    train_model_path: &Path,
    inference_root: &Path,
    artifact_version: &str,
) -> io::Result<PathBuf> {
    let version_dir = inference_version_dir(inference_root, artifact_version);
    std::fs::create_dir_all(&version_dir)?;
    let out = version_dir.join("model.mpk");
    std::fs::copy(train_model_path, &out)?;
    let checksum = compute_sha256_hex(&out)?;
    let manifest = ArtifactManifest::for_current("model.mpk", artifact_version, checksum);
    manifest.write_to_dir(&version_dir)?;
    write_current_version(inference_root, artifact_version)?;
    Ok(out)
}

pub fn copy_calibration_metadata_sidecar_if_present(
    train_dir: &Path,
    inference_dir: &Path,
    artifact_version: &str,
) -> io::Result<Option<ExportedCalibrationSidecar>> {
    let source = train_dir.join(CALIBRATION_METADATA_FILENAME);
    if !source.exists() {
        return Ok(None);
    }

    let metadata = load_calibration_metadata_from_path(&source).map_err(|err| {
        io::Error::other(format!(
            "invalid calibration metadata sidecar at {}: {}",
            err.path().display(),
            err.detail()
        ))
    })?;
    if metadata.artifact_version != artifact_version {
        return Err(io::Error::other(format!(
            "calibration metadata artifact_version `{}` does not match export artifact_version `{artifact_version}`",
            metadata.artifact_version
        )));
    }

    let destination = inference_dir.join(CALIBRATION_METADATA_FILENAME);
    std::fs::copy(&source, &destination)?;
    Ok(Some(ExportedCalibrationSidecar {
        path: destination,
        metadata,
    }))
}

pub fn inference_version_dir(inference_root: &Path, artifact_version: &str) -> PathBuf {
    inference_root.join(artifact_version)
}

fn write_current_version(inference_root: &Path, artifact_version: &str) -> io::Result<PathBuf> {
    let path = inference_root.join(CURRENT_VERSION_FILENAME);
    std::fs::write(&path, format!("{artifact_version}\n"))?;
    Ok(path)
}
