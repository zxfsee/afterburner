use std::fs;
use std::path::Path;

use crate::observability::emit_event;
use serde_json::{Map, Value};

#[derive(Debug)]
pub enum JsonArtifactError {
    Io(std::io::Error),
    Serialize(serde_json::Error),
}

impl std::fmt::Display for JsonArtifactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Serialize(err) => write!(f, "serialize json: {err}"),
        }
    }
}

impl std::error::Error for JsonArtifactError {}

impl From<std::io::Error> for JsonArtifactError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for JsonArtifactError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialize(value)
    }
}

pub fn write_json_value(path: &Path, value: &Value) -> Result<(), JsonArtifactError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(value)?;
    fs::write(path, text)?;
    Ok(())
}

pub fn path_payload_fields(
    path_key: &str,
    path: &Path,
    payload_key: &str,
    payload: Value,
) -> Value {
    path_payload_fields_with_extra(path_key, path, payload_key, payload, Map::new())
}

pub fn path_payload_fields_with_extra(
    path_key: &str,
    path: &Path,
    payload_key: &str,
    payload: Value,
    mut extra_fields: Map<String, Value>,
) -> Value {
    extra_fields.insert(
        path_key.to_string(),
        Value::from(path.display().to_string()),
    );
    extra_fields.insert(payload_key.to_string(), payload);
    Value::Object(extra_fields)
}

pub fn emit_json_artifact_written(
    source: &str,
    event: &str,
    path_key: &str,
    path: &Path,
    payload_key: &str,
    payload: Value,
) {
    emit_event(
        "info",
        source,
        event,
        path_payload_fields(path_key, path, payload_key, payload),
    );
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{path_payload_fields, write_json_value};
    use serde_json::json;

    #[test]
    fn write_json_value_writes_pretty_json_and_parent_dirs() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("nested").join("artifact.json");
        let value = json!({"schema_version":"1","ok":true});

        write_json_value(&path, &value).expect("write artifact json");

        let text = fs::read_to_string(&path).expect("read artifact json");
        assert!(text.contains("\n"), "artifact json must be pretty-printed");
        let roundtrip: serde_json::Value =
            serde_json::from_str(&text).expect("parse written artifact");
        assert_eq!(roundtrip, value);
    }

    #[test]
    fn path_payload_fields_build_standard_path_and_payload_shape() {
        let value = path_payload_fields(
            "profile_path",
            std::path::Path::new("artifacts/train/profile.json"),
            "profile",
            json!({"schema_version":"1"}),
        );

        assert_eq!(
            value,
            json!({
                "profile_path": "artifacts/train/profile.json",
                "profile": {"schema_version":"1"}
            })
        );
    }
}
