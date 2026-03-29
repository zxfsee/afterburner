use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[path = "fixture_support.rs"]
mod fixture_support;

#[test]
fn infer_fails_fast_on_missing_artifact() {
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("infer").arg("does-not-exist.mpk");
    cmd.assert().failure().code(2).stderr(
        predicate::str::contains(r#""event":"infer_error""#)
            .and(predicate::str::contains(r#""kind":"artifact_missing""#))
            .and(predicate::str::contains(r#""artifact":"#))
            .and(predicate::str::contains("does-not-exist.mpk")),
    );
}

#[test]
fn infer_done_event_matches_fixture_contract() {
    let (_artifact_dir, artifact) = fixture_support::build_runtime_model_artifact();

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("infer").arg(&artifact).env("BACKEND", "cpu");
    let assert = cmd.assert().success();
    let stderr =
        String::from_utf8(assert.get_output().stderr.clone()).expect("infer stderr must be utf8");
    let artifact_load_ok = stderr_event(&stderr, "artifact_load_ok");
    assert_precision_fields(
        artifact_load_ok
            .get("fields")
            .and_then(Value::as_object)
            .expect("artifact_load_ok fields must be an object"),
    );
    let infer_done = stderr_event(&stderr, "infer_done");
    assert_precision_fields(
        infer_done
            .get("fields")
            .and_then(Value::as_object)
            .expect("infer_done fields must be an object"),
    );
    let normalized = normalize_infer_done_event(&infer_done);
    let expected = fixture_json("infer_done_event.fixture.json");
    assert_eq!(
        normalized, expected,
        "infer_done event must match the fixture-backed contract"
    );
}

fn fixture_json(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name);
    let text = fs::read_to_string(path).expect("read fixture");
    serde_json::from_str(&text).expect("parse fixture")
}

fn stderr_event(stderr: &str, event_name: &str) -> Value {
    stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| event.get("event").and_then(Value::as_str) == Some(event_name))
        .unwrap_or_else(|| panic!("missing event `{event_name}` in stderr: {stderr}"))
}

fn normalize_infer_done_event(event: &Value) -> Value {
    let mut normalized = event.clone();
    let object = normalized
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

    let elapsed_ms = fields
        .get("duration_ms")
        .and_then(Value::as_u64)
        .expect("duration_ms must be numeric");
    assert!(elapsed_ms >= 1, "duration_ms must be positive");
    fields.insert("duration_ms".to_string(), Value::from(1));

    assert_eq!(
        fields.get("batch_size").and_then(Value::as_u64),
        Some(1),
        "cli infer_done must report batch_size=1"
    );

    normalized
}

fn assert_precision_fields(fields: &serde_json::Map<String, Value>) {
    let precision = fields
        .get("precision")
        .and_then(Value::as_object)
        .expect("precision fields must be present");
    assert_eq!(
        precision.get("weights_dtype").and_then(Value::as_str),
        Some("f32")
    );
    assert_eq!(
        precision.get("activation_dtype").and_then(Value::as_str),
        Some("f32")
    );
    assert_eq!(
        precision.get("quantization").and_then(Value::as_str),
        Some("none")
    );
}
