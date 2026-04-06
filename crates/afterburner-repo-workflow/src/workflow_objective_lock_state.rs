use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::queue_workflow_metadata::{is_allowed_path, normalize_candidate_path};
use crate::workflow_objective_lock_cli::Objective;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectiveLock {
    pub objective: Objective,
    pub expected_action: Option<String>,
    pub allowed_paths: Vec<String>,
    pub forbidden_paths: Vec<String>,
    pub source_title: Option<String>,
    pub carried_worktree_paths: BTreeMap<String, String>,
}

#[derive(Debug)]
pub enum ObjectiveLockError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Parse(String),
}

impl std::fmt::Display for ObjectiveLockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ObjectiveLockError {}

impl From<std::io::Error> for ObjectiveLockError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for ObjectiveLockError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

pub fn write_lock(path: &Path, lock: &ObjectiveLock) -> Result<(), ObjectiveLockError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut payload = json!({
        "schema_version": 1,
        "objective": lock.objective.as_str(),
        "allowed_paths": lock.allowed_paths.clone(),
        "forbidden_paths": lock.forbidden_paths.clone(),
    });
    if let Some(expected_action) = &lock.expected_action {
        payload["expected_action"] = Value::String(expected_action.clone());
    }
    if let Some(source_title) = &lock.source_title {
        payload["source_title"] = Value::String(source_title.clone());
    }
    if !lock.carried_worktree_paths.is_empty() {
        payload["carried_worktree_paths"] = json!(lock.carried_worktree_paths);
    }
    fs::write(path, serde_json::to_vec_pretty(&payload)?)?;
    Ok(())
}

pub fn read_lock(path: &Path) -> Result<ObjectiveLock, ObjectiveLockError> {
    let raw = fs::read_to_string(path).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            ObjectiveLockError::Parse(format!(
                "objective lock not found at `{}`; pin one first",
                path.display()
            ))
        } else {
            ObjectiveLockError::Io(err)
        }
    })?;
    let value: Value = serde_json::from_str(&raw)?;

    let objective =
        Objective::parse(string_field(&value, "objective")?).map_err(ObjectiveLockError::Parse)?;
    let expected_action = optional_string_field(&value, "expected_action");
    let source_title = optional_string_field(&value, "source_title");
    let allowed_paths = string_array_field(&value, "allowed_paths")?;
    let forbidden_paths = string_array_field(&value, "forbidden_paths")?;
    let carried_worktree_paths = optional_string_map_field(&value, "carried_worktree_paths")?;

    Ok(ObjectiveLock {
        objective,
        expected_action,
        allowed_paths,
        forbidden_paths,
        source_title,
        carried_worktree_paths,
    })
}

pub fn validate_action(
    lock: &ObjectiveLock,
    action: Option<&str>,
) -> Result<(), ObjectiveLockError> {
    match (&lock.expected_action, action) {
        (Some(expected), Some(actual)) if expected != actual => {
            Err(ObjectiveLockError::Parse(format!(
                "objective lock expects action `{expected}`, got `{actual}` for objective `{}`",
                lock.objective.as_str()
            )))
        }
        (Some(expected), None) => Err(ObjectiveLockError::Parse(format!(
            "objective lock expects action `{expected}` for objective `{}`",
            lock.objective.as_str()
        ))),
        _ => Ok(()),
    }
}

pub fn validate_paths(lock: &ObjectiveLock, paths: &[String]) -> Result<(), ObjectiveLockError> {
    let mut rejected = Vec::new();
    for path in paths.iter().map(|path| normalize_candidate_path(path)) {
        if !is_allowed_path(&lock.allowed_paths, &path) {
            rejected.push(path);
        }
    }

    if rejected.is_empty() {
        return Ok(());
    }

    let scope_source = lock
        .source_title
        .as_ref()
        .map(|title| format!(" from top TODO `{title}`"))
        .unwrap_or_default();
    let mut message = format!(
        "objective `{}`{} forbids repo mutations outside {:?}; rejected paths: {:?}",
        lock.objective.as_str(),
        scope_source,
        lock.allowed_paths,
        rejected
    );
    if lock.objective == Objective::ExecuteTopItem
        && rejected
            .iter()
            .all(|path| path == "Cargo.toml" || path == "CHANGELOG.md")
    {
        message.push_str(
            "\nif you are repairing stale top TODO scope metadata, run `just queue-fix-top-scope` first",
        );
    }
    Err(ObjectiveLockError::Parse(message))
}

pub fn validate_worktree_paths(
    lock: &ObjectiveLock,
    repo_root: &Path,
    paths: &[String],
) -> Result<(), ObjectiveLockError> {
    let filtered = paths
        .iter()
        .filter_map(|path| {
            let normalized = normalize_candidate_path(path);
            match lock.carried_worktree_paths.get(&normalized) {
                Some(expected_hash)
                    if file_sha256_hex(&repo_root.join(&normalized))
                        .is_ok_and(|current_hash| current_hash == *expected_hash) =>
                {
                    None
                }
                _ => Some(normalized),
            }
        })
        .collect::<Vec<_>>();
    validate_paths(lock, &filtered)
}

pub(crate) fn hash_path(path: &Path) -> Result<String, ObjectiveLockError> {
    file_sha256_hex(path)
}

fn file_sha256_hex(path: &Path) -> Result<String, ObjectiveLockError> {
    let bytes = fs::read(path)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str, ObjectiveLockError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| ObjectiveLockError::Parse(format!("objective lock missing `{field}`")))
}

fn optional_string_field(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn string_array_field(value: &Value, field: &str) -> Result<Vec<String>, ObjectiveLockError> {
    let Some(array) = value.get(field).and_then(Value::as_array) else {
        return Err(ObjectiveLockError::Parse(format!(
            "objective lock missing `{field}`"
        )));
    };
    array
        .iter()
        .map(|entry| {
            entry.as_str().map(ToString::to_string).ok_or_else(|| {
                ObjectiveLockError::Parse(format!(
                    "objective lock field `{field}` must contain only strings"
                ))
            })
        })
        .collect()
}

fn optional_string_map_field(
    value: &Value,
    field: &str,
) -> Result<BTreeMap<String, String>, ObjectiveLockError> {
    let Some(object) = value.get(field) else {
        return Ok(BTreeMap::new());
    };
    let Some(map) = object.as_object() else {
        return Err(ObjectiveLockError::Parse(format!(
            "objective lock field `{field}` must be an object"
        )));
    };
    map.iter()
        .map(|(key, value)| {
            value
                .as_str()
                .map(|raw| (key.clone(), raw.to_string()))
                .ok_or_else(|| {
                    ObjectiveLockError::Parse(format!(
                        "objective lock field `{field}` must map to string hashes"
                    ))
                })
        })
        .collect()
}
