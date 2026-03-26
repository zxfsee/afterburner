use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn repo_artifact() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .join("inference")
        .join("0.1.0")
        .join("model.mpk")
}

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn infer_output_drift_summary_schema_is_explicit() {
    let schema_text = fs::read_to_string(fixture_path("infer_output_drift_summary.schema.json"))
        .expect("read infer drift summary schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("parse infer drift schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/infer-output-drift-summary/v1")
    );
    assert_eq!(schema.get("type").and_then(Value::as_str), Some("object"));
    assert_eq!(
        schema.get("additionalProperties").and_then(Value::as_bool),
        Some(false)
    );

    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("schema.required items must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "artifact",
        "artifact_version",
        "backend",
        "predicted_class",
        "top_probability",
        "margin_to_second",
        "logits_sha256",
        "probabilities_sha256",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("infer_output_drift_summary.json"),
        "reference index must mention the infer output drift summary artifact"
    );
}

#[test]
fn infer_writes_output_drift_summary_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.current_dir(tmp.path())
        .arg("infer")
        .arg(repo_artifact())
        .env("BACKEND", "cpu");
    let assert = cmd.assert().success();

    let summary_path = tmp
        .path()
        .join("artifacts")
        .join("eval")
        .join("infer_output_drift_summary.json");
    assert!(
        summary_path.exists(),
        "infer must write the drift summary artifact"
    );

    let summary_text = fs::read_to_string(&summary_path).expect("read drift summary");
    let mut summary: Value = serde_json::from_str(&summary_text).expect("parse drift summary");
    normalize_summary(&mut summary);

    let stderr =
        String::from_utf8(assert.get_output().stderr.clone()).expect("infer stderr must be utf8");
    let event = stderr_event(&stderr, "infer_output_drift_summary_written");
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be a top-level object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be an object");
    fields.insert(
        "artifact".to_string(),
        Value::from("<artifact>/artifacts/inference/0.1.0/model.mpk"),
    );
    fields.insert(
        "summary_path".to_string(),
        Value::from("artifacts/eval/infer_output_drift_summary.json"),
    );
    fields
        .get_mut("summary")
        .and_then(Value::as_object_mut)
        .expect("event summary must be an object")
        .insert(
            "artifact".to_string(),
            Value::from("<artifact>/artifacts/inference/0.1.0/model.mpk"),
        );

    let expected = summary.clone();
    let expected_event = serde_json::json!({
        "ts_ms": 0,
        "level": "info",
        "source": "infer_cli",
        "event": "infer_output_drift_summary_written",
        "fields": {
            "artifact": "<artifact>/artifacts/inference/0.1.0/model.mpk",
            "summary_path": "artifacts/eval/infer_output_drift_summary.json",
            "summary": expected
        }
    });
    assert_eq!(
        normalized_event, expected_event,
        "infer drift summary event must mirror the written artifact"
    );
}

fn stderr_event(stderr: &str, event_name: &str) -> Value {
    stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| event.get("event").and_then(Value::as_str) == Some(event_name))
        .unwrap_or_else(|| panic!("missing event `{event_name}` in stderr: {stderr}"))
}

fn normalize_summary(summary: &mut Value) {
    let object = summary
        .as_object_mut()
        .expect("drift summary must be a top-level object");
    object.insert(
        "artifact".to_string(),
        Value::from("<artifact>/artifacts/inference/0.1.0/model.mpk"),
    );
}
