use std::fs;
use std::path::Path;

use crate::command_artifacts::{JsonArtifactError, emit_json_artifact_written, write_json_value};
use serde_json::{Map, Value};

#[derive(Debug)]
pub enum ReconciliationError {
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for ReconciliationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ReconciliationError {}

impl From<std::io::Error> for ReconciliationError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for ReconciliationError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => {
                Self::Parse(format!("serialize reconciliation json: {err}"))
            }
        }
    }
}

pub fn load_json_object(
    path: &Path,
    kind: &str,
) -> Result<Map<String, Value>, ReconciliationError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        ReconciliationError::Parse(format!("parse {kind} `{}`: {err}", path.display()))
    })?;
    value.as_object().cloned().ok_or_else(|| {
        ReconciliationError::Parse(format!("{kind} `{}` must be an object", path.display()))
    })
}

pub fn read_string(
    object: &Map<String, Value>,
    key: &'static str,
    path: &Path,
    kind: &str,
) -> Result<String, ReconciliationError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            ReconciliationError::Parse(format!(
                "{kind} `{}` missing string field `{key}`",
                path.display()
            ))
        })
}

pub fn read_u64(
    object: &Map<String, Value>,
    key: &'static str,
    path: &Path,
    kind: &str,
) -> Result<u64, ReconciliationError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        ReconciliationError::Parse(format!(
            "{kind} `{}` missing integer field `{key}`",
            path.display()
        ))
    })
}

pub fn read_object(
    object: &Map<String, Value>,
    key: &'static str,
    path: &Path,
    kind: &str,
) -> Result<Map<String, Value>, ReconciliationError> {
    object
        .get(key)
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| {
            ReconciliationError::Parse(format!(
                "{kind} `{}` missing object field `{key}`",
                path.display()
            ))
        })
}

pub fn read_array(
    object: &Map<String, Value>,
    key: &'static str,
    path: &Path,
    kind: &str,
) -> Result<Value, ReconciliationError> {
    let value = object.get(key).cloned().ok_or_else(|| {
        ReconciliationError::Parse(format!("{kind} `{}` missing field `{key}`", path.display()))
    })?;
    if !value.is_array() {
        return Err(ReconciliationError::Parse(format!(
            "{kind} `{}` field `{key}` must be an array",
            path.display()
        )));
    }
    Ok(value)
}

pub fn reconciliation_status(desired: &Value, current: &Value) -> &'static str {
    if desired == current {
        "aligned"
    } else {
        "needs-update"
    }
}

pub fn load_or_init_history(path: &Path) -> Result<Value, ReconciliationError> {
    if path.exists() {
        let text = fs::read_to_string(path)?;
        let history: Value = serde_json::from_str(&text).map_err(|err| {
            ReconciliationError::Parse(format!(
                "parse reconciliation history `{}`: {err}",
                path.display()
            ))
        })?;
        history
            .get("entries")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                ReconciliationError::Parse(format!(
                    "reconciliation history `{}` must contain an `entries` array",
                    path.display()
                ))
            })?;
        Ok(history)
    } else {
        Ok(serde_json::json!({
            "schema_version": "1",
            "entries": []
        }))
    }
}

pub fn append_history_entry(history: &mut Value, entry: Value) {
    history
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .expect("history entries must be an array")
        .push(entry);
}

pub fn write_emitted_json(
    source: &str,
    event: &str,
    path_key: &str,
    path: &Path,
    payload_key: &str,
    payload: Value,
) -> Result<(), ReconciliationError> {
    write_json_value(path, &payload)?;
    emit_json_artifact_written(source, event, path_key, path, payload_key, payload);
    Ok(())
}
