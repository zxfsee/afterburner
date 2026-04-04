use std::fs;
use std::path::{Path, PathBuf};

pub const CURRENT_POINTER_PATH: &str = "artifacts/inference/current";
pub const PREVIOUS_CURRENT_VERSION_PATH: &str = "artifacts/deploy/previous_current_version.txt";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionRecord {
    pub current_pointer_path: String,
    pub previous_version_path: String,
    pub previous_version: String,
    pub promoted_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackRecord {
    pub current_pointer_path: String,
    pub previous_version_path: String,
    pub previous_version: String,
    pub restored_version: String,
}

pub fn default_current_pointer_path() -> PathBuf {
    PathBuf::from(CURRENT_POINTER_PATH)
}

pub fn default_previous_current_version_path() -> PathBuf {
    PathBuf::from(PREVIOUS_CURRENT_VERSION_PATH)
}

pub fn promote_current_pointer(
    current_pointer_path: &Path,
    previous_version_path: &Path,
    artifact_version: &str,
) -> Result<PromotionRecord, String> {
    let promoted_version = normalize_version(artifact_version, "artifact version")?;
    let previous_version = read_version_file(current_pointer_path, "current inference pointer")?;
    write_version_file(previous_version_path, previous_version.as_str()).map_err(|err| {
        format!(
            "write previous current version `{}`: {err}",
            previous_version_path.display()
        )
    })?;
    write_version_file(current_pointer_path, promoted_version.as_str()).map_err(|err| {
        format!(
            "write current inference pointer `{}`: {err}",
            current_pointer_path.display()
        )
    })?;

    Ok(PromotionRecord {
        current_pointer_path: current_pointer_path.display().to_string(),
        previous_version_path: previous_version_path.display().to_string(),
        previous_version,
        promoted_version,
    })
}

pub fn rollback_current_pointer(
    current_pointer_path: &Path,
    previous_version_path: &Path,
) -> Result<RollbackRecord, String> {
    let current_version = read_version_file(current_pointer_path, "current inference pointer")?;
    let restored_version =
        read_version_file(previous_version_path, "saved previous current version")?;
    write_version_file(current_pointer_path, restored_version.as_str()).map_err(|err| {
        format!(
            "write current inference pointer `{}`: {err}",
            current_pointer_path.display()
        )
    })?;

    Ok(RollbackRecord {
        current_pointer_path: current_pointer_path.display().to_string(),
        previous_version_path: previous_version_path.display().to_string(),
        previous_version: current_version,
        restored_version,
    })
}

fn read_version_file(path: &Path, kind: &str) -> Result<String, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("read {kind} `{}`: {err}", path.display()))?;
    normalize_version(raw.as_str(), kind)
}

fn normalize_version(raw: &str, kind: &str) -> Result<String, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(format!("{kind} must not be empty"));
    }
    Ok(value.to_string())
}

fn write_version_file(path: &Path, value: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{value}\n"))
}
